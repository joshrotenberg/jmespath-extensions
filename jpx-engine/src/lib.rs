//! # jpx_engine
//!
//! The JMESPath query engine - a high-level platform built on `jmespath_extensions`.
//!
//! This crate provides the "brain" of jpx: everything you can do with JMESPath
//! beyond basic compile and evaluate. It's protocol-agnostic - the CLI, MCP server,
//! REST API, and gRPC server are all thin adapters over this engine.
//!
//! ## Features
//!
//! - **Evaluation**: Single, batch, and file-based evaluation with validation
//! - **Introspection**: List functions, search by keyword, describe, find similar
//! - **Discovery**: Cross-server tool discovery with BM25 search indexing
//! - **Query Store**: Named queries for reuse
//! - **JSON Utilities**: Format, diff, patch, merge, stats, paths, keys
//!
//! ## Usage
//!
//! ```rust
//! use jpx_engine::JpxEngine;
//! use serde_json::json;
//!
//! let engine = JpxEngine::new();
//!
//! // Evaluate a JMESPath expression
//! let result = engine.evaluate("users[*].name", &json!({
//!     "users": [{"name": "alice"}, {"name": "bob"}]
//! })).unwrap();
//! assert_eq!(result, json!(["alice", "bob"]));
//!
//! // Search for functions
//! let results = engine.search_functions("string", 10);
//! assert!(!results.is_empty());
//!
//! // Describe a function
//! let info = engine.describe_function("upper").unwrap();
//! assert_eq!(info.name, "upper");
//! ```
//!
//! ## Architecture
//!
//! ```text
//! jmespath_extensions    (400+ functions, registry)
//!         ↓
//!    jpx_engine          (this crate - evaluation, search, discovery)
//!         ↓
//!    ┌────┴────┐
//!    ↓         ↓
//!   jpx    jpx-server    (CLI and network transports)
//! ```

mod bm25;
mod discovery;
mod error;
mod query_store;
mod types;

pub use bm25::{Bm25Index, DocInfo, IndexOptions, SearchResult as Bm25SearchResult, TermInfo};
pub use discovery::{
    CategoryInfo, CategorySummary, DiscoveryRegistry, DiscoverySpec, ExampleSpec, IndexStats,
    ParamSpec, RegistrationResult, ReturnSpec, ServerInfo, ServerSummary, ToolQueryResult,
    ToolSpec,
};
pub use error::{EngineError, Result};
pub use query_store::{QueryStore, StoredQuery};
pub use types::{
    BatchEvaluateResult, BatchExpressionResult, EvalRequest, EvalResponse, ValidationResult,
};

use jmespath::Runtime;
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{FunctionRegistry, expand_search_terms, lookup_synonyms};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strsim::jaro_winkler;

// Re-export commonly used types from jmespath_extensions
pub use jmespath_extensions::registry::{Category, FunctionInfo};

/// Serializable function detail for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDetail {
    pub name: String,
    pub category: String,
    pub description: String,
    pub signature: String,
    pub example: String,
    pub is_standard: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jep: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl From<&FunctionInfo> for FunctionDetail {
    fn from(info: &FunctionInfo) -> Self {
        Self {
            name: info.name.to_string(),
            category: format!("{:?}", info.category),
            description: info.description.to_string(),
            signature: info.signature.to_string(),
            example: info.example.to_string(),
            is_standard: info.is_standard,
            jep: info.jep.map(|s| s.to_string()),
            aliases: info.aliases.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Search result with match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Function details
    pub function: FunctionDetail,
    /// How the function matched the query
    pub match_type: String,
    /// Relevance score (higher = better match)
    pub score: i32,
}

/// Similar functions result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFunctionsResult {
    /// Functions in the same category
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub same_category: Vec<FunctionDetail>,
    /// Functions with similar signatures
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub similar_signature: Vec<FunctionDetail>,
    /// Functions with related concepts (based on description keywords)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_concepts: Vec<FunctionDetail>,
}

/// Statistics about JSON data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    /// Type of the root value
    pub root_type: String,
    /// Estimated size in bytes
    pub size_bytes: usize,
    /// Human-readable size
    pub size_human: String,
    /// Nesting depth
    pub depth: usize,
    /// For arrays: number of items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    /// For objects: number of keys
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_count: Option<usize>,
    /// For arrays of objects: field analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldAnalysis>>,
    /// Type distribution for arrays
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_distribution: Option<HashMap<String, usize>>,
}

