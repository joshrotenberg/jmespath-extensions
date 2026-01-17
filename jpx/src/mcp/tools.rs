//! MCP tool implementations for JMESPath

use jmespath::Runtime;
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{
    Category, FunctionInfo, FunctionRegistry, expand_search_terms, lookup_synonyms,
};
use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{OnceLock, RwLock};
use strsim::jaro_winkler;

use super::discovery::{DiscoveryRegistry, DiscoverySpec};
use super::query_store::{self, StoredQuery};

/// Global JMESPath runtime with all extensions registered
static RUNTIME_FULL: OnceLock<Runtime> = OnceLock::new();

/// Global JMESPath runtime with only standard functions (strict mode)
static RUNTIME_STRICT: OnceLock<Runtime> = OnceLock::new();

/// Global function registry for introspection
static REGISTRY: OnceLock<FunctionRegistry> = OnceLock::new();

/// Global discovery registry for MCP tool discovery
static DISCOVERY_REGISTRY: OnceLock<RwLock<DiscoveryRegistry>> = OnceLock::new();

/// Get the global discovery registry
fn discovery_registry() -> &'static RwLock<DiscoveryRegistry> {
    DISCOVERY_REGISTRY.get_or_init(|| RwLock::new(DiscoveryRegistry::new()))
}

/// Get the full JMESPath runtime (with all extensions)
fn runtime_full() -> &'static Runtime {
    RUNTIME_FULL.get_or_init(|| {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register_all(&mut runtime);
        runtime
    })
}

/// Get the strict JMESPath runtime (standard functions only)
fn runtime_strict() -> &'static Runtime {
    RUNTIME_STRICT.get_or_init(|| {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        // No extensions registered
        runtime
    })
}

/// Get the global function registry
fn registry() -> &'static FunctionRegistry {
    REGISTRY.get_or_init(|| {
        let mut reg = FunctionRegistry::new();
        reg.register_all();
        reg
    })
}

// =============================================================================
// Parameter structs for MCP tools
// =============================================================================

/// Parameters for the evaluate tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvaluateParams {
    /// JSON input to evaluate the expression against
    pub input: String,
    /// JMESPath expression to evaluate
    pub expression: String,
}

/// Parameters for the functions tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FunctionsParams {
    /// Optional category filter (e.g., "String", "Math", "Array", "Datetime")
    #[serde(default)]
    pub category: Option<String>,
}

/// Parameters for the describe tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DescribeParams {
    /// Function name or alias to describe
    pub name: String,
}

/// Parameters for the validate tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    /// JMESPath expression to validate
    pub expression: String,
}

/// Parameters for the batch_evaluate tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatchEvaluateParams {
    /// JSON input to evaluate the expressions against
    pub input: String,
    /// List of JMESPath expressions to evaluate
    pub expressions: Vec<String>,
}

/// Parameters for the format tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FormatParams {
    /// JSON string to format
    pub input: String,
    /// Number of spaces for indentation (default: 2, use 0 for compact)
    #[serde(default = "default_indent")]
    pub indent: usize,
}

fn default_indent() -> usize {
    2
}

/// Parameters for the diff tool (RFC 6902)
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffParams {
    /// Source JSON document
    pub source: String,
    /// Target JSON document
    pub target: String,
}

/// Parameters for the patch tool (RFC 6902)
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatchParams {
    /// JSON document to patch
    pub input: String,
    /// JSON Patch operations array (RFC 6902)
    pub patch: String,
}

/// Parameters for the merge tool (RFC 7396)
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeParams {
    /// JSON document to merge into
    pub input: String,
    /// JSON Merge Patch document (RFC 7396)
    pub patch: String,
}

/// Parameters for the keys tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KeysParams {
    /// JSON document to extract keys from
    pub input: String,
    /// If true, recursively extract all keys using dot notation (default: false)
    #[serde(default)]
    pub recursive: bool,
}

/// Parameters for the evaluate_file tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvaluateFileParams {
    /// Path to the JSON file to read
    pub file_path: String,
    /// JMESPath expression to evaluate
    pub expression: String,
}

/// Parameters for the search tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    /// Search query to match against function names, descriptions, categories, or signatures
    pub query: String,
    /// Maximum number of results to return (default: 20)
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

fn default_search_limit() -> usize {
    20
}

/// Parameters for the similar tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SimilarParams {
    /// Function name to find similar functions for
    pub function: String,
}

/// Parameters for the stats tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StatsParams {
    /// JSON input to analyze
    pub input: String,
}

/// Parameters for the paths tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PathsParams {
    /// JSON input to extract paths from
    pub input: String,
    /// Include type information for each path (default: true)
    #[serde(default = "default_true")]
    pub include_types: bool,
    /// Include values for leaf paths (default: false)
    #[serde(default)]
    pub include_values: bool,
}

fn default_true() -> bool {
    true
}

// =============================================================================
// Discovery tool parameters
// =============================================================================

/// Parameters for the get_discovery_schema tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetDiscoverySchemaParams {
    /// Schema version (optional, defaults to latest)
    #[serde(default)]
    #[allow(dead_code)]
    pub version: Option<String>,
}

/// Parameters for the register_discovery tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterDiscoveryParams {
    /// The discovery spec containing server info and tool definitions
    pub spec: DiscoverySpec,
    /// Replace existing registration if server already registered (default: false)
    #[serde(default)]
    pub replace: bool,
}

/// Parameters for the query_tools tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct QueryToolsParams {
    /// Search query (searches across name, description, tags, etc.)
    pub query: String,
    /// Maximum number of results to return (default: 10)
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

/// Parameters for the similar_tools tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SimilarToolsParams {
    /// Tool ID in format "server:tool_name"
    pub tool_id: String,
    /// Maximum number of similar tools to return (default: 5)
    #[serde(default = "default_similar_k")]
    pub top_k: usize,
}

fn default_similar_k() -> usize {
    5
}

/// Parameters for the unregister_discovery tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UnregisterDiscoveryParams {
    /// Server name to unregister
    pub server_name: String,
}

// =============================================================================
// Query store tool parameters
// =============================================================================

/// Parameters for the define_query tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DefineQueryParams {
    /// Unique name for the query (used to reference it later)
    pub name: String,
    /// JMESPath expression
    pub expression: String,
    /// Optional description of what the query does
    #[serde(default)]
    pub description: Option<String>,
}

/// Parameters for the get_query tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetQueryParams {
    /// Name of the query to retrieve
    pub name: String,
}

/// Parameters for the delete_query tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteQueryParams {
    /// Name of the query to delete
    pub name: String,
}

/// Parameters for the run_query tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunQueryParams {
    /// Name of the stored query to run
    pub name: String,
    /// JSON input to evaluate the query against
    pub input: String,
}

// =============================================================================
// Response types
// =============================================================================

/// Serializable function info for MCP responses
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

/// Validation result
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for a single expression in a batch evaluation
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchExpressionResult {
    /// The expression that was evaluated
    pub expression: String,
    /// The result if successful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The error message if evaluation failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for batch evaluation
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchEvaluateResult {
    /// Results for each expression in order
    pub results: Vec<BatchExpressionResult>,
}

/// Search result with match information
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Function details
    pub function: FunctionDetail,
    /// How the function matched the query
    pub match_type: String,
    /// Relevance score (higher = better match)
    pub score: i32,
}

/// Search response with results and suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Matching functions
    pub results: Vec<SearchResult>,
    /// "Did you mean" suggestions when no exact matches found
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    /// Query terms that were expanded via synonyms
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expanded_terms: Vec<String>,
}

/// Similar functions result
#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarResult {
    /// Functions in the same category
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub same_category: Vec<FunctionDetail>,
    /// Functions with similar signatures
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub similar_signature: Vec<FunctionDetail>,
    /// Functions with related concepts (based on description keywords)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related_concepts: Vec<FunctionDetail>,
}

/// Statistics about JSON data
#[derive(Debug, Serialize, Deserialize)]
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
    pub type_distribution: Option<std::collections::HashMap<String, usize>>,
}

/// Field analysis for arrays of objects
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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

// =============================================================================
// Helper functions
// =============================================================================

/// Get the string name for a category.
/// This function uses exhaustive matching to ensure compile errors when new categories are added.
fn category_to_string(category: Category) -> &'static str {
    match category {
        Category::Standard => "standard",
        Category::String => "string",
        Category::Array => "array",
        Category::Object => "object",
        Category::Math => "math",
        Category::Type => "type",
        Category::Utility => "utility",
        Category::Validation => "validation",
        Category::Path => "path",
        Category::Expression => "expression",
        Category::Text => "text",
        Category::Hash => "hash",
        Category::Encoding => "encoding",
        Category::Regex => "regex",
        Category::Url => "url",
        Category::Uuid => "uuid",
        Category::Rand => "rand",
        Category::Datetime => "datetime",
        Category::Fuzzy => "fuzzy",
        Category::Phonetic => "phonetic",
        Category::Geo => "geo",
        Category::Semver => "semver",
        Category::Network => "network",
        Category::Ids => "ids",
        Category::Duration => "duration",
        Category::Color => "color",
        Category::Computing => "computing",
        Category::MultiMatch => "multimatch",
        Category::Jsonpatch => "jsonpatch",
        Category::Format => "format",
        Category::Language => "language",
        Category::Discovery => "discovery",
    }
}

/// Parse category string to Category enum.
/// Note: When adding a new category, also add it to `category_to_string` above
/// which will cause a compile error if missed.
fn parse_category(name: &str) -> Option<Category> {
    // Use Category::all() to ensure we check all categories
    Category::all()
        .iter()
        .find(|cat| category_to_string(**cat) == name.to_lowercase())
        .copied()
}

/// Create a successful text result
fn text_result(content: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(content)])
}

/// Create a successful JSON result
fn json_result(value: &impl Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value).map_err(|e| {
        McpError::internal_error(format!("Failed to serialize result: {}", e), None)
    })?;
    Ok(text_result(json))
}

/// Create an error result
fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(message)])
}

// =============================================================================
// MCP Server
// =============================================================================

