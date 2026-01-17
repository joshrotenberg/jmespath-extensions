//! Error types for the jpx engine.

use thiserror::Error;

/// Engine error type.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Expression compilation failed
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    /// JSON parsing failed
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    /// Expression evaluation failed
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    /// Function not found
    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    /// Query not found
    #[error("Query not found: {0}")]
    QueryNotFound(String),

    /// Discovery registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Internal error (lock poisoning, etc.)
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for engine operations.
pub type Result<T> = std::result::Result<T, EngineError>;