/// Field analysis for arrays of objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAnalysis {
    /// Field name
    pub name: String,
    /// Predominant type
    pub field_type: String,
    /// Count of null values
    pub null_count: usize,
    /// Number of unique values (for low-cardinality fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_count: Option<usize>,
}

/// Path information in JSON structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    /// The path in dot notation
    pub path: String,
    /// The type at this path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_type: Option<String>,
    /// The value at this path (for leaf nodes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// The JMESPath query engine.
///
/// This is the main entry point for all jpx functionality. It wraps the JMESPath
/// runtime with additional capabilities for introspection, search, and discovery.
pub struct JpxEngine {
    /// JMESPath runtime with all extensions registered
    runtime: Runtime,
    /// Function registry for introspection
    registry: FunctionRegistry,
    /// Discovery registry for cross-server tool search
    discovery: Arc<RwLock<DiscoveryRegistry>>,
    /// Query store for named queries
    queries: Arc<RwLock<QueryStore>>,
    /// Whether to use strict mode (standard JMESPath only)
    strict: bool,
}

impl JpxEngine {
    /// Create a new engine with all extension functions enabled.
    pub fn new() -> Self {
        Self::with_options(false)
    }

    /// Create a new engine with strict mode option.
    ///
    /// In strict mode, only standard JMESPath functions are available for evaluation.
    /// Introspection and discovery features still work for all functions.
    pub fn with_options(strict: bool) -> Self {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        if !strict {
            register_all(&mut runtime);
        }

        let mut registry = FunctionRegistry::new();
        registry.register_all();

        Self {
            runtime,
            registry,
            discovery: Arc::new(RwLock::new(DiscoveryRegistry::new())),
            queries: Arc::new(RwLock::new(QueryStore::new())),
            strict,
        }
    }

    /// Create a new engine in strict mode (standard JMESPath only).
    pub fn strict() -> Self {
        Self::with_options(true)
    }

    /// Check if the engine is in strict mode.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Get a reference to the underlying JMESPath runtime.
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Get a reference to the function registry.
    pub fn registry(&self) -> &FunctionRegistry {
        &self.registry
    }

    /// Get a reference to the discovery registry.
    pub fn discovery(&self) -> &Arc<RwLock<DiscoveryRegistry>> {
        &self.discovery
    }

    /// Get a reference to the query store.
    pub fn queries(&self) -> &Arc<RwLock<QueryStore>> {
        &self.queries
    }

    // =========================================================================
    // Evaluation methods
    // =========================================================================

    /// Evaluate a JMESPath expression against JSON input.
    pub fn evaluate(&self, expression: &str, input: &Value) -> Result<Value> {
        let expr = jmespath::compile(expression)
            .map_err(|e| EngineError::InvalidExpression(e.to_string()))?;

        let result = expr
            .search(input)
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        // Convert Rcvar to Value
        let value: Value = serde_json::to_value(result.as_ref())
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        Ok(value)
    }

    /// Evaluate a JMESPath expression against JSON input string.
    pub fn evaluate_str(&self, expression: &str, input: &str) -> Result<Value> {
        let json: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        self.evaluate(expression, &json)
    }

    /// Evaluate multiple expressions against the same input.
    pub fn batch_evaluate(&self, expressions: &[String], input: &Value) -> BatchEvaluateResult {
        let results = expressions
            .iter()
            .map(|expr| match self.evaluate(expr, input) {
                Ok(result) => BatchExpressionResult {
                    expression: expr.clone(),
                    result: Some(result),
                    error: None,
                },
                Err(e) => BatchExpressionResult {
                    expression: expr.clone(),
                    result: None,
                    error: Some(e.to_string()),
                },
            })
            .collect();

        BatchEvaluateResult { results }
    }