/// JMESPath MCP server
#[derive(Clone)]
pub struct JpxMcp {
    tool_router: ToolRouter<JpxMcp>,
    /// If true, only standard JMESPath functions are available in evaluate tools
    strict: bool,
}

impl JpxMcp {
    /// Create a new JpxMcp server
    ///
    /// # Arguments
    /// * `strict` - If true, only standard JMESPath functions are available in evaluate tools.
    ///   JSON utility tools (format, diff, patch, merge, keys) are always available.
    pub fn new(strict: bool) -> Self {
        // Initialize the appropriate runtime and registry eagerly
        if strict {
            let _ = runtime_strict();
        } else {
            let _ = runtime_full();
        }
        let _ = registry();
        Self {
            tool_router: Self::tool_router(),
            strict,
        }
    }

    /// Get the appropriate runtime based on strict mode
    fn runtime(&self) -> &'static Runtime {
        if self.strict {
            runtime_strict()
        } else {
            runtime_full()
        }
    }
}

impl Default for JpxMcp {
    fn default() -> Self {
        Self::new(false)
    }
}

#[tool_router]
impl JpxMcp {
    /// Evaluate a JMESPath expression against JSON input
    #[tool(
        description = "Evaluate a JMESPath expression against JSON input. Returns the result of applying the expression to the input data. Supports 400+ extended functions beyond standard JMESPath."
    )]
    async fn evaluate(
        &self,
        Parameters(params): Parameters<EvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        // Compile expression (uses strict or full runtime based on server mode)
        let expr = self
            .runtime()
            .compile(&params.expression)
            .map_err(|e| McpError::invalid_params(format!("Invalid expression: {}", e), None))?;

        // Convert to jmespath Variable
        let var = jmespath::Variable::from_json(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON input: {}", e), None))?;

        // Execute
        let result = expr
            .search(&var)
            .map_err(|e| McpError::internal_error(format!("Evaluation failed: {}", e), None))?;

        // Convert result back to JSON
        let result_json: Value = serde_json::to_value(&*result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        json_result(&result_json)
    }

    /// List available JMESPath functions
    #[tool(
        description = "List available JMESPath functions. Optionally filter by category (e.g., 'String', 'Math', 'Array', 'Datetime', 'Hash', 'Encoding', etc.). Returns function names with signatures and descriptions."
    )]
    async fn functions(
        &self,
        Parameters(params): Parameters<FunctionsParams>,
    ) -> Result<CallToolResult, McpError> {
        let reg = registry();

        let functions: Vec<FunctionDetail> = match params.category {
            Some(ref cat_name) => {
                if let Some(cat) = parse_category(cat_name) {
                    reg.functions_in_category(cat)
                        .map(FunctionDetail::from)
                        .collect()
                } else {
                    return Ok(error_result(format!(
                        "Unknown category '{}'. Use the 'categories' tool to list available categories.",
                        cat_name
                    )));
                }
            }
            None => reg.functions().map(FunctionDetail::from).collect(),
        };

        json_result(&functions)
    }

    /// Get detailed information about a specific function
    #[tool(
        description = "Get detailed information about a specific JMESPath function including its signature, description, example usage, and category. Accepts function name or alias."
    )]
    async fn describe(
        &self,
        Parameters(params): Parameters<DescribeParams>,
    ) -> Result<CallToolResult, McpError> {
        let reg = registry();

        match reg.get_function_by_name_or_alias(&params.name) {
            Some(info) => {
                let detail = FunctionDetail::from(info);
                json_result(&detail)
            }
            None => Ok(error_result(format!(
                "Unknown function '{}'. Use the 'functions' tool to list available functions.",
                params.name
            ))),
        }
    }

    /// List all function categories
    #[tool(
        description = "List all available JMESPath function categories. Use these category names with the 'functions' tool to filter by category."
    )]
    async fn categories(&self) -> Result<CallToolResult, McpError> {
        let categories: Vec<String> = Category::all()
            .iter()
            .filter(|c| c.is_available())
            .map(|c| c.name().to_string())
            .collect();

        json_result(&categories)
    }

    /// Validate a JMESPath expression without executing it
    #[tool(
        description = "Validate a JMESPath expression without executing it. Returns whether the expression is syntactically valid and any parse errors."
    )]
    async fn validate(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        // Validate against strict or full runtime based on server mode
        let result = match self.runtime().compile(&params.expression) {
            Ok(_) => ValidationResult {
                valid: true,
                error: None,
            },
            Err(e) => ValidationResult {
                valid: false,
                error: Some(e.to_string()),
            },
        };

        json_result(&result)
    }

    /// Evaluate multiple JMESPath expressions against the same input
    #[tool(
        description = "Evaluate multiple JMESPath expressions against the same JSON input in a single call. Parses the input once and runs all expressions, returning results for each. Useful for extracting multiple values from the same data."
    )]
    async fn batch_evaluate(
        &self,
        Parameters(params): Parameters<BatchEvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        // Parse input JSON once
        let var = jmespath::Variable::from_json(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON input: {}", e), None))?;

        // Use strict or full runtime based on server mode
        let rt = self.runtime();

        // Evaluate each expression
        let results: Vec<BatchExpressionResult> = params
            .expressions
            .iter()
            .map(|expr_str| {
                // Compile expression
                let compiled = match rt.compile(expr_str) {
                    Ok(expr) => expr,
                    Err(e) => {
                        return BatchExpressionResult {
                            expression: expr_str.clone(),
                            result: None,
                            error: Some(format!("Compile error: {}", e)),
                        };
                    }
                };

                // Execute expression
                match compiled.search(&var) {
                    Ok(result) => {
                        // Convert to JSON Value
                        match serde_json::to_value(&*result) {
                            Ok(json_value) => BatchExpressionResult {
                                expression: expr_str.clone(),
                                result: Some(json_value),
                                error: None,
                            },
                            Err(e) => BatchExpressionResult {
                                expression: expr_str.clone(),
                                result: None,
                                error: Some(format!("Serialization error: {}", e)),
                            },
                        }
                    }
                    Err(e) => BatchExpressionResult {
                        expression: expr_str.clone(),
                        result: None,
                        error: Some(format!("Evaluation error: {}", e)),
                    },
                }
            })
            .collect();

        json_result(&BatchEvaluateResult { results })
    }

    /// Format/pretty-print JSON
    #[tool(
        description = "Format and validate JSON. Pretty-prints the input with configurable indentation. Use indent=0 for compact output. Returns an error if the input is not valid JSON."
    )]
    async fn format(
        &self,
        Parameters(params): Parameters<FormatParams>,
    ) -> Result<CallToolResult, McpError> {
        // Parse the JSON to validate and normalize it
        let value: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {}", e), None))?;

        // Format based on indent
        let formatted = if params.indent == 0 {
            serde_json::to_string(&value)
                .map_err(|e| McpError::internal_error(format!("Formatting error: {}", e), None))?
        } else {
            // Create a custom formatter with the specified indent
            let indent_str = " ".repeat(params.indent);
            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_str.as_bytes());
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            value.serialize(&mut ser).map_err(|e| {
                McpError::internal_error(format!("Serialization error: {}", e), None)
            })?;
            String::from_utf8(buf)
                .map_err(|e| McpError::internal_error(format!("UTF-8 error: {}", e), None))?
        };

        Ok(text_result(formatted))
    }

    /// Generate a JSON Patch (RFC 6902) between two documents
    #[tool(
        description = "Generate a JSON Patch (RFC 6902) that transforms the source document into the target document. Returns an array of patch operations (add, remove, replace, move, copy, test). See https://datatracker.ietf.org/doc/html/rfc6902"
    )]
    async fn diff(
        &self,
        Parameters(params): Parameters<DiffParams>,
    ) -> Result<CallToolResult, McpError> {
        // Parse source JSON
        let source: Value = serde_json::from_str(&params.source)
            .map_err(|e| McpError::invalid_params(format!("Invalid source JSON: {}", e), None))?;

        // Parse target JSON
        let target: Value = serde_json::from_str(&params.target)
            .map_err(|e| McpError::invalid_params(format!("Invalid target JSON: {}", e), None))?;

        // Generate the diff
        let patch = json_patch::diff(&source, &target);

        // Convert patch to JSON value
        let patch_json = serde_json::to_value(&patch).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize patch: {}", e), None)
        })?;

        json_result(&patch_json)
    }

    /// Apply a JSON Patch (RFC 6902) to a document
    #[tool(
        description = "Apply a JSON Patch (RFC 6902) to a JSON document. The patch is an array of operations (add, remove, replace, move, copy, test). Returns the patched document or an error if the patch cannot be applied. See https://datatracker.ietf.org/doc/html/rfc6902"
    )]
    async fn patch(
        &self,
        Parameters(params): Parameters<PatchParams>,
    ) -> Result<CallToolResult, McpError> {
        // Parse input JSON
        let mut input: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid input JSON: {}", e), None))?;

        // Parse patch JSON
        let patch_value: Value = serde_json::from_str(&params.patch)
            .map_err(|e| McpError::invalid_params(format!("Invalid patch JSON: {}", e), None))?;

        // Convert to json_patch::Patch
        let patch: json_patch::Patch = serde_json::from_value(patch_value).map_err(|e| {
            McpError::invalid_params(format!("Invalid JSON Patch format: {}", e), None)
        })?;

        // Apply the patch
        json_patch::patch(&mut input, &patch)
            .map_err(|e| McpError::invalid_params(format!("Failed to apply patch: {}", e), None))?;

        json_result(&input)
    }

    /// Apply a JSON Merge Patch (RFC 7396) to a document
    #[tool(
        description = "Apply a JSON Merge Patch (RFC 7396) to a JSON document. The merge patch is a JSON document that describes changes: values are replaced, null values remove keys, and objects are merged recursively. Simpler than JSON Patch but less expressive. See https://datatracker.ietf.org/doc/html/rfc7396"
    )]
    async fn merge(
        &self,
        Parameters(params): Parameters<MergeParams>,
    ) -> Result<CallToolResult, McpError> {
        // Parse input JSON
        let mut input: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid input JSON: {}", e), None))?;

        // Parse merge patch JSON
        let patch: Value = serde_json::from_str(&params.patch).map_err(|e| {
            McpError::invalid_params(format!("Invalid merge patch JSON: {}", e), None)
        })?;

        // Apply the merge patch
        json_patch::merge(&mut input, &patch);

        json_result(&input)
    }

    /// Extract keys from a JSON object
    #[tool(
        description = "Extract keys from a JSON object. By default returns top-level keys only. Set recursive=true to get all nested keys in dot notation (e.g., 'user.profile.age'). Useful for understanding JSON structure before querying."
    )]
    async fn keys(
        &self,
        Parameters(params): Parameters<KeysParams>,
    ) -> Result<CallToolResult, McpError> {
        // Parse input JSON
        let value: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {}", e), None))?;

        let keys: Vec<String> = if params.recursive {
            // Recursively collect all keys with dot notation
            collect_keys_recursive(&value, String::new())
        } else {
            // Top-level keys only
            match &value {
                Value::Object(map) => map.keys().cloned().collect(),
                _ => vec![],
            }
        };

        json_result(&keys)
    }

    /// Evaluate a JMESPath expression against a JSON file
    #[tool(
        description = "Read a JSON file from disk and evaluate a JMESPath expression against it. More efficient than passing large JSON content through the protocol. The file must exist and contain valid JSON."
    )]
    async fn evaluate_file(
        &self,
        Parameters(params): Parameters<EvaluateFileParams>,
    ) -> Result<CallToolResult, McpError> {
        use std::path::Path;

        let path = Path::new(&params.file_path);

        // Security: Validate the path
        // 1. Must be absolute path
        if !path.is_absolute() {
            return Err(McpError::invalid_params(
                "File path must be absolute".to_string(),
                None,
            ));
        }

        // 2. Canonicalize to resolve symlinks and check for traversal
        let canonical_path = path
            .canonicalize()
            .map_err(|e| McpError::invalid_params(format!("Cannot resolve path: {}", e), None))?;

        // 3. Check file exists and is a file (not directory)
        if !canonical_path.is_file() {
            return Err(McpError::invalid_params(
                format!("Not a file: {}", canonical_path.display()),
                None,
            ));
        }

        // 4. Check file size (limit to 50MB)
        let metadata = std::fs::metadata(&canonical_path).map_err(|e| {
            McpError::invalid_params(format!("Cannot read file metadata: {}", e), None)
        })?;
        const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
        if metadata.len() > MAX_FILE_SIZE {
            return Err(McpError::invalid_params(
                format!(
                    "File too large: {} bytes (max {} bytes)",
                    metadata.len(),
                    MAX_FILE_SIZE
                ),
                None,
            ));
        }

        // Read the file
        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| McpError::invalid_params(format!("Cannot read file: {}", e), None))?;

        // Compile expression (uses strict or full runtime based on server mode)
        let expr = self
            .runtime()
            .compile(&params.expression)
            .map_err(|e| McpError::invalid_params(format!("Invalid expression: {}", e), None))?;

        // Parse JSON
        let var = jmespath::Variable::from_json(&content)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON in file: {}", e), None))?;

        // Execute
        let result = expr
            .search(&var)
            .map_err(|e| McpError::internal_error(format!("Evaluation failed: {}", e), None))?;

        // Convert result back to JSON
        let result_json: Value = serde_json::to_value(&*result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        json_result(&result_json)
    }

    /// Search for functions by name, description, category, or signature
    #[tool(
        description = "Search for JMESPath functions using fuzzy matching. Searches function names, descriptions, categories, signatures, and aliases. Returns ranked results with match type and relevance score. Supports synonym expansion (e.g., 'aggregate' finds 'group_by') and provides 'did you mean' suggestions when no exact matches are found."
    )]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let reg = registry();
        let query_lower = params.query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();
        let mut expanded_terms: Vec<String> = Vec::new();

        // Expand query using synonyms
        let search_terms = expand_search_terms(&params.query);

        // Track which terms were expanded from synonyms
        for word in params.query.split_whitespace() {
            let word_lower = word.to_lowercase();
            if let Some(targets) = lookup_synonyms(&word_lower) {
                for target in targets {
                    if !expanded_terms.contains(&(*target).to_string()) {
                        expanded_terms.push((*target).to_string());
                    }
                }
            }
        }

        for func in reg.functions() {
            let name_lower = func.name.to_lowercase();
            let desc_lower = func.description.to_lowercase();
            let sig_lower = func.signature.to_lowercase();
            let cat_lower = func.category.name().to_lowercase();

            // Check for matches with the original query first (highest priority)
            let (score, match_type) = if name_lower == query_lower {
                (1000, "exact_name")
            } else if name_lower.starts_with(&query_lower) {
                (800, "name_prefix")
            } else if name_lower.contains(&query_lower) {
                (600, "name_contains")
            } else if func
                .aliases
                .iter()
                .any(|a| a.to_lowercase().contains(&query_lower))
            {
                (500, "alias_match")
            } else if cat_lower.contains(&query_lower) {
                (400, "category_match")
            } else if desc_lower.contains(&query_lower) {
                // Boost if query appears early in description
                let pos = desc_lower.find(&query_lower).unwrap_or(100);
                (300 - pos.min(100) as i32, "description_match")
            } else if sig_lower.contains(&query_lower) {
                (100, "signature_match")
            } else {
                // Check synonym-expanded terms
                let mut synonym_match: Option<(i32, &str)> = None;
                for term in &search_terms {
                    if term == &query_lower {
                        continue; // Already checked original query
                    }
                    if name_lower == *term {
                        synonym_match = Some((450, "synonym_exact"));
                        break;
                    } else if name_lower.contains(term.as_str()) {
                        synonym_match = Some((350, "synonym_contains"));
                        break;
                    } else if desc_lower.contains(term.as_str()) {
                        synonym_match = Some((250, "synonym_description"));
                        break;
                    }
                }

                if let Some((s, t)) = synonym_match {
                    (s, t)
                } else {
                    continue; // No match
                }
            };

            results.push(SearchResult {
                function: FunctionDetail::from(func),
                match_type: match_type.to_string(),
                score,
            });
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        // Limit results
        results.truncate(params.limit);

        // If no results, find fuzzy suggestions using Jaro-Winkler
        let suggestions = if results.is_empty() {
            let mut fuzzy_matches: Vec<(String, f64)> = Vec::new();
            let threshold = 0.7; // Minimum similarity score

            for func in reg.functions() {
                let similarity = jaro_winkler(&query_lower, &func.name.to_lowercase());
                if similarity >= threshold {
                    fuzzy_matches.push((func.name.to_string(), similarity));
                }
                // Also check aliases
                for alias in func.aliases {
                    let alias_sim = jaro_winkler(&query_lower, &alias.to_lowercase());
                    if alias_sim >= threshold {
                        fuzzy_matches.push((func.name.to_string(), alias_sim));
                    }
                }
            }

            // Sort by similarity descending and dedupe
            fuzzy_matches
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let mut seen = std::collections::HashSet::new();
            fuzzy_matches
                .into_iter()
                .filter(|(name, _)| seen.insert(name.clone()))
                .take(5)
                .map(|(name, _)| name)
                .collect()
        } else {
            Vec::new()
        };

        // Build response
        let response = SearchResponse {
            results,
            suggestions,
            expanded_terms,
        };

        if response.results.is_empty() && response.suggestions.is_empty() {
            Ok(error_result(format!(
                "No functions found matching '{}'. Try broader search terms like 'string', 'array', 'date', 'hash', etc.",
                params.query
            )))
        } else {
            json_result(&response)
        }
    }

    /// Find functions similar to a specified function
    #[tool(
        description = "Find functions similar to a specified function. Returns functions in the same category, functions with similar signatures (same input/output types), and functions with related concepts based on description keywords. Useful for discovering alternative approaches."
    )]
    async fn similar(
        &self,
        Parameters(params): Parameters<SimilarParams>,
    ) -> Result<CallToolResult, McpError> {
        let reg = registry();

        let target = match reg.get_function(&params.function) {
            Some(f) => f,
            None => {
                return Ok(error_result(format!(
                    "Unknown function '{}'. Use the 'search' tool to find functions.",
                    params.function
                )));
            }
        };

        // Parse target signature
        let target_sig = parse_signature_parts(target.signature);

        // 1. Same category (excluding target)
        let same_category: Vec<FunctionDetail> = reg
            .functions_in_category(target.category)
            .filter(|f| f.name != target.name)
            .take(10)
            .map(FunctionDetail::from)
            .collect();

        // 2. Similar signature (different category, same input->output pattern)
        let similar_signature: Vec<FunctionDetail> = reg
            .functions()
            .filter(|f| {
                f.name != target.name
                    && f.category != target.category
                    && signatures_similar(&parse_signature_parts(f.signature), &target_sig)
            })
            .take(10)
            .map(FunctionDetail::from)
            .collect();

        // 3. Related concepts (shared keywords in description)
        let keywords = extract_description_keywords(target.description);
        let mut related: Vec<(FunctionDetail, usize)> = reg
            .functions()
            .filter(|f| f.name != target.name && f.category != target.category)
            .filter_map(|f| {
                let score = keywords
                    .iter()
                    .filter(|kw| f.description.to_lowercase().contains(&kw.to_lowercase()))
                    .count();
                if score > 0 {
                    Some((FunctionDetail::from(f), score))
                } else {
                    None
                }
            })
            .collect();

        related.sort_by(|a, b| b.1.cmp(&a.1));
        let related_concepts: Vec<FunctionDetail> =
            related.into_iter().take(8).map(|(f, _)| f).collect();

        json_result(&SimilarResult {
            same_category,
            similar_signature,
            related_concepts,
        })
    }

    /// Get statistics about JSON data structure
    #[tool(
        description = "Analyze JSON data and return statistics including type, size, depth, field analysis for arrays of objects, and type distribution. Useful for understanding data structure before writing queries."
    )]
    async fn stats(
        &self,
        Parameters(params): Parameters<StatsParams>,
    ) -> Result<CallToolResult, McpError> {
        let value: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {}", e), None))?;

        let size_bytes = serde_json::to_string(&value).map(|s| s.len()).unwrap_or(0);

        let mut result = StatsResult {
            root_type: get_json_type_name(&value),
            size_bytes,
            size_human: format_size_human(size_bytes),
            depth: calculate_json_depth(&value),
            length: None,
            key_count: None,
            fields: None,
            type_distribution: None,
        };

        match &value {
            Value::Array(arr) => {
                result.length = Some(arr.len());

                // Type distribution
                let mut type_counts = std::collections::HashMap::new();
                for item in arr {
                    let type_name = get_json_type_name(item);
                    *type_counts.entry(type_name).or_insert(0usize) += 1;
                }
                result.type_distribution = Some(type_counts.clone());

                // If all objects, analyze fields
                if type_counts.len() == 1 && type_counts.contains_key("object") {
                    result.fields = Some(analyze_array_fields(arr));
                }
            }
            Value::Object(obj) => {
                result.key_count = Some(obj.len());
            }
            _ => {}
        }

        json_result(&result)
    }

    /// List all paths in JSON data
    #[tool(
        description = "Extract all paths from JSON data in dot notation (e.g., 'users.0.name'). Optionally includes type information and values. Essential for understanding complex JSON structure before writing JMESPath queries."
    )]
    async fn paths(
        &self,
        Parameters(params): Parameters<PathsParams>,
    ) -> Result<CallToolResult, McpError> {
        let value: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {}", e), None))?;

        let mut paths: Vec<PathInfo> = Vec::new();
        collect_all_paths(
            &value,
            String::new(),
            &mut paths,
            params.include_types,
            params.include_values,
        );

        json_result(&paths)
    }

    // =========================================================================
    // Discovery tools - MCP tool discovery protocol
    // =========================================================================

    /// Get the discovery schema for registering MCP server capabilities
    #[tool(
        description = "Get the JSON schema for the MCP discovery protocol. MCP servers can use this schema to register their tools with jpx for cross-server discovery and search."
    )]
    async fn get_discovery_schema(
        &self,
        Parameters(_params): Parameters<GetDiscoverySchemaParams>,
    ) -> Result<CallToolResult, McpError> {
        let schema = DiscoveryRegistry::get_schema();
        json_result(&schema)
    }

    /// Register an MCP server's tools for discovery
    #[tool(
        description = "Register an MCP server's capabilities for discovery. Accepts a discovery spec with server info and tool definitions. Tools are indexed for full-text search across name, description, tags, and parameters."
    )]
    async fn register_discovery(
        &self,
        Parameters(params): Parameters<RegisterDiscoveryParams>,
    ) -> Result<CallToolResult, McpError> {
        // Register with the global registry
        let result = {
            let mut registry = discovery_registry()
                .write()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.register(params.spec, params.replace)
        };

        json_result(&result)
    }

    /// Query tools across all registered MCP servers
    #[tool(
        description = "Search for tools across all registered MCP servers. Uses BM25 full-text search to find relevant tools by name, description, tags, category, or parameters. Returns ranked results with match scores."
    )]
    async fn query_tools(
        &self,
        Parameters(params): Parameters<QueryToolsParams>,
    ) -> Result<CallToolResult, McpError> {
        let results = {
            let registry = discovery_registry()
                .read()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.query(&params.query, params.top_k)
        };

        json_result(&results)
    }

    /// Find tools similar to a given tool
    #[tool(
        description = "Find tools similar to a specified tool based on shared terms and concepts. Uses the tool's indexed content to find related tools across all registered servers."
    )]
    async fn similar_tools(
        &self,
        Parameters(params): Parameters<SimilarToolsParams>,
    ) -> Result<CallToolResult, McpError> {
        let results = {
            let registry = discovery_registry()
                .read()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.similar(&params.tool_id, params.top_k)
        };

        json_result(&results)
    }

    /// List all registered MCP servers
    #[tool(
        description = "List all MCP servers that have registered their tools for discovery. Returns server names, versions, descriptions, and tool counts."
    )]
    async fn list_discovery_servers(&self) -> Result<CallToolResult, McpError> {
        let servers = {
            let registry = discovery_registry()
                .read()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.list_servers()
        };

        json_result(&servers)
    }

    /// List all tool categories across registered servers
    #[tool(
        description = "List all tool categories from registered MCP servers. Returns category names with tool counts and which servers provide tools in each category."
    )]
    async fn list_discovery_categories(&self) -> Result<CallToolResult, McpError> {
        let categories = {
            let registry = discovery_registry()
                .read()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.list_categories()
        };

        json_result(&categories)
    }

    /// Get discovery index statistics
    #[tool(
        description = "Get statistics about the discovery index including document count, term count, average document length, and top indexed terms. Useful for understanding what's been indexed."
    )]
    async fn inspect_discovery_index(&self) -> Result<CallToolResult, McpError> {
        let stats = {
            let registry = discovery_registry()
                .read()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.index_stats()
        };

        match stats {
            Some(s) => json_result(&s),
            None => Ok(text_result(
                "No tools have been indexed yet. Use register_discovery to add MCP server tools.",
            )),
        }
    }

    /// Unregister an MCP server from discovery
    #[tool(
        description = "Remove an MCP server's tools from the discovery index. Use this when a server is no longer available or to re-register with updated tools."
    )]
    async fn unregister_discovery(
        &self,
        Parameters(params): Parameters<UnregisterDiscoveryParams>,
    ) -> Result<CallToolResult, McpError> {
        let removed = {
            let mut registry = discovery_registry()
                .write()
                .map_err(|_| McpError::internal_error("Failed to acquire registry lock", None))?;
            registry.unregister(&params.server_name)
        };

        if removed {
            Ok(text_result(format!(
                "Successfully unregistered server '{}'",
                params.server_name
            )))
        } else {
            Ok(error_result(format!(
                "Server '{}' was not registered",
                params.server_name
            )))
        }
    }

    // =========================================================================
    // Query Store Tools
    // =========================================================================

    /// Define a named query for later reuse
    #[tool(
        description = "Store a named JMESPath query for reuse during this session. Useful for building and refining complex queries iteratively. The query is validated before storing."
    )]
    async fn define_query(
        &self,
        Parameters(params): Parameters<DefineQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        // Validate the expression first
        if let Err(e) = self.runtime().compile(&params.expression) {
            return Ok(error_result(format!("Invalid JMESPath expression: {}", e)));
        }

        let query = StoredQuery {
            name: params.name.clone(),
            expression: params.expression,
            description: params.description,
        };

        let store = query_store::query_store();
        let was_replaced = match store.write() {
            Ok(mut s) => s.define(query).is_some(),
            Err(_) => return Ok(error_result("Failed to access query store")),
        };

        if was_replaced {
            Ok(text_result(format!(
                "Query '{}' updated successfully",
                params.name
            )))
        } else {
            Ok(text_result(format!(
                "Query '{}' defined successfully",
                params.name
            )))
        }
    }

    /// List all stored queries
    #[tool(
        description = "List all named queries stored in this session. Shows query names, expressions, and descriptions."
    )]
    async fn list_queries(&self) -> Result<CallToolResult, McpError> {
        let store = query_store::query_store();
        let queries: Vec<StoredQuery> = match store.read() {
            Ok(s) => s.list().into_iter().cloned().collect(),
            Err(_) => return Ok(error_result("Failed to access query store")),
        };

        if queries.is_empty() {
            return Ok(text_result(
                "No queries stored. Use 'define_query' to store a named query.",
            ));
        }

        json_result(&queries)
    }

    /// Get a stored query by name
    #[tool(
        description = "Retrieve a stored query by name. Returns the expression and description if found."
    )]
    async fn get_query(
        &self,
        Parameters(params): Parameters<GetQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = query_store::query_store();
        let query: Option<StoredQuery> = match store.read() {
            Ok(s) => s.get(&params.name).cloned(),
            Err(_) => return Ok(error_result("Failed to access query store")),
        };

        match query {
            Some(q) => json_result(&q),
            None => Ok(error_result(format!(
                "Query '{}' not found. Use 'list_queries' to see available queries.",
                params.name
            ))),
        }
    }

    /// Delete a stored query
    #[tool(description = "Delete a stored query by name. Returns the deleted query if it existed.")]
    async fn delete_query(
        &self,
        Parameters(params): Parameters<DeleteQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let store = query_store::query_store();
        let deleted: Option<StoredQuery> = match store.write() {
            Ok(mut s) => s.delete(&params.name),
            Err(_) => return Ok(error_result("Failed to access query store")),
        };

        match deleted {
            Some(q) => Ok(text_result(format!(
                "Query '{}' deleted (expression was: {})",
                q.name, q.expression
            ))),
            None => Ok(error_result(format!("Query '{}' not found", params.name))),
        }
    }

    /// Run a stored query against input data
    #[tool(
        description = "Execute a stored query by name against JSON input. Combines the convenience of named queries with evaluation."
    )]
    async fn run_query(
        &self,
        Parameters(params): Parameters<RunQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        // Get the stored query
        let store = query_store::query_store();
        let query: Option<StoredQuery> = match store.read() {
            Ok(s) => s.get(&params.name).cloned(),
            Err(_) => return Ok(error_result("Failed to access query store")),
        };

        let query = match query {
            Some(q) => q,
            None => {
                return Ok(error_result(format!(
                    "Query '{}' not found. Use 'list_queries' to see available queries.",
                    params.name
                )));
            }
        };

        // Compile and execute
        let expr = self.runtime().compile(&query.expression).map_err(|e| {
            McpError::internal_error(format!("Stored query has invalid expression: {}", e), None)
        })?;

        let var = jmespath::Variable::from_json(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON input: {}", e), None))?;

        let result = expr
            .search(&var)
            .map_err(|e| McpError::internal_error(format!("Evaluation failed: {}", e), None))?;

        let result_json: Value = serde_json::to_value(&*result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        json_result(&result_json)
    }
}

// =============================================================================
// Helper functions for new tools
// =============================================================================

/// Parse signature into (input_types, output_type)
fn parse_signature_parts(sig: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = sig.split("->").collect();
    if parts.len() != 2 {
        return (vec![], String::new());
    }

    let inputs: Vec<String> = parts[0]
        .split(',')
        .map(|s| normalize_type_name(s.trim()))
        .collect();
    let output = normalize_type_name(parts[1].trim());

    (inputs, output)
}

/// Normalize type names for comparison
fn normalize_type_name(t: &str) -> String {
    let t = t.trim_end_matches('?').trim_end_matches("...");
    match t.to_lowercase().as_str() {
        "number" | "integer" => "number".to_string(),
        "string" | "str" => "string".to_string(),
        "array" | "list" => "array".to_string(),
        "object" | "hash" | "map" => "object".to_string(),
        "boolean" | "bool" => "boolean".to_string(),
        "any" | "expression" | "expref" => "any".to_string(),
        _ => t.to_lowercase(),
    }
}

/// Check if two signatures are similar
fn signatures_similar(a: &(Vec<String>, String), b: &(Vec<String>, String)) -> bool {
    // Must have same output type
    if a.1 != b.1 || a.1.is_empty() {
        return false;
    }
    // Same number of inputs
    if a.0.len() != b.0.len() {
        return false;
    }
    // First input type should match
    if !a.0.is_empty() && !b.0.is_empty() && a.0[0] == b.0[0] {
        return true;
    }
    false
}

/// Extract keywords from description
fn extract_description_keywords(description: &str) -> Vec<String> {
    let stopwords = [
        "a",
        "an",
        "the",
        "to",
        "of",
        "in",
        "for",
        "is",
        "are",
        "and",
        "or",
        "with",
        "from",
        "by",
        "on",
        "at",
        "as",
        "if",
        "be",
        "this",
        "that",
        "it",
        "its",
        "can",
        "will",
        "into",
        "using",
        "returns",
        "return",
        "value",
        "values",
        "given",
        "specified",
    ];

    description
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

/// Get JSON type name
fn get_json_type_name(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else {
                "number".to_string()
            }
        }
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

/// Format size in human-readable form
fn format_size_human(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Calculate JSON nesting depth
fn calculate_json_depth(value: &Value) -> usize {
    match value {
        Value::Array(arr) => 1 + arr.iter().map(calculate_json_depth).max().unwrap_or(0),
        Value::Object(obj) => 1 + obj.values().map(calculate_json_depth).max().unwrap_or(0),
        _ => 1,
    }
}

/// Analyze fields in an array of objects
fn analyze_array_fields(arr: &[Value]) -> Vec<FieldAnalysis> {
    use std::collections::{HashMap, HashSet};

    let mut all_keys: Vec<String> = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

    // Collect all unique keys from first 100 objects
    for item in arr.iter().take(100) {
        if let Value::Object(obj) = item {
            for key in obj.keys() {
                if seen_keys.insert(key.clone()) {
                    all_keys.push(key.clone());
                }
            }
        }
    }

    // Analyze each key
    let mut results: Vec<FieldAnalysis> = Vec::new();

    for key in all_keys.iter().take(20) {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut null_count = 0;
        let mut unique_values: HashSet<String> = HashSet::new();

        for item in arr {
            if let Value::Object(obj) = item {
                match obj.get(key) {
                    Some(Value::Null) | None => {
                        null_count += 1;
                    }
                    Some(v) => {
                        let type_name = get_json_type_name(v);
                        *type_counts.entry(type_name).or_insert(0) += 1;

                        if unique_values.len() < 100 {
                            if let Some(s) = v.as_str() {
                                unique_values.insert(s.to_string());
                            } else if let Some(n) = v.as_i64() {
                                unique_values.insert(n.to_string());
                            } else if let Some(b) = v.as_bool() {
                                unique_values.insert(b.to_string());
                            }
                        }
                    }
                }
            }
        }

        let field_type = type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t.clone())
            .unwrap_or_else(|| "null".to_string());

        results.push(FieldAnalysis {
            name: key.clone(),
            field_type,
            null_count,
            unique_count: if unique_values.len() <= 50 {
                Some(unique_values.len())
            } else {
                None
            },
        });
    }

    results
}

/// Recursively collect all paths from JSON
fn collect_all_paths(
    value: &Value,
    current_path: String,
    paths: &mut Vec<PathInfo>,
    include_types: bool,
    include_values: bool,
) {
    let display_path = if current_path.is_empty() {
        ".".to_string()
    } else {
        current_path.clone()
    };

    match value {
        Value::Object(obj) => {
            paths.push(PathInfo {
                path: display_path,
                path_type: if include_types {
                    Some(format!("object{{{}}}", obj.len()))
                } else {
                    None
                },
                value: None,
            });

            for (key, val) in obj {
                let new_path = if current_path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", current_path, key)
                };
                collect_all_paths(val, new_path, paths, include_types, include_values);
            }
        }
        Value::Array(arr) => {
            paths.push(PathInfo {
                path: display_path,
                path_type: if include_types {
                    Some(format!("array[{}]", arr.len()))
                } else {
                    None
                },
                value: None,
            });

            for (i, val) in arr.iter().enumerate() {
                let new_path = if current_path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", current_path, i)
                };
                collect_all_paths(val, new_path, paths, include_types, include_values);
            }
        }
        _ => {
            let value_preview = if include_values {
                Some(truncate_value(value))
            } else {
                None
            };

            paths.push(PathInfo {
                path: display_path,
                path_type: if include_types {
                    Some(get_json_type_name(value))
                } else {
                    None
                },
                value: value_preview,
            });
        }
    }
}

