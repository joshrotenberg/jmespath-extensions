//! Common types for engine requests and responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request to evaluate a JMESPath expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRequest {
    /// The JMESPath expression to evaluate
    pub expression: String,
    /// The JSON input to evaluate against
    pub input: Value,
}

/// Response from evaluating a JMESPath expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResponse {
    /// The result of evaluation
    pub result: Value,
}

/// Result of validating a JMESPath expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the expression is valid
    pub valid: bool,
    /// Error message if invalid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for a single expression in batch evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Result of batch evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEvaluateResult {
    /// Results for each expression in order
    pub results: Vec<BatchExpressionResult>,
}