    /// Validate a JMESPath expression without evaluating it.
    pub fn validate(&self, expression: &str) -> ValidationResult {
        match jmespath::compile(expression) {
            Ok(_) => ValidationResult {
                valid: true,
                error: None,
            },
            Err(e) => ValidationResult {
                valid: false,
                error: Some(e.to_string()),
            },
        }
    }

    // =========================================================================
    // Introspection methods
    // =========================================================================

    /// List all available function categories.
    pub fn categories(&self) -> Vec<String> {
        Category::all().iter().map(|c| format!("{:?}", c)).collect()
    }

    /// List all functions, optionally filtered by category.
    pub fn functions(&self, category: Option<&str>) -> Vec<FunctionDetail> {
        match category.and_then(parse_category) {
            Some(cat) => self
                .registry
                .functions_in_category(cat)
                .map(FunctionDetail::from)
                .collect(),
            None => self
                .registry
                .functions()
                .map(FunctionDetail::from)
                .collect(),
        }
    }

    /// Describe a function by name or alias.
    pub fn describe_function(&self, name: &str) -> Option<FunctionDetail> {
        self.registry.get_function(name).map(FunctionDetail::from)
    }

    /// Search for functions matching a query.
    pub fn search_functions(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();

        // Expand query terms using synonyms
        let expanded_terms = expand_search_terms(&query_lower);

        let all_functions: Vec<_> = self.registry.functions().collect();
        let mut results: Vec<SearchResult> = Vec::new();

        for info in &all_functions {
            let name_lower = info.name.to_lowercase();
            let desc_lower = info.description.to_lowercase();
            let category_lower = format!("{:?}", info.category).to_lowercase();
            let signature_lower = info.signature.to_lowercase();
            let aliases_lower: Vec<String> = info
                .aliases
                .iter()
                .map(|a: &&str| a.to_lowercase())
                .collect();

            // Calculate match score and type
            let (score, match_type) = calculate_match_score(
                &query_lower,
                &expanded_terms,
                &MatchContext {
                    name: &name_lower,
                    aliases: &aliases_lower,
                    category: &category_lower,
                    description: &desc_lower,
                    signature: &signature_lower,
                },
            );

            if score > 0 {
                results.push(SearchResult {
                    function: FunctionDetail::from(*info),
                    match_type,
                    score,
                });
            }
        }

        // Sort by score descending, then by name
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.function.name.cmp(&b.function.name))
        });

        results.truncate(limit);
        results
    }

    /// Find functions similar to a given function.
    pub fn similar_functions(&self, name: &str) -> Option<SimilarFunctionsResult> {
        let info = self.registry.get_function(name)?;
        let all_functions: Vec<_> = self.registry.functions().collect();

        // Same category
        let same_category: Vec<FunctionDetail> = all_functions
            .iter()
            .filter(|f| f.category == info.category && f.name != info.name)
            .take(5)
            .map(|f| FunctionDetail::from(*f))
            .collect();

        // Similar signature (same arity)
        let this_arity = count_params(info.signature);
        let similar_signature: Vec<FunctionDetail> = all_functions
            .iter()
            .filter(|f| {
                f.name != info.name
                    && f.category != info.category
                    && count_params(f.signature) == this_arity
            })
            .take(5)
            .map(|f| FunctionDetail::from(*f))
            .collect();

        // Related concepts (description keyword overlap)
        let keywords = extract_keywords(info.description);
        let mut concept_scores: Vec<(&FunctionInfo, usize)> = all_functions
            .iter()
            .filter(|f| f.name != info.name)
            .map(|f| {
                let f_keywords = extract_keywords(f.description);
                let overlap = keywords.iter().filter(|k| f_keywords.contains(*k)).count();
                (*f, overlap)
            })
            .filter(|(_, score)| *score > 0)
            .collect();

        concept_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let related_concepts: Vec<FunctionDetail> = concept_scores
            .into_iter()
            .take(5)
            .map(|(f, _)| FunctionDetail::from(f))
            .collect();

        Some(SimilarFunctionsResult {
            same_category,
            similar_signature,
            related_concepts,
        })
    }

    // =========================================================================
    // JSON utility methods
    // =========================================================================

    /// Format JSON with indentation.
    pub fn format_json(&self, input: &str, indent: usize) -> Result<String> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        if indent == 0 {
            serde_json::to_string(&value).map_err(|e| EngineError::Internal(e.to_string()))
        } else {
            let indent_bytes = vec![b' '; indent];
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_bytes);
            let mut buf = Vec::new();
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            value
                .serialize(&mut ser)
                .map_err(|e| EngineError::Internal(e.to_string()))?;
            String::from_utf8(buf).map_err(|e| EngineError::Internal(e.to_string()))
        }
    }

    /// Generate a JSON Patch (RFC 6902) from source to target.
    pub fn diff(&self, source: &str, target: &str) -> Result<Value> {
        let source_val: Value =
            serde_json::from_str(source).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let target_val: Value =
            serde_json::from_str(target).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let patch = json_patch::diff(&source_val, &target_val);
        serde_json::to_value(&patch).map_err(|e| EngineError::Internal(e.to_string()))
    }

    /// Apply a JSON Patch (RFC 6902) to a document.
    pub fn patch(&self, input: &str, patch: &str) -> Result<Value> {
        let mut doc: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let patch: json_patch::Patch =
            serde_json::from_str(patch).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        json_patch::patch(&mut doc, &patch)
            .map_err(|e| EngineError::EvaluationFailed(e.to_string()))?;

        Ok(doc)
    }

    /// Apply a JSON Merge Patch (RFC 7396) to a document.
    pub fn merge(&self, input: &str, patch: &str) -> Result<Value> {
        let mut doc: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;
        let patch_val: Value =
            serde_json::from_str(patch).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        json_patch::merge(&mut doc, &patch_val);
        Ok(doc)
    }

    /// Extract keys from a JSON object.
    pub fn keys(&self, input: &str, recursive: bool) -> Result<Vec<String>> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let mut keys = Vec::new();
        if recursive {
            extract_keys_recursive(&value, "", &mut keys);
        } else if let Value::Object(map) = &value {
            keys = map.keys().cloned().collect();
            keys.sort();
        }
        Ok(keys)
    }

    /// Extract all paths from JSON data.
    pub fn paths(
        &self,
        input: &str,
        include_types: bool,
        include_values: bool,
    ) -> Result<Vec<PathInfo>> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let mut paths = Vec::new();
        extract_paths(&value, "", include_types, include_values, &mut paths);
        Ok(paths)
    }

    /// Analyze JSON data and return statistics.
    pub fn stats(&self, input: &str) -> Result<StatsResult> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| EngineError::InvalidJson(e.to_string()))?;

        let size_bytes = input.len();
        let depth = calculate_depth(&value);

        let (length, key_count, fields, type_distribution) = match &value {
            Value::Array(arr) => {
                let type_dist = calculate_type_distribution(arr);
                let field_analysis = if arr.iter().all(|v| v.is_object()) {
                    Some(analyze_array_fields(arr))
                } else {
                    None
                };
                (Some(arr.len()), None, field_analysis, Some(type_dist))
            }
            Value::Object(map) => (None, Some(map.len()), None, None),
            _ => (None, None, None, None),
        };

        Ok(StatsResult {
            root_type: json_type_name(&value).to_string(),
            size_bytes,
            size_human: format_size(size_bytes),
            depth,
            length,
            key_count,
            fields,
            type_distribution,
        })
    }

    // =========================================================================
    // Query store methods
    // =========================================================================

    /// Define (store) a named query.
    pub fn define_query(
        &self,
        name: String,
        expression: String,
        description: Option<String>,
    ) -> Result<Option<StoredQuery>> {
        // Validate expression first
        let validation = self.validate(&expression);
        if !validation.valid {
            return Err(EngineError::InvalidExpression(
                validation
                    .error
                    .unwrap_or_else(|| "Invalid expression".to_string()),
            ));
        }

        let query = StoredQuery {
            name,
            expression,
            description,
        };

        self.queries
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .define(query)
            .pipe(Ok)
    }

    /// Get a stored query by name.
    pub fn get_query(&self, name: &str) -> Result<Option<StoredQuery>> {
        Ok(self
            .queries
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .get(name)
            .cloned())
    }

    /// Delete a stored query.
    pub fn delete_query(&self, name: &str) -> Result<Option<StoredQuery>> {
        Ok(self
            .queries
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .delete(name))
    }

    /// List all stored queries.
    pub fn list_queries(&self) -> Result<Vec<StoredQuery>> {
        Ok(self
            .queries
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .list()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Run a stored query.
    pub fn run_query(&self, name: &str, input: &Value) -> Result<Value> {
        let query = self
            .get_query(name)?
            .ok_or_else(|| EngineError::QueryNotFound(name.to_string()))?;

        self.evaluate(&query.expression, input)
    }

    // =========================================================================
    // Discovery methods
    // =========================================================================

    /// Register a discovery spec.
    pub fn register_discovery(
        &self,
        spec: DiscoverySpec,
        replace: bool,
    ) -> Result<RegistrationResult> {
        Ok(self
            .discovery
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .register(spec, replace))
    }

    /// Unregister a server from discovery.
    pub fn unregister_discovery(&self, server_name: &str) -> Result<bool> {
        Ok(self
            .discovery
            .write()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .unregister(server_name))
    }

    /// Query tools across registered servers.
    pub fn query_tools(&self, query: &str, top_k: usize) -> Result<Vec<ToolQueryResult>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .query(query, top_k))
    }

    /// Find tools similar to a given tool.
    pub fn similar_tools(&self, tool_id: &str, top_k: usize) -> Result<Vec<ToolQueryResult>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .similar(tool_id, top_k))
    }

    /// List all registered discovery servers.
    pub fn list_discovery_servers(&self) -> Result<Vec<ServerSummary>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .list_servers())
    }

    /// List discovery categories.
    pub fn list_discovery_categories(&self) -> Result<HashMap<String, CategorySummary>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .list_categories())
    }

    /// Get discovery index stats.
    pub fn discovery_index_stats(&self) -> Result<Option<IndexStats>> {
        Ok(self
            .discovery
            .read()
            .map_err(|e| EngineError::Internal(e.to_string()))?
            .index_stats())
    }

    /// Get the discovery schema.
    pub fn get_discovery_schema(&self) -> Value {
        DiscoveryRegistry::get_schema()
    }
}

