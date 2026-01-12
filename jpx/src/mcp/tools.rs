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
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Global function registry for introspection
static REGISTRY: OnceLock<FunctionRegistry> = OnceLock::new();

/// Get the global JMESPath runtime
fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register_all(&mut runtime);
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
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for a single expression in a batch evaluation
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
pub struct BatchEvaluateResult {
    /// Results for each expression in order
    pub results: Vec<BatchExpressionResult>,
}

// =============================================================================
// Helper functions
// =============================================================================

/// Parse category string to Category enum
fn parse_category(name: &str) -> Option<Category> {
    match name.to_lowercase().as_str() {
        "standard" => Some(Category::Standard),
        "string" => Some(Category::String),
        "array" => Some(Category::Array),
        "object" => Some(Category::Object),
        "math" => Some(Category::Math),
        "type" => Some(Category::Type),
        "utility" => Some(Category::Utility),
        "validation" => Some(Category::Validation),
        "path" => Some(Category::Path),
        "expression" => Some(Category::Expression),
        "text" => Some(Category::Text),
        "hash" => Some(Category::Hash),
        "encoding" => Some(Category::Encoding),
        "regex" => Some(Category::Regex),
        "url" => Some(Category::Url),
        "uuid" => Some(Category::Uuid),
        "rand" => Some(Category::Rand),
        "datetime" => Some(Category::Datetime),
        "fuzzy" => Some(Category::Fuzzy),
        "phonetic" => Some(Category::Phonetic),
        "geo" => Some(Category::Geo),
        "semver" => Some(Category::Semver),
        "network" => Some(Category::Network),
        "ids" => Some(Category::Ids),
        "duration" => Some(Category::Duration),
        "color" => Some(Category::Color),
        "computing" => Some(Category::Computing),
        "multimatch" => Some(Category::MultiMatch),
        "jsonpatch" => Some(Category::Jsonpatch),
        "format" => Some(Category::Format),
        _ => None,
    }
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
}

impl JpxMcp {
    pub fn new() -> Self {
        // Initialize the runtime and registry eagerly
        let _ = runtime();
        let _ = registry();
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for JpxMcp {
    fn default() -> Self {
        Self::new()
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
        // Compile expression
        let expr = runtime()
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
        let result = match runtime().compile(&params.expression) {
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

        let rt = runtime();

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
                 'batch_evaluate' for multiple expressions on the same input, 'format' to pretty-print JSON, \
                 'functions' to discover available functions, 'describe' for function details, \
                 'categories' to list function categories, and 'validate' to check expression syntax."
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
    fn test_runtime_initialization() {
        let rt = runtime();
        // Should be able to compile a basic expression
        assert!(rt.compile("@").is_ok());
        assert!(rt.compile("upper(@)").is_ok());
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
        let mcp = JpxMcp::new();
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
        let mcp = JpxMcp::new();
        let params = FormatParams {
            input: r#"{"name":"alice","age":30}"#.to_string(),
            indent: 2,
        };
        let result = mcp.format(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        // Check formatted output - Content derefs to RawContent
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
        let mcp = JpxMcp::new();
        let params = FormatParams {
            input: r#"{ "name" : "alice" , "age" : 30 }"#.to_string(),
            indent: 0,
        };
        let result = mcp.format(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                // Compact output should have no unnecessary whitespace
                assert_eq!(text_content.text, r#"{"age":30,"name":"alice"}"#);
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_format_custom_indent() {
        let mcp = JpxMcp::new();
        let params = FormatParams {
            input: r#"{"key":"value"}"#.to_string(),
            indent: 4,
        };
        let result = mcp.format(Parameters(params)).await.unwrap();
        assert_eq!(result.is_error, Some(false));

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text_content) = &content.raw {
                assert!(text_content.text.contains("    \"key\": \"value\""));
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected content");
        }
    }

    #[tokio::test]
    async fn test_format_invalid_json() {
        let mcp = JpxMcp::new();
        let params = FormatParams {
            input: r#"{"invalid": }"#.to_string(),
            indent: 2,
        };
        let result = mcp.format(Parameters(params)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid JSON"));
    }
}
