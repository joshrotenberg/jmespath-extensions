//! MCP tool implementations for JMESPath
//!
//! This module wraps jpx_engine functionality in MCP tool handlers.

use jpx_engine::{Category, DiscoverySpec, JpxEngine, ServerInfo as DiscoveryServerInfo, ToolSpec};
use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

// =============================================================================
// Parameter structs for MCP tools
// =============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvaluateParams {
    /// JSON input to evaluate the expression against
    pub input: String,
    /// JMESPath expression to evaluate
    pub expression: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FunctionsParams {
    /// Optional category filter (e.g., "String", "Math", "Array", "Datetime")
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DescribeParams {
    /// Function name or alias to describe
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ValidateParams {
    /// JMESPath expression to validate
    pub expression: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatchEvaluateParams {
    /// JSON input to evaluate the expressions against
    pub input: String,
    /// List of JMESPath expressions to evaluate
    pub expressions: Vec<String>,
}

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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffParams {
    /// Source JSON document
    pub source: String,
    /// Target JSON document
    pub target: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatchParams {
    /// JSON document to patch
    pub input: String,
    /// JSON Patch operations array (RFC 6902)
    pub patch: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeParams {
    /// JSON document to merge into
    pub input: String,
    /// JSON Merge Patch document (RFC 7396)
    pub patch: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct KeysParams {
    /// JSON document to extract keys from
    pub input: String,
    /// If true, recursively extract all keys using dot notation (default: false)
    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EvaluateFileParams {
    /// Path to the JSON file to read
    pub file_path: String,
    /// JMESPath expression to evaluate
    pub expression: String,
}

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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SimilarParams {
    /// Function name to find similar functions for
    pub function: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StatsParams {
    /// JSON input to analyze
    pub input: String,
}

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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetDiscoverySchemaParams {
    /// Schema version (optional, defaults to latest)
    #[serde(default)]
    #[allow(dead_code)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterDiscoveryParams {
    /// The discovery spec JSON
    pub spec: Value,
    /// Replace existing registration if server already registered (default: false)
    #[serde(default)]
    pub replace: bool,
}

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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UnregisterDiscoveryParams {
    /// Server name to unregister
    pub server_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SimpleTool {
    /// Tool name (required)
    pub name: String,
    /// Tool description (optional but recommended)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags for categorization and search (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegisterToolsSimpleParams {
    /// Server name (required)
    pub server_name: String,
    /// Server version (optional)
    #[serde(default)]
    pub version: Option<String>,
    /// List of tools to register
    pub tools: Vec<SimpleTool>,
}

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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetQueryParams {
    /// Name of the query to retrieve
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteQueryParams {
    /// Name of the query to delete
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunQueryParams {
    /// Name of the stored query to run
    pub name: String,
    /// JSON input to evaluate the query against
    pub input: String,
}

// =============================================================================
// Helper functions
// =============================================================================

fn text_result(content: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(content)])
}

fn json_result(value: &impl Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("Failed to serialize: {}", e), None))?;
    Ok(text_result(json))
}

fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(message)])
}

// =============================================================================
// MCP Server
// =============================================================================

/// JMESPath MCP server backed by jpx_engine
#[derive(Clone)]
pub struct JpxMcp {
    tool_router: ToolRouter<JpxMcp>,
    engine: Arc<JpxEngine>,
}

impl JpxMcp {
    /// Create a new JpxMcp server
    pub fn new(strict: bool) -> Self {
        let engine = if strict {
            JpxEngine::strict()
        } else {
            JpxEngine::new()
        };

        Self {
            tool_router: Self::tool_router(),
            engine: Arc::new(engine),
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
    #[tool(
        description = "Evaluate a JMESPath expression against JSON input. Returns the result of applying the expression to the input data. Supports 400+ extended functions beyond standard JMESPath."
    )]
    async fn evaluate(
        &self,
        Parameters(params): Parameters<EvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.evaluate_str(&params.expression, &params.input) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "List available JMESPath functions. Optionally filter by category (e.g., 'String', 'Math', 'Array', 'Datetime', 'Hash', 'Encoding', etc.). Returns function names with signatures and descriptions."
    )]
    async fn functions(
        &self,
        Parameters(params): Parameters<FunctionsParams>,
    ) -> Result<CallToolResult, McpError> {
        let functions = self.engine.functions(params.category.as_deref());
        json_result(&functions)
    }

    #[tool(
        description = "Get detailed information about a specific JMESPath function including its signature, description, example usage, and category. Accepts function name or alias."
    )]
    async fn describe(
        &self,
        Parameters(params): Parameters<DescribeParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.describe_function(&params.name) {
            Some(detail) => json_result(&detail),
            None => Ok(error_result(format!(
                "Unknown function '{}'. Use the 'functions' tool to list available functions.",
                params.name
            ))),
        }
    }

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

    #[tool(
        description = "Validate a JMESPath expression without executing it. Returns whether the expression is syntactically valid and any parse errors."
    )]
    async fn validate(
        &self,
        Parameters(params): Parameters<ValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.engine.validate(&params.expression);
        json_result(&result)
    }

    #[tool(
        description = "Evaluate multiple JMESPath expressions against the same JSON input in a single call. Parses the input once and runs all expressions, returning results for each. Useful for extracting multiple values from the same data."
    )]
    async fn batch_evaluate(
        &self,
        Parameters(params): Parameters<BatchEvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        let input: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {}", e), None))?;
        let result = self.engine.batch_evaluate(&params.expressions, &input);
        json_result(&result)
    }

    #[tool(
        description = "Format and validate JSON. Pretty-prints the input with configurable indentation. Use indent=0 for compact output. Returns an error if the input is not valid JSON."
    )]
    async fn format(
        &self,
        Parameters(params): Parameters<FormatParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.format_json(&params.input, params.indent) {
            Ok(formatted) => Ok(text_result(formatted)),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Generate a JSON Patch (RFC 6902) that transforms the source document into the target document. Returns an array of patch operations (add, remove, replace, move, copy, test). See https://datatracker.ietf.org/doc/html/rfc6902"
    )]
    async fn diff(
        &self,
        Parameters(params): Parameters<DiffParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.diff(&params.source, &params.target) {
            Ok(patch) => json_result(&patch),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Apply a JSON Patch (RFC 6902) to a JSON document. The patch is an array of operations (add, remove, replace, move, copy, test). Returns the patched document or an error if the patch cannot be applied. See https://datatracker.ietf.org/doc/html/rfc6902"
    )]
    async fn patch(
        &self,
        Parameters(params): Parameters<PatchParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.patch(&params.input, &params.patch) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Apply a JSON Merge Patch (RFC 7396) to a JSON document. The merge patch is a JSON document that describes changes: values are replaced, null values remove keys, and objects are merged recursively. Simpler than JSON Patch but less expressive. See https://datatracker.ietf.org/doc/html/rfc7396"
    )]
    async fn merge(
        &self,
        Parameters(params): Parameters<MergeParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.merge(&params.input, &params.patch) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Extract keys from a JSON object. By default returns top-level keys only. Set recursive=true to get all nested keys in dot notation (e.g., 'user.profile.age'). Useful for understanding JSON structure before querying."
    )]
    async fn keys(
        &self,
        Parameters(params): Parameters<KeysParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.keys(&params.input, params.recursive) {
            Ok(keys) => json_result(&keys),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Read a JSON file from disk and evaluate a JMESPath expression against it. More efficient than passing large JSON content through the protocol. The file must exist and contain valid JSON."
    )]
    async fn evaluate_file(
        &self,
        Parameters(params): Parameters<EvaluateFileParams>,
    ) -> Result<CallToolResult, McpError> {
        use std::path::Path;

        let path = Path::new(&params.file_path);

        // Security validations
        if !path.is_absolute() {
            return Err(McpError::invalid_params("File path must be absolute", None));
        }

        let canonical_path = path
            .canonicalize()
            .map_err(|e| McpError::invalid_params(format!("Cannot resolve path: {}", e), None))?;

        if !canonical_path.is_file() {
            return Err(McpError::invalid_params(
                format!("Not a file: {}", canonical_path.display()),
                None,
            ));
        }

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

        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| McpError::invalid_params(format!("Cannot read file: {}", e), None))?;

        match self.engine.evaluate_str(&params.expression, &content) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Search for JMESPath functions using fuzzy matching. Searches function names, descriptions, categories, signatures, and aliases. Returns ranked results with match type and relevance score. Essential for discovering functions when you're not sure of the exact name."
    )]
    async fn search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let results = self.engine.search_functions(&params.query, params.limit);
        json_result(&results)
    }

    #[tool(
        description = "Find functions similar to a specified function. Returns functions in the same category, functions with similar signatures (same input/output types), and functions with related concepts based on description keywords. Useful for discovering alternative approaches."
    )]
    async fn similar(
        &self,
        Parameters(params): Parameters<SimilarParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.similar_functions(&params.function) {
            Some(result) => json_result(&result),
            None => Ok(error_result(format!(
                "Unknown function '{}'. Use the 'search' tool to find functions.",
                params.function
            ))),
        }
    }

    #[tool(
        description = "Analyze JSON data and return statistics including type, size, depth, field analysis for arrays of objects, and type distribution. Useful for understanding data structure before writing queries."
    )]
    async fn stats(
        &self,
        Parameters(params): Parameters<StatsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.stats(&params.input) {
            Ok(stats) => json_result(&stats),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Extract all paths from JSON data in dot notation (e.g., 'users.0.name'). Optionally includes type information and values. Essential for understanding complex JSON structure before writing JMESPath queries."
    )]
    async fn paths(
        &self,
        Parameters(params): Parameters<PathsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .engine
            .paths(&params.input, params.include_types, params.include_values)
        {
            Ok(paths) => json_result(&paths),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    // =========================================================================
    // Discovery tools
    // =========================================================================

    #[tool(
        description = "Get the JSON schema for the MCP discovery protocol. MCP servers can use this schema to register their tools with jpx for cross-server discovery and search."
    )]
    async fn get_discovery_schema(
        &self,
        Parameters(_params): Parameters<GetDiscoverySchemaParams>,
    ) -> Result<CallToolResult, McpError> {
        let schema = self.engine.get_discovery_schema();
        json_result(&schema)
    }

    #[tool(
        description = "Register an MCP server's capabilities for discovery. Accepts a discovery spec with server info and tool definitions. Tools are indexed for full-text search across name, description, tags, and parameters."
    )]
    async fn register_discovery(
        &self,
        Parameters(params): Parameters<RegisterDiscoveryParams>,
    ) -> Result<CallToolResult, McpError> {
        let spec: DiscoverySpec = serde_json::from_value(params.spec).map_err(|e| {
            McpError::invalid_params(format!("Invalid discovery spec: {}", e), None)
        })?;

        if spec.server.name.is_empty() {
            return Err(McpError::invalid_params(
                "server.name is required and cannot be empty",
                None,
            ));
        }

        match self.engine.register_discovery(spec, params.replace) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Register tools using a simplified format. Just provide server name and a list of tools with name, description, and tags. Easier than the full discovery spec for simple use cases."
    )]
    async fn register_tools_simple(
        &self,
        Parameters(params): Parameters<RegisterToolsSimpleParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.server_name.is_empty() {
            return Err(McpError::invalid_params(
                "server_name is required and cannot be empty",
                None,
            ));
        }

        let tools: Vec<ToolSpec> = params
            .tools
            .into_iter()
            .map(|t| ToolSpec {
                name: t.name,
                aliases: vec![],
                category: None,
                subcategory: None,
                tags: t.tags,
                summary: t.description.clone(),
                description: t.description,
                params: vec![],
                returns: None,
                examples: vec![],
                related: vec![],
                since: None,
                stability: None,
            })
            .collect();

        let spec = DiscoverySpec {
            schema: None,
            server: DiscoveryServerInfo {
                name: params.server_name,
                version: params.version,
                description: None,
            },
            tools,
            categories: std::collections::HashMap::new(),
        };

        match self.engine.register_discovery(spec, true) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Search for tools across all registered MCP servers. Uses BM25 full-text search to find relevant tools by name, description, tags, category, or parameters. Returns ranked results with match scores."
    )]
    async fn query_tools(
        &self,
        Parameters(params): Parameters<QueryToolsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.query_tools(&params.query, params.top_k) {
            Ok(results) => json_result(&results),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Find tools similar to a specified tool based on shared terms and concepts. Uses the tool's indexed content to find related tools across all registered servers."
    )]
    async fn similar_tools(
        &self,
        Parameters(params): Parameters<SimilarToolsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.similar_tools(&params.tool_id, params.top_k) {
            Ok(results) => json_result(&results),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Remove an MCP server's tools from the discovery index. Use this when a server is no longer available or to re-register with updated tools."
    )]
    async fn unregister_discovery(
        &self,
        Parameters(params): Parameters<UnregisterDiscoveryParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.unregister_discovery(&params.server_name) {
            Ok(true) => {
                json_result(&serde_json::json!({"ok": true, "message": "Server unregistered"}))
            }
            Ok(false) => Ok(error_result(format!(
                "Server '{}' not found",
                params.server_name
            ))),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "List all MCP servers that have registered their tools for discovery. Returns server names, versions, descriptions, and tool counts."
    )]
    async fn list_discovery_servers(&self) -> Result<CallToolResult, McpError> {
        match self.engine.list_discovery_servers() {
            Ok(servers) => json_result(&servers),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "List all tool categories from registered MCP servers. Returns category names with tool counts and which servers provide tools in each category."
    )]
    async fn list_discovery_categories(&self) -> Result<CallToolResult, McpError> {
        match self.engine.list_discovery_categories() {
            Ok(categories) => json_result(&categories),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Get statistics about the discovery index including document count, term count, average document length, and top indexed terms. Useful for understanding what's been indexed."
    )]
    async fn inspect_discovery_index(&self) -> Result<CallToolResult, McpError> {
        match self.engine.discovery_index_stats() {
            Ok(Some(stats)) => json_result(&stats),
            Ok(None) => json_result(&serde_json::json!({"message": "No tools indexed yet"})),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    // =========================================================================
    // Query store tools
    // =========================================================================

    #[tool(
        description = "Store a named JMESPath query for reuse during this session. Useful for building and refining complex queries iteratively. The query is validated before storing."
    )]
    async fn define_query(
        &self,
        Parameters(params): Parameters<DefineQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        match self
            .engine
            .define_query(params.name.clone(), params.expression, params.description)
        {
            Ok(prev) => {
                let msg = if prev.is_some() {
                    format!("Query '{}' updated", params.name)
                } else {
                    format!("Query '{}' defined", params.name)
                };
                json_result(&serde_json::json!({"ok": true, "message": msg}))
            }
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Retrieve a stored query by name. Returns the expression and description if found."
    )]
    async fn get_query(
        &self,
        Parameters(params): Parameters<GetQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.get_query(&params.name) {
            Ok(Some(query)) => json_result(&query),
            Ok(None) => Ok(error_result(format!("Query '{}' not found", params.name))),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(description = "Delete a stored query by name. Returns the deleted query if it existed.")]
    async fn delete_query(
        &self,
        Parameters(params): Parameters<DeleteQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.engine.delete_query(&params.name) {
            Ok(Some(query)) => json_result(&serde_json::json!({
                "ok": true,
                "deleted": query
            })),
            Ok(None) => Ok(error_result(format!("Query '{}' not found", params.name))),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "List all named queries stored in this session. Shows query names, expressions, and descriptions."
    )]
    async fn list_queries(&self) -> Result<CallToolResult, McpError> {
        match self.engine.list_queries() {
            Ok(queries) => json_result(&queries),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(
        description = "Execute a stored query by name against JSON input. Combines the convenience of named queries with evaluation."
    )]
    async fn run_query(
        &self,
        Parameters(params): Parameters<RunQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let input: Value = serde_json::from_str(&params.input)
            .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {}", e), None))?;

        match self.engine.run_query(&params.name, &input) {
            Ok(result) => json_result(&result),
            Err(e) => Err(McpError::invalid_params(e.to_string(), None)),
        }
    }
}

#[tool_handler]
impl ServerHandler for JpxMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "JMESPath query tool with 400+ extended functions. \
                \n\nDISCOVERY: Use 'search' to find functions by keyword, 'similar' to find related functions, \
                'functions' to list all (optionally by category), 'describe' for function details, 'categories' to list categories. \
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