impl Default for JpxEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Context for calculating match scores
struct MatchContext<'a> {
    name: &'a str,
    aliases: &'a [String],
    category: &'a str,
    description: &'a str,
    signature: &'a str,
}

/// Calculate match score and type for a function
fn calculate_match_score(
    query: &str,
    expanded_terms: &[String],
    ctx: &MatchContext,
) -> (i32, String) {
    // Exact name match
    if ctx.name == query {
        return (1000, "exact_name".to_string());
    }

    // Alias match
    if ctx.aliases.iter().any(|a| a == query) {
        return (900, "alias".to_string());
    }

    // Name starts with query
    if ctx.name.starts_with(query) {
        return (800, "name_prefix".to_string());
    }

    // Name contains query
    if ctx.name.contains(query) {
        return (700, "name_contains".to_string());
    }

    // Category match
    if ctx.category == query {
        return (600, "category".to_string());
    }

    // Check expanded terms in description/signature
    let mut desc_score = 0;
    let mut matched_terms = Vec::new();

    for term in expanded_terms {
        if ctx.description.contains(term) || ctx.signature.contains(term) {
            desc_score += 100;
            matched_terms.push(term.clone());
        }
    }

    if desc_score > 0 {
        return (
            desc_score,
            format!("description ({})", matched_terms.join(", ")),
        );
    }

    // Fuzzy name match using Jaro-Winkler
    let similarity = jaro_winkler(query, ctx.name);
    if similarity > 0.8 {
        return ((similarity * 500.0) as i32, "fuzzy_name".to_string());
    }

    // Check synonyms
    if let Some(synonyms) = lookup_synonyms(query) {
        for syn in synonyms {
            if ctx.name.contains(syn) || ctx.description.contains(syn) {
                return (300, format!("synonym ({})", syn));
            }
        }
    }

    (0, String::new())
}