/// Truncate a JSON value for preview
fn truncate_value(value: &Value) -> Value {
    match value {
        Value::String(s) if s.len() > 50 => Value::String(format!("{}...", &s[..47])),
        _ => value.clone(),
    }
}

/// Recursively collect keys from a JSON value using dot notation
fn collect_keys_recursive(value: &Value, prefix: String) -> Vec<String> {
    let mut keys = Vec::new();

    if let Value::Object(map) = value {
        for (key, val) in map {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            keys.push(full_key.clone());

            // Recurse into nested objects
            if val.is_object() {
                keys.extend(collect_keys_recursive(val, full_key));
            }
        }
    }

    keys
}

#[tool_handler]
impl ServerHandler for JpxMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "JMESPath query tool with 320+ extended functions. \
                 \n\nDISCOVERY: Use 'search' to find functions by keyword (fuzzy matching), 'similar' to find related functions, \
                 'functions' to list all functions (optionally by category), 'describe' for function details, 'categories' to list categories. \
                 \n\nDATA ANALYSIS: Use 'stats' to analyze JSON structure before querying, 'paths' to list all paths in dot notation, \
                 'keys' to extract object keys (optionally recursive). \
                 \n\nQUERYING: Use 'evaluate' to run JMESPath queries, 'evaluate_file' to query JSON files directly, \
                 'batch_evaluate' for multiple expressions against the same input, 'validate' to check expression syntax. \
                 \n\nJSON UTILITIES: Use 'format' to pretty-print JSON, 'diff' to generate RFC 6902 JSON Patches, \
                 'patch' to apply RFC 6902 patches, 'merge' to apply RFC 7396 JSON Merge Patches."
                    .to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category() {
        assert_eq!(parse_category("string"), Some(Category::String));
        assert_eq!(parse_category("STRING"), Some(Category::String));
        assert_eq!(parse_category("Math"), Some(Category::Math));
        assert_eq!(parse_category("unknown"), None);
    }

    #[test]
    fn test_parse_category_all_categories() {
        // Test all valid categories
        let categories = [
            "standard",
            "string",
            "array",
            "object",
            "math",
            "type",
            "utility",
            "validation",
            "path",
            "expression",
            "text",
            "hash",
            "encoding",
            "regex",
            "url",
            "uuid",
            "rand",
            "datetime",
            "fuzzy",
            "phonetic",
            "geo",
            "semver",
            "network",
            "ids",
            "duration",
            "color",
            "computing",
            "multimatch",
            "jsonpatch",
            "format",
        ];
        for cat in categories {
            assert!(
                parse_category(cat).is_some(),
                "Category '{}' should parse",
                cat
            );
        }
    }

    #[test]
    fn test_function_detail_from() {
        let reg = registry();
        let info = reg.get_function("upper").unwrap();
        let detail = FunctionDetail::from(info);
        assert_eq!(detail.name, "upper");
        assert!(!detail.description.is_empty());
    }

    #[test]
    fn test_function_detail_with_aliases() {
        let reg = registry();
        // find a function with aliases if one exists
        if let Some(info) = reg.get_function("every") {
            let detail = FunctionDetail::from(info);
            assert_eq!(detail.name, "every");
        }
    }

    #[test]
    fn test_registry_initialization() {
        let reg = registry();
        // Should have many functions
        assert!(reg.functions().count() > 100);
    }

    #[test]
    fn test_runtime_full_initialization() {
        let rt = runtime_full();
        // Should be able to compile a basic expression
        assert!(rt.compile("@").is_ok());
        // Extension functions should work
        assert!(rt.compile("upper(@)").is_ok());
    }

    #[test]
    fn test_runtime_strict_initialization() {
        let rt = runtime_strict();
        // Should be able to compile a basic expression
        assert!(rt.compile("@").is_ok());
        // Standard functions should work
        assert!(rt.compile("length(@)").is_ok());
    }

    #[test]
    fn test_text_result() {
        let result = text_result("hello");
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_json_result() {
        let value = serde_json::json!({"key": "value"});
        let result = json_result(&value).unwrap();
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn test_error_result() {
        let result = error_result("something went wrong");
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn test_validation_result_serialization() {
        let valid = ValidationResult {
            valid: true,
            error: None,
        };
        let json = serde_json::to_string(&valid).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(!json.contains("error")); // None should be skipped

        let invalid = ValidationResult {
            valid: false,
            error: Some("parse error".to_string()),
        };
        let json = serde_json::to_string(&invalid).unwrap();
        assert!(json.contains("\"valid\":false"));
        assert!(json.contains("parse error"));
    }

    #[test]
    fn test_jpx_mcp_new() {
        let mcp = JpxMcp::new(false);
        // Should initialize without panic
        drop(mcp);
    }

    #[test]
    fn test_jpx_mcp_default() {
        let mcp = JpxMcp::default();
        drop(mcp);
    }

    #[test]
    fn test_batch_expression_result_success() {
        let result = BatchExpressionResult {
            expression: "name".to_string(),
            result: Some(serde_json::json!("alice")),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"expression\":\"name\""));
        assert!(json.contains("\"result\":\"alice\""));
        assert!(!json.contains("error")); // None should be skipped
    }

    #[test]
    fn test_batch_expression_result_error() {
        let result = BatchExpressionResult {
            expression: "invalid[".to_string(),
            result: None,
            error: Some("Parse error".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"expression\":\"invalid[\""));
        assert!(json.contains("\"error\":\"Parse error\""));
        assert!(!json.contains("\"result\"")); // None should be skipped
    }

    #[test]
    fn test_batch_evaluate_result_serialization() {
        let result = BatchEvaluateResult {
            results: vec![
                BatchExpressionResult {
                    expression: "name".to_string(),
                    result: Some(serde_json::json!("alice")),
                    error: None,
                },
                BatchExpressionResult {
                    expression: "age".to_string(),
                    result: Some(serde_json::json!(30)),
                    error: None,
                },
            ],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"results\""));
        assert!(json.contains("\"alice\""));
        assert!(json.contains("30"));
    }

    #[test]
    fn test_batch_evaluate_result_mixed() {
        let result = BatchEvaluateResult {
            results: vec![
                BatchExpressionResult {
                    expression: "name".to_string(),
                    result: Some(serde_json::json!("alice")),
                    error: None,
                },
                BatchExpressionResult {
                    expression: "invalid[".to_string(),
                    result: None,
                    error: Some("Compile error".to_string()),
                },
            ],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"alice\""));
        assert!(json.contains("Compile error"));
    }

    // =========================================================================
    // Format tool tests
    // =========================================================================

    #[test]
    fn test_format_params_default_indent() {
        let params: FormatParams = serde_json::from_str(r#"{"input": "{}"}"#).unwrap();
        assert_eq!(params.indent, 2); // default
    }

    #[test]
    fn test_format_params_custom_indent() {
        let params: FormatParams = serde_json::from_str(r#"{"input": "{}", "indent": 4}"#).unwrap();
        assert_eq!(params.indent, 4);
    }

    #[test]
    fn test_format_params_compact() {
        let params: FormatParams = serde_json::from_str(r#"{"input": "{}", "indent": 0}"#).unwrap();
        assert_eq!(params.indent, 0);
    }

    #[tokio::test]
    async fn test_format_pretty_print() {
        let mcp = JpxMcp::new(false);
        let params = FormatParams {
            input: r#"{"name":"alice","age":30}"#.to_string(),
            indent: 2,
        };
        let result = mcp.format(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("  \"name\": \"alice\""));
                assert!(text_content.text.contains("  \"age\": 30"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_format_compact() {
        let mcp = JpxMcp::new(false);
        let params = FormatParams {
            input: r#"{ "name" : "alice" , "age" : 30 }"#.to_string(),
            indent: 0,
        };
        let result = mcp.format(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text, r#"{"age":30,"name":"alice"}"#);
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_format_invalid_json() {
        let mcp = JpxMcp::new(false);
        let params = FormatParams {
            input: r#"{"invalid": }"#.to_string(),
            indent: 2,
        };
        let result = mcp.format(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid JSON"));
    }

    // =========================================================================
    // Diff tool tests (RFC 6902)
    // =========================================================================

    #[tokio::test]
    async fn test_diff_add_field() {
        let mcp = JpxMcp::new(false);
        let params = DiffParams {
            source: r#"{"name":"alice"}"#.to_string(),
            target: r#"{"name":"alice","age":30}"#.to_string(),
        };
        let result = mcp.diff(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("add"));
                assert!(text_content.text.contains("/age"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_diff_remove_field() {
        let mcp = JpxMcp::new(false);
        let params = DiffParams {
            source: r#"{"name":"alice","age":30}"#.to_string(),
            target: r#"{"name":"alice"}"#.to_string(),
        };
        let result = mcp.diff(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("remove"));
                assert!(text_content.text.contains("/age"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_diff_replace_value() {
        let mcp = JpxMcp::new(false);
        let params = DiffParams {
            source: r#"{"name":"alice"}"#.to_string(),
            target: r#"{"name":"bob"}"#.to_string(),
        };
        let result = mcp.diff(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("replace"));
                assert!(text_content.text.contains("/name"));
                assert!(text_content.text.contains("bob"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_diff_no_changes() {
        let mcp = JpxMcp::new(false);
        let params = DiffParams {
            source: r#"{"name":"alice"}"#.to_string(),
            target: r#"{"name":"alice"}"#.to_string(),
        };
        let result = mcp.diff(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text.trim(), "[]");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    // =========================================================================
    // Patch tool tests (RFC 6902)
    // =========================================================================

    #[tokio::test]
    async fn test_patch_add() {
        let mcp = JpxMcp::new(false);
        let params = PatchParams {
            input: r#"{"name":"alice"}"#.to_string(),
            patch: r#"[{"op":"add","path":"/age","value":30}]"#.to_string(),
        };
        let result = mcp.patch(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"alice\""));
                assert!(text_content.text.contains("\"age\": 30"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_patch_remove() {
        let mcp = JpxMcp::new(false);
        let params = PatchParams {
            input: r#"{"name":"alice","age":30}"#.to_string(),
            patch: r#"[{"op":"remove","path":"/age"}]"#.to_string(),
        };
        let result = mcp.patch(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"alice\""));
                assert!(!text_content.text.contains("age"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_patch_replace() {
        let mcp = JpxMcp::new(false);
        let params = PatchParams {
            input: r#"{"name":"alice"}"#.to_string(),
            patch: r#"[{"op":"replace","path":"/name","value":"bob"}]"#.to_string(),
        };
        let result = mcp.patch(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"bob\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_patch_invalid_path() {
        let mcp = JpxMcp::new(false);
        let params = PatchParams {
            input: r#"{"name":"alice"}"#.to_string(),
            patch: r#"[{"op":"remove","path":"/nonexistent"}]"#.to_string(),
        };
        let result = mcp.patch(Parameters(params)).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Merge tool tests (RFC 7396)
    // =========================================================================

    #[tokio::test]
    async fn test_merge_add_field() {
        let mcp = JpxMcp::new(false);
        let params = MergeParams {
            input: r#"{"name":"alice"}"#.to_string(),
            patch: r#"{"age":30}"#.to_string(),
        };
        let result = mcp.merge(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"alice\""));
                assert!(text_content.text.contains("\"age\": 30"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_merge_remove_with_null() {
        let mcp = JpxMcp::new(false);
        let params = MergeParams {
            input: r#"{"name":"alice","age":30}"#.to_string(),
            patch: r#"{"age":null}"#.to_string(),
        };
        let result = mcp.merge(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"alice\""));
                assert!(!text_content.text.contains("\"age\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_merge_replace_value() {
        let mcp = JpxMcp::new(false);
        let params = MergeParams {
            input: r#"{"name":"alice","age":30}"#.to_string(),
            patch: r#"{"age":31}"#.to_string(),
        };
        let result = mcp.merge(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"age\": 31"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_merge_nested() {
        let mcp = JpxMcp::new(false);
        let params = MergeParams {
            input: r#"{"user":{"name":"alice","age":30}}"#.to_string(),
            patch: r#"{"user":{"city":"NYC"}}"#.to_string(),
        };
        let result = mcp.merge(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"alice\""));
                assert!(text_content.text.contains("\"age\": 30"));
                assert!(text_content.text.contains("\"city\": \"NYC\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    // =========================================================================
    // Roundtrip test: diff -> patch
    // =========================================================================

    #[tokio::test]
    async fn test_diff_patch_roundtrip() {
        let mcp = JpxMcp::new(false);
        let source = r#"{"name":"alice","age":30}"#;
        let target = r#"{"name":"bob","age":31,"city":"NYC"}"#;

        // Generate diff
        let diff_params = DiffParams {
            source: source.to_string(),
            target: target.to_string(),
        };
        let diff_result = mcp.diff(Parameters(diff_params)).await.unwrap();

        // Extract patch from diff result
        let patch_str = if let Some(content) = diff_result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                text_content.text.clone()
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        };

        // Apply patch to source
        let patch_params = PatchParams {
            input: source.to_string(),
            patch: patch_str,
        };
        let patch_result = mcp.patch(Parameters(patch_params)).await.unwrap();

        // Verify result matches target
        if let Some(content) = patch_result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\": \"bob\""));
                assert!(text_content.text.contains("\"age\": 31"));
                assert!(text_content.text.contains("\"city\": \"NYC\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    // =========================================================================
    // Keys tool tests
    // =========================================================================

    #[test]
    fn test_keys_params_default_recursive() {
        let params: KeysParams = serde_json::from_str(r#"{"input": "{}"}"#).unwrap();
        assert!(!params.recursive); // default is false
    }

    #[test]
    fn test_keys_params_recursive_true() {
        let params: KeysParams =
            serde_json::from_str(r#"{"input": "{}", "recursive": true}"#).unwrap();
        assert!(params.recursive);
    }

    #[tokio::test]
    async fn test_keys_top_level() {
        let mcp = JpxMcp::new(false);
        let params = KeysParams {
            input: r#"{"name":"alice","age":30,"city":"NYC"}"#.to_string(),
            recursive: false,
        };
        let result = mcp.keys(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"name\""));
                assert!(text_content.text.contains("\"age\""));
                assert!(text_content.text.contains("\"city\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_keys_recursive() {
        let mcp = JpxMcp::new(false);
        let params = KeysParams {
            input: r#"{"user":{"name":"alice","profile":{"age":30}}}"#.to_string(),
            recursive: true,
        };
        let result = mcp.keys(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"user\""));
                assert!(text_content.text.contains("\"user.name\""));
                assert!(text_content.text.contains("\"user.profile\""));
                assert!(text_content.text.contains("\"user.profile.age\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_keys_nested_non_recursive() {
        let mcp = JpxMcp::new(false);
        let params = KeysParams {
            input: r#"{"user":{"name":"alice","profile":{"age":30}}}"#.to_string(),
            recursive: false,
        };
        let result = mcp.keys(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                // Should only have top-level key
                assert!(text_content.text.contains("\"user\""));
                assert!(!text_content.text.contains("\"user.name\""));
                assert!(!text_content.text.contains("\"name\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_keys_empty_object() {
        let mcp = JpxMcp::new(false);
        let params = KeysParams {
            input: r#"{}"#.to_string(),
            recursive: false,
        };
        let result = mcp.keys(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text.trim(), "[]");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_keys_array_input() {
        let mcp = JpxMcp::new(false);
        let params = KeysParams {
            input: r#"[1, 2, 3]"#.to_string(),
            recursive: false,
        };
        let result = mcp.keys(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        // Arrays don't have keys
        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text.trim(), "[]");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_keys_invalid_json() {
        let mcp = JpxMcp::new(false);
        let params = KeysParams {
            input: r#"{"invalid": }"#.to_string(),
            recursive: false,
        };
        let result = mcp.keys(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid JSON"));
    }

    #[test]
    fn test_collect_keys_recursive_helper() {
        let value: Value = serde_json::from_str(r#"{"a":{"b":{"c":1},"d":2},"e":3}"#).unwrap();
        let keys = collect_keys_recursive(&value, String::new());

        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"a.b".to_string()));
        assert!(keys.contains(&"a.b.c".to_string()));
        assert!(keys.contains(&"a.d".to_string()));
        assert!(keys.contains(&"e".to_string()));
        assert_eq!(keys.len(), 5);
    }

    // =========================================================================
    // EvaluateFile tool tests
    // =========================================================================

    #[tokio::test]
    async fn test_evaluate_file_success() {
        use std::io::Write;

        let mcp = JpxMcp::new(false);

        // Create a temp file with JSON content
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"users": [{{"name": "alice"}}, {{"name": "bob"}}]}}"#
        )
        .unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let params = EvaluateFileParams {
            file_path,
            expression: "users[*].name".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"alice\""));
                assert!(text_content.text.contains("\"bob\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_file_relative_path_rejected() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateFileParams {
            file_path: "relative/path/file.json".to_string(),
            expression: "@".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("absolute"));
    }

    #[tokio::test]
    async fn test_evaluate_file_not_found() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateFileParams {
            file_path: "/nonexistent/path/to/file.json".to_string(),
            expression: "@".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Cannot resolve path"));
    }

    #[tokio::test]
    async fn test_evaluate_file_invalid_json() {
        use std::io::Write;

        let mcp = JpxMcp::new(false);

        // Create a temp file with invalid JSON
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{invalid json}}"#).unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let params = EvaluateFileParams {
            file_path,
            expression: "@".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid JSON"));
    }

    #[tokio::test]
    async fn test_evaluate_file_invalid_expression() {
        use std::io::Write;

        let mcp = JpxMcp::new(false);

        // Create a temp file with valid JSON
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"key": "value"}}"#).unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let params = EvaluateFileParams {
            file_path,
            expression: "invalid[".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid expression"));
    }

    #[tokio::test]
    async fn test_evaluate_file_directory_rejected() {
        let mcp = JpxMcp::new(false);

        // Use a known directory
        let params = EvaluateFileParams {
            file_path: "/tmp".to_string(),
            expression: "@".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Not a file"));
    }

    // =========================================================================
    // Strict mode tests
    // =========================================================================

    #[test]
    fn test_strict_mode_rejects_extension_functions() {
        let rt = runtime_strict();
        // Extension function 'upper' should not be available in strict mode
        // Note: jmespath compiles lazily - unknown functions fail at search time, not compile time
        let expr = rt
            .compile("upper('hello')")
            .expect("compile should succeed");
        let var = jmespath::Variable::from_json("\"test\"").unwrap();
        let result = expr.search(&var);
        assert!(
            result.is_err(),
            "Extension function 'upper' should fail at runtime in strict mode"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("undefined function"),
            "Error should mention undefined function, got: {}",
            err
        );
    }

    #[test]
    fn test_full_mode_accepts_extension_functions() {
        let rt = runtime_full();
        // Extension function 'upper' should work in full mode
        let expr = rt
            .compile("upper('hello')")
            .expect("compile should succeed");
        let var = jmespath::Variable::from_json("\"test\"").unwrap();
        let result = expr.search(&var);
        assert!(
            result.is_ok(),
            "Extension function 'upper' should work in full mode"
        );
        // Should return "HELLO"
        let value = result.unwrap();
        assert_eq!(value.as_string().unwrap(), "HELLO");
    }

    #[test]
    fn test_strict_mode_accepts_standard_functions() {
        let rt = runtime_strict();
        let var = jmespath::Variable::from_json(r#"[1, 2, 3]"#).unwrap();

        // Standard functions should compile and execute
        let expr = rt.compile("length(@)").expect("length should compile");
        let result = expr.search(&var);
        assert!(result.is_ok(), "length should execute in strict mode");

        let obj_var = jmespath::Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let expr = rt.compile("keys(@)").expect("keys should compile");
        let result = expr.search(&obj_var);
        assert!(result.is_ok(), "keys should execute in strict mode");
    }

    #[test]
    fn test_jpx_mcp_strict_mode_flag() {
        let mcp_strict = JpxMcp::new(true);
        assert!(mcp_strict.strict);

        let mcp_full = JpxMcp::new(false);
        assert!(!mcp_full.strict);
    }

    #[tokio::test]
    async fn test_evaluate_strict_mode_rejects_extensions() {
        let mcp = JpxMcp::new(true);
        let params = EvaluateParams {
            input: r#""hello""#.to_string(),
            expression: "upper(@)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await;
        assert!(
            result.is_err(),
            "Extension function should fail in strict mode"
        );
        let err = result.unwrap_err();
        // jmespath returns undefined function error at runtime (search time), wrapped as "Evaluation failed"
        assert!(
            err.message.contains("Evaluation failed") && err.message.contains("undefined function"),
            "Expected undefined function error, got: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn test_evaluate_full_mode_accepts_extensions() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#""hello""#.to_string(),
            expression: "upper(@)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("HELLO"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_strict_mode_allows_standard() {
        let mcp = JpxMcp::new(true);
        let params = EvaluateParams {
            input: r#"[1, 2, 3]"#.to_string(),
            expression: "length(@)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("3"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_batch_evaluate_strict_mode() {
        let mcp = JpxMcp::new(true);
        let params = BatchEvaluateParams {
            input: r#"{"items": [1, 2, 3]}"#.to_string(),
            expressions: vec![
                "length(items)".to_string(),  // standard - should work
                "upper('hello')".to_string(), // extension - should fail
            ],
        };
        let result = mcp.batch_evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let batch_result: BatchEvaluateResult =
                    serde_json::from_str(&text_content.text).unwrap();

                // First expression should succeed
                assert!(batch_result.results[0].error.is_none());
                assert_eq!(batch_result.results[0].result, Some(serde_json::json!(3)));

                // Second expression should fail (extension function not available)
                assert!(batch_result.results[1].error.is_some());
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_validate_syntax_check() {
        // Note: validate only does syntax/compile-time checking.
        // The jmespath crate allows unknown functions to compile - they fail at search time.
        // So validate reports syntax validity, not function availability.
        let mcp = JpxMcp::new(true);

        // Valid syntax (function availability checked at runtime, not compile time)
        let params = ValidateParams {
            expression: "upper(@)".to_string(),
        };
        let result = mcp.validate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));
        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let validation: ValidationResult =
                    serde_json::from_str(&text_content.text).unwrap();
                // Syntactically valid (function check happens at runtime)
                assert!(validation.valid, "Expression should be syntactically valid");
            } else {
                panic!("Expected text content");
            }
        }

        // Invalid syntax should fail
        let params = ValidateParams {
            expression: "invalid[".to_string(),
        };
        let result = mcp.validate(Parameters(params)).await.unwrap();
        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let validation: ValidationResult =
                    serde_json::from_str(&text_content.text).unwrap();
                assert!(!validation.valid, "Invalid syntax should fail validation");
                assert!(validation.error.is_some());
            } else {
                panic!("Expected text content");
            }
        }
    }

    #[tokio::test]
    async fn test_validate_standard_functions() {
        let mcp = JpxMcp::new(true);
        let params = ValidateParams {
            expression: "length(@)".to_string(),
        };
        let result = mcp.validate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let validation: ValidationResult =
                    serde_json::from_str(&text_content.text).unwrap();
                assert!(validation.valid, "Standard function should be valid");
                assert!(validation.error.is_none());
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_file_strict_mode() {
        use std::io::Write;

        let mcp = JpxMcp::new(true);

        // Create a temp file with JSON content
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"name": "alice"}}"#).unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();

        // Extension function should fail in strict mode
        let params = EvaluateFileParams {
            file_path: file_path.clone(),
            expression: "upper(name)".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await;
        assert!(
            result.is_err(),
            "Extension function should fail in strict mode"
        );

        // Standard expression should work
        let params = EvaluateFileParams {
            file_path,
            expression: "name".to_string(),
        };
        let result = mcp.evaluate_file(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));
    }

    #[tokio::test]
    async fn test_json_utility_tools_unaffected_by_strict_mode() {
        // JSON utility tools (format, diff, patch, merge, keys) should work
        // identically in both strict and full mode since they don't use JMESPath
        let mcp_strict = JpxMcp::new(true);

        // format should work
        let format_params = FormatParams {
            input: r#"{"a":1}"#.to_string(),
            indent: 2,
        };
        let result = mcp_strict.format(Parameters(format_params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        // diff should work
        let diff_params = DiffParams {
            source: r#"{"a":1}"#.to_string(),
            target: r#"{"a":2}"#.to_string(),
        };
        let result = mcp_strict.diff(Parameters(diff_params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        // patch should work
        let patch_params = PatchParams {
            input: r#"{"a":1}"#.to_string(),
            patch: r#"[{"op":"replace","path":"/a","value":2}]"#.to_string(),
        };
        let result = mcp_strict.patch(Parameters(patch_params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        // merge should work
        let merge_params = MergeParams {
            input: r#"{"a":1}"#.to_string(),
            patch: r#"{"b":2}"#.to_string(),
        };
        let result = mcp_strict.merge(Parameters(merge_params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        // keys should work
        let keys_params = KeysParams {
            input: r#"{"a":1,"b":2}"#.to_string(),
            recursive: false,
        };
        let result = mcp_strict.keys(Parameters(keys_params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));
    }

    // =========================================================================
    // Path function tests (get_path, has_path, set_path, delete_path)
    // =========================================================================

    #[tokio::test]
    async fn test_evaluate_get_path_dot_notation() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"a": {"b": {"c": 42}}}"#.to_string(),
            expression: "get_path(@, `\"a.b.c\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text.trim(), "42");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_get_path_with_default() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"a": 1}"#.to_string(),
            expression: "get_path(@, `\"a.b.c\"`, `\"missing\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("missing"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_get_path_array_index() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#.to_string(),
            expression: "get_path(@, `\"users.0.name\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("alice"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_has_path_exists() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"a": {"b": 1}}"#.to_string(),
            expression: "has_path(@, `\"a.b\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text.trim(), "true");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_has_path_missing() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"a": {"b": 1}}"#.to_string(),
            expression: "has_path(@, `\"a.c\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert_eq!(text_content.text.trim(), "false");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_set_path_dot_notation() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"a": {}}"#.to_string(),
            expression: "set_path(@, `\"a.b\"`, `99`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"b\": 99"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_set_path_creates_nested() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{}"#.to_string(),
            expression: "set_path(@, `\"a.b.c\"`, `\"deep\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("\"c\": \"deep\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_evaluate_delete_path_dot_notation() {
        let mcp = JpxMcp::new(false);
        let params = EvaluateParams {
            input: r#"{"a": {"b": 1, "c": 2}}"#.to_string(),
            expression: "delete_path(@, `\"a.b\"`)".to_string(),
        };
        let result = mcp.evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(!text_content.text.contains("\"b\":"));
                assert!(text_content.text.contains("\"c\": 2"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_batch_evaluate_path_functions() {
        let mcp = JpxMcp::new(false);
        let params = BatchEvaluateParams {
            input: r#"{"user": {"name": "alice", "age": 30}}"#.to_string(),
            expressions: vec![
                "get_path(@, `\"user.name\"`)".to_string(),
                "has_path(@, `\"user.email\"`)".to_string(),
                "get_path(@, `\"user.email\"`, `\"none\"`)".to_string(),
            ],
        };
        let result = mcp.batch_evaluate(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let batch_result: BatchEvaluateResult =
                    serde_json::from_str(&text_content.text).unwrap();

                // get_path returns "alice"
                assert_eq!(
                    batch_result.results[0].result,
                    Some(serde_json::json!("alice"))
                );
                // has_path returns false (email doesn't exist)
                assert_eq!(
                    batch_result.results[1].result,
                    Some(serde_json::json!(false))
                );
                // get_path with default returns "none"
                assert_eq!(
                    batch_result.results[2].result,
                    Some(serde_json::json!("none"))
                );
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    // =========================================================================
    // Search tool tests
    // =========================================================================

    #[tokio::test]
    async fn test_search_exact_match() {
        let mcp = JpxMcp::new(false);
        let params = SearchParams {
            query: "upper".to_string(),
            limit: 10,
        };
        let result = mcp.search(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let response: SearchResponse = serde_json::from_str(&text_content.text).unwrap();
                assert!(!response.results.is_empty());
                // First result should be exact match
                assert_eq!(response.results[0].function.name, "upper");
                assert_eq!(response.results[0].match_type, "exact_name");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_search_category_match() {
        let mcp = JpxMcp::new(false);
        let params = SearchParams {
            query: "hash".to_string(),
            limit: 20,
        };
        let result = mcp.search(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let response: SearchResponse = serde_json::from_str(&text_content.text).unwrap();
                assert!(!response.results.is_empty());
                // Should find hash-related functions (via synonym expansion)
                assert!(
                    response
                        .results
                        .iter()
                        .any(|r| r.function.name.contains("md5")
                            || r.function.name.contains("sha")
                            || r.function.category == "Hash")
                );
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let mcp = JpxMcp::new(false);
        let params = SearchParams {
            query: "xyznonexistent123".to_string(),
            limit: 10,
        };
        let result = mcp.search(Parameters(params)).await.unwrap();
        // Should return error result (not Err)
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_search_synonym_expansion() {
        let mcp = JpxMcp::new(false);
        let params = SearchParams {
            query: "aggregate".to_string(),
            limit: 20,
        };
        let result = mcp.search(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let response: SearchResponse = serde_json::from_str(&text_content.text).unwrap();
                // Should find group_by via synonym expansion
                assert!(
                    response
                        .results
                        .iter()
                        .any(|r| r.function.name == "group_by"),
                    "Expected to find group_by via synonym expansion"
                );
                // expanded_terms should contain the synonym targets
                assert!(
                    response.expanded_terms.contains(&"group_by".to_string()),
                    "Expected expanded_terms to contain group_by"
                );
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_search_fuzzy_suggestions() {
        let mcp = JpxMcp::new(false);
        let params = SearchParams {
            query: "uper".to_string(), // typo for "upper"
            limit: 10,
        };
        let result = mcp.search(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let response: SearchResponse = serde_json::from_str(&text_content.text).unwrap();
                // Should have suggestions including "upper"
                assert!(
                    response.suggestions.contains(&"upper".to_string()),
                    "Expected suggestions to contain 'upper' for typo 'uper'"
                );
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    // =========================================================================
    // Similar tool tests
    // =========================================================================

    #[tokio::test]
    async fn test_similar_finds_related() {
        let mcp = JpxMcp::new(false);
        let params = SimilarParams {
            function: "upper".to_string(),
        };
        let result = mcp.similar(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let similar: SimilarResult = serde_json::from_str(&text_content.text).unwrap();
                // Should find same-category functions (other string functions)
                assert!(
                    !similar.same_category.is_empty(),
                    "Expected same_category to have functions"
                );
                // All same_category functions should be string functions (same category as upper)
                assert!(
                    similar.same_category.iter().all(|f| f.category == "String"),
                    "All same_category functions should be String category"
                );
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_similar_unknown_function() {
        let mcp = JpxMcp::new(false);
        let params = SimilarParams {
            function: "nonexistent_function".to_string(),
        };
        let result = mcp.similar(Parameters(params)).await.unwrap();
        // Should return error result
        assert_eq!(result.is_error, Some(true));
    }

    // =========================================================================
    // Stats tool tests
    // =========================================================================

    #[tokio::test]
    async fn test_stats_array_of_objects() {
        let mcp = JpxMcp::new(false);
        let params = StatsParams {
            input: r#"[{"name":"alice","age":30},{"name":"bob","age":25}]"#.to_string(),
        };
        let result = mcp.stats(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let stats: StatsResult = serde_json::from_str(&text_content.text).unwrap();
                assert_eq!(stats.root_type, "array");
                assert_eq!(stats.length, Some(2));
                assert!(stats.fields.is_some());
                let fields = stats.fields.unwrap();
                assert!(fields.iter().any(|f| f.name == "name"));
                assert!(fields.iter().any(|f| f.name == "age"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_stats_object() {
        let mcp = JpxMcp::new(false);
        let params = StatsParams {
            input: r#"{"a":1,"b":{"c":2}}"#.to_string(),
        };
        let result = mcp.stats(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let stats: StatsResult = serde_json::from_str(&text_content.text).unwrap();
                assert_eq!(stats.root_type, "object");
                assert_eq!(stats.key_count, Some(2));
                assert_eq!(stats.depth, 3); // root -> b -> c
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    // =========================================================================
    // Paths tool tests
    // =========================================================================

    #[tokio::test]
    async fn test_paths_basic() {
        let mcp = JpxMcp::new(false);
        let params = PathsParams {
            input: r#"{"user":{"name":"alice"}}"#.to_string(),
            include_types: true,
            include_values: false,
        };
        let result = mcp.paths(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let paths: Vec<PathInfo> = serde_json::from_str(&text_content.text).unwrap();
                // Should have paths: ., user, user.name
                assert!(paths.iter().any(|p| p.path == "."));
                assert!(paths.iter().any(|p| p.path == "user"));
                assert!(paths.iter().any(|p| p.path == "user.name"));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_paths_with_array() {
        let mcp = JpxMcp::new(false);
        let params = PathsParams {
            input: r#"{"items":[1,2]}"#.to_string(),
            include_types: true,
            include_values: true,
        };
        let result = mcp.paths(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                let paths: Vec<PathInfo> = serde_json::from_str(&text_content.text).unwrap();
                // Should have array index paths
                assert!(paths.iter().any(|p| p.path == "items[0]"));
                assert!(paths.iter().any(|p| p.path == "items[1]"));
                // Values should be included
                let item0 = paths.iter().find(|p| p.path == "items[0]").unwrap();
                assert_eq!(item0.value, Some(serde_json::json!(1)));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_paths_invalid_json() {
        let mcp = JpxMcp::new(false);
        let params = PathsParams {
            input: r#"{"invalid"}"#.to_string(),
            include_types: true,
            include_values: false,
        };
        let result = mcp.paths(Parameters(params)).await;
        assert!(result.is_err());
    }
}
