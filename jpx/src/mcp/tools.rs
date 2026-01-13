//! MCP tool implementations for JMESPath

use jmespath::Runtime;
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{Category, FunctionInfo, FunctionRegistry};
use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

/// Global JMESPath runtime with all extensions registered
static RUNTIME_FULL: OnceLock<Runtime> = OnceLock::new();

/// Global JMESPath runtime with only standard functions (strict mode)
static RUNTIME_STRICT: OnceLock<Runtime> = OnceLock::new();

/// Global function registry for introspection
static REGISTRY: OnceLock<FunctionRegistry> = OnceLock::new();

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
        description = "Evaluate a JMESPath expression against JSON input. Returns the result of applying the expression to the input data. Supports 320+ extended functions beyond standard JMESPath."
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
                "JMESPath query tool with 320+ extended functions. Use 'evaluate' to run queries, \
                 'evaluate_file' to query JSON files directly, 'batch_evaluate' for multiple expressions, \
                 'format' to pretty-print JSON, 'diff' to generate RFC 6902 JSON Patches, \
                 'patch' to apply RFC 6902 patches, 'merge' to apply RFC 7396 JSON Merge Patches, \
                 'keys' to extract object keys (optionally recursive), 'functions' to discover functions, \
                 'describe' for function details, 'categories' to list categories, and 'validate' to check syntax."
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
}