/// Parse category string to Category enum
fn parse_category(name: &str) -> Option<Category> {
    Category::all()
        .iter()
        .find(|cat| format!("{:?}", cat).to_lowercase() == name.to_lowercase())
        .copied()
}

/// Count parameters in a function signature
fn count_params(signature: &str) -> usize {
    signature.matches(',').count() + 1
}

/// Extract keywords from a description for related concept matching
fn extract_keywords(description: &str) -> Vec<&str> {
    let stopwords = [
        "a",
        "an",
        "the",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "must",
        "shall",
        "can",
        "to",
        "of",
        "in",
        "for",
        "on",
        "with",
        "at",
        "by",
        "from",
        "or",
        "and",
        "as",
        "if",
        "that",
        "which",
        "this",
        "these",
        "those",
        "it",
        "its",
        "such",
        "when",
        "where",
        "how",
        "all",
        "each",
        "every",
        "both",
        "few",
        "more",
        "most",
        "other",
        "some",
        "any",
        "no",
        "not",
        "only",
        "same",
        "than",
        "very",
        "just",
        "also",
        "into",
        "over",
        "after",
        "before",
        "between",
        "under",
        "again",
        "further",
        "then",
        "once",
        "here",
        "there",
        "why",
        "because",
        "while",
        "although",
        "though",
        "unless",
        "until",
        "whether",
        "returns",
        "return",
        "value",
        "values",
        "given",
        "input",
        "output",
        "function",
        "functions",
        "used",
        "using",
        "use",
    ];

    description
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(&w.to_lowercase().as_str()))
        .collect()
}

/// Extract keys recursively from a JSON value
fn extract_keys_recursive(value: &Value, prefix: &str, keys: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                keys.push(path.clone());
                extract_keys_recursive(v, &path, keys);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let path = format!("{}.{}", prefix, i);
                extract_keys_recursive(v, &path, keys);
            }
        }
        _ => {}
    }
}

/// Extract paths from a JSON value
fn extract_paths(
    value: &Value,
    prefix: &str,
    include_types: bool,
    include_values: bool,
    paths: &mut Vec<PathInfo>,
) {
    let current_path = if prefix.is_empty() {
        "@".to_string()
    } else {
        prefix.to_string()
    };

    match value {
        Value::Object(map) => {
            paths.push(PathInfo {
                path: current_path.clone(),
                path_type: if include_types {
                    Some("object".to_string())
                } else {
                    None
                },
                value: None,
            });
            for (k, v) in map {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                extract_paths(v, &new_prefix, include_types, include_values, paths);
            }
        }
        Value::Array(arr) => {
            paths.push(PathInfo {
                path: current_path.clone(),
                path_type: if include_types {
                    Some("array".to_string())
                } else {
                    None
                },
                value: None,
            });
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}.{}", prefix, i);
                extract_paths(v, &new_prefix, include_types, include_values, paths);
            }
        }
        _ => {
            paths.push(PathInfo {
                path: current_path,
                path_type: if include_types {
                    Some(json_type_name(value).to_string())
                } else {
                    None
                },
                value: if include_values {
                    Some(value.clone())
                } else {
                    None
                },
            });
        }
    }
}

/// Calculate the nesting depth of a JSON value
fn calculate_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => 1 + map.values().map(calculate_depth).max().unwrap_or(0),
        Value::Array(arr) => 1 + arr.iter().map(calculate_depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Get the type name of a JSON value
fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Calculate type distribution in an array
fn calculate_type_distribution(arr: &[Value]) -> HashMap<String, usize> {
    let mut dist = HashMap::new();
    for item in arr {
        *dist.entry(json_type_name(item).to_string()).or_insert(0) += 1;
    }
    dist
}

/// Analyze fields in an array of objects
fn analyze_array_fields(arr: &[Value]) -> Vec<FieldAnalysis> {
    let mut field_types: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut field_null_counts: HashMap<String, usize> = HashMap::new();
    let mut field_values: HashMap<String, Vec<Value>> = HashMap::new();

    for item in arr {
        if let Value::Object(map) = item {
            for (k, v) in map {
                let types = field_types.entry(k.clone()).or_default();
                *types.entry(json_type_name(v).to_string()).or_insert(0) += 1;

                if v.is_null() {
                    *field_null_counts.entry(k.clone()).or_insert(0) += 1;
                }

                // Track unique values for low-cardinality detection
                let values = field_values.entry(k.clone()).or_default();
                if values.len() < 100 && !values.contains(v) {
                    values.push(v.clone());
                }
            }
        }
    }

    let mut fields: Vec<FieldAnalysis> = field_types
        .into_iter()
        .map(|(name, types)| {
            let predominant_type = types
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(t, _)| t)
                .unwrap_or_else(|| "unknown".to_string());

            let null_count = field_null_counts.get(&name).copied().unwrap_or(0);
            let unique_count = field_values.get(&name).map(|v| v.len());

            FieldAnalysis {
                name,
                field_type: predominant_type,
                null_count,
                unique_count,
            }
        })
        .collect();

    fields.sort_by(|a, b| a.name.cmp(&b.name));
    fields
}

/// Format size in human-readable form
fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Extension trait for pipe-style method chaining
trait Pipe: Sized {
    fn pipe<T, F: FnOnce(Self) -> T>(self, f: F) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_engine_creation() {
        let engine = JpxEngine::new();
        assert!(!engine.is_strict());
    }

    #[test]
    fn test_engine_strict_mode() {
        let engine = JpxEngine::strict();
        assert!(engine.is_strict());
    }

    #[test]
    fn test_engine_default() {
        let engine = JpxEngine::default();
        assert!(!engine.is_strict());
    }

    #[test]
    fn test_evaluate() {
        let engine = JpxEngine::new();
        let input = json!({"users": [{"name": "alice"}, {"name": "bob"}]});
        let result = engine.evaluate("users[*].name", &input).unwrap();
        assert_eq!(result, json!(["alice", "bob"]));
    }

    #[test]
    fn test_evaluate_str() {
        let engine = JpxEngine::new();
        let result = engine.evaluate_str("length(@)", r#"[1, 2, 3]"#).unwrap();
        assert_eq!(result, json!(3));
    }

    #[test]
    fn test_batch_evaluate() {
        let engine = JpxEngine::new();
        let input = json!({"a": 1, "b": 2});
        let exprs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = engine.batch_evaluate(&exprs, &input);

        assert_eq!(result.results.len(), 3);
        assert_eq!(result.results[0].result, Some(json!(1)));
        assert_eq!(result.results[1].result, Some(json!(2)));
        assert_eq!(result.results[2].result, Some(json!(null)));
    }

    #[test]
    fn test_validate() {
        let engine = JpxEngine::new();

        let valid = engine.validate("users[*].name");
        assert!(valid.valid);
        assert!(valid.error.is_none());

        let invalid = engine.validate("users[*.name");
        assert!(!invalid.valid);
        assert!(invalid.error.is_some());
    }

    #[test]
    fn test_categories() {
        let engine = JpxEngine::new();
        let cats = engine.categories();
        assert!(!cats.is_empty());
        assert!(cats.iter().any(|c| c == "String"));
    }

    #[test]
    fn test_functions() {
        let engine = JpxEngine::new();

        // All functions
        let all = engine.functions(None);
        assert!(!all.is_empty());

        // Filtered by category
        let string_funcs = engine.functions(Some("String"));
        assert!(!string_funcs.is_empty());
        assert!(string_funcs.iter().all(|f| f.category == "String"));
    }

    #[test]
    fn test_describe_function() {
        let engine = JpxEngine::new();

        let info = engine.describe_function("upper").unwrap();
        assert_eq!(info.name, "upper");
        assert_eq!(info.category, "String");

        let missing = engine.describe_function("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_search_functions() {
        let engine = JpxEngine::new();

        let results = engine.search_functions("string", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_similar_functions() {
        let engine = JpxEngine::new();

        let result = engine.similar_functions("upper").unwrap();
        // Should have functions in same category
        assert!(!result.same_category.is_empty());
    }

    #[test]
    fn test_format_json() {
        let engine = JpxEngine::new();

        let formatted = engine.format_json(r#"{"a":1,"b":2}"#, 2).unwrap();
        assert!(formatted.contains('\n'));

        let compact = engine.format_json(r#"{"a":1,"b":2}"#, 0).unwrap();
        assert!(!compact.contains('\n'));
    }

    #[test]
    fn test_diff() {
        let engine = JpxEngine::new();

        let patch = engine.diff(r#"{"a": 1}"#, r#"{"a": 2}"#).unwrap();

        let patch_arr = patch.as_array().unwrap();
        assert!(!patch_arr.is_empty());
    }

    #[test]
    fn test_patch() {
        let engine = JpxEngine::new();

        let result = engine
            .patch(
                r#"{"a": 1}"#,
                r#"[{"op": "replace", "path": "/a", "value": 2}]"#,
            )
            .unwrap();

        assert_eq!(result, json!({"a": 2}));
    }

    #[test]
    fn test_merge() {
        let engine = JpxEngine::new();

        let result = engine
            .merge(r#"{"a": 1, "b": 2}"#, r#"{"b": 3, "c": 4}"#)
            .unwrap();

        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn test_keys() {
        let engine = JpxEngine::new();

        let keys = engine.keys(r#"{"a": 1, "b": {"c": 2}}"#, false).unwrap();
        assert_eq!(keys, vec!["a", "b"]);

        let recursive_keys = engine.keys(r#"{"a": 1, "b": {"c": 2}}"#, true).unwrap();
        assert!(recursive_keys.contains(&"b.c".to_string()));
    }

    #[test]
    fn test_paths() {
        let engine = JpxEngine::new();

        let paths = engine.paths(r#"{"a": 1}"#, true, false).unwrap();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_stats() {
        let engine = JpxEngine::new();

        let stats = engine.stats(r#"[1, 2, 3]"#).unwrap();
        assert_eq!(stats.root_type, "array");
        assert_eq!(stats.length, Some(3));
    }

    #[test]
    fn test_query_store() {
        let engine = JpxEngine::new();

        // Define a query
        engine
            .define_query("count".to_string(), "length(@)".to_string(), None)
            .unwrap();

        // Get it
        let query = engine.get_query("count").unwrap().unwrap();
        assert_eq!(query.expression, "length(@)");

        // Run it
        let result = engine.run_query("count", &json!([1, 2, 3])).unwrap();
        assert_eq!(result, json!(3));

        // List queries
        let queries = engine.list_queries().unwrap();
        assert_eq!(queries.len(), 1);

        // Delete it
        engine.delete_query("count").unwrap();
        assert!(engine.get_query("count").unwrap().is_none());
    }

    #[test]
    fn test_discovery() {
        let engine = JpxEngine::new();

        let spec: DiscoverySpec = serde_json::from_value(json!({
            "server": {"name": "test-server", "version": "1.0.0"},
            "tools": [
                {"name": "test_tool", "description": "A test tool", "tags": ["test"]}
            ]
        }))
        .unwrap();

        // Register
        let result = engine.register_discovery(spec, false).unwrap();
        assert!(result.ok);
        assert_eq!(result.tools_indexed, 1);

        // List servers
        let servers = engine.list_discovery_servers().unwrap();
        assert_eq!(servers.len(), 1);

        // Query tools
        let tools = engine.query_tools("test", 10).unwrap();
        assert!(!tools.is_empty());

        // Unregister
        assert!(engine.unregister_discovery("test-server").unwrap());
        assert!(engine.list_discovery_servers().unwrap().is_empty());
    }
}
