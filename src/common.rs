//! Common types and utilities for JMESPath extension functions.
//!
//! This module provides helper functions and re-exports for implementing custom JMESPath functions.
//!
//! # Error Handling
//!
//! When implementing custom functions, use these helpers for consistent error messages:
//!
//! - [`invalid_type_error`] - For type mismatches (produces structured `RuntimeError::InvalidType`)
//! - [`custom_error`] - For domain-specific errors (e.g., "Invalid regex pattern")
//!
//! ## Example
//!
//! ```ignore
//! impl Function for MyFn {
//!     fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
//!         self.signature.validate(args, ctx)?;
//!
//!         // After signature validation, type checks are guaranteed to pass.
//!         // Use unwrap() for validated types, or invalid_type_error() for custom validation.
//!         let s = args[0].as_string().unwrap();
//!
//!         // For domain-specific errors:
//!         if s.is_empty() {
//!             return Err(custom_error(ctx, "String cannot be empty"));
//!         }
//!
//!         Ok(Rc::new(Variable::String(s.to_uppercase())))
//!     }
//! }
//! ```

use std::rc::Rc;

pub use jmespath::RuntimeError;
pub use jmespath::functions::{ArgumentType, Function, Signature};
pub use jmespath::{Context, ErrorReason, JmespathError, Rcvar, Runtime, Variable};

/// Creates a JmespathError for an invalid argument type.
///
/// This produces a structured `RuntimeError::InvalidType` error which provides
/// better debugging information than a generic parse error string.
///
/// # Arguments
/// * `ctx` - The evaluation context
/// * `position` - The argument position (0-indexed)
/// * `expected` - Description of the expected type(s)
/// * `actual` - The actual value that was provided
///
/// # Example
/// ```ignore
/// let err = invalid_type_error(ctx, 0, "string", &args[0]);
/// ```
pub fn invalid_type_error(
    ctx: &Context<'_>,
    position: usize,
    expected: &str,
    actual: &Rcvar,
) -> JmespathError {
    JmespathError::from_ctx(
        ctx,
        ErrorReason::Runtime(RuntimeError::InvalidType {
            expected: expected.to_owned(),
            actual: actual.get_type().to_string(),
            position,
        }),
    )
}

/// Creates a JmespathError for a custom runtime error message.
///
/// Use this for domain-specific errors that don't fit the standard error types,
/// such as "Invalid regex pattern" or "Division by zero".
///
/// # Arguments
/// * `ctx` - The evaluation context
/// * `message` - A descriptive error message
///
/// # Example
/// ```ignore
/// let err = custom_error(ctx, "Invalid regex pattern: unclosed group");
/// ```
pub fn custom_error(ctx: &Context<'_>, message: &str) -> JmespathError {
    JmespathError::from_ctx(ctx, ErrorReason::Parse(message.to_owned()))
}

/// Helper macro for defining JMESPath custom functions.
///
/// This macro creates a struct with a signature field and implements
/// the necessary boilerplate for creating new JMESPath functions.
#[macro_export]
macro_rules! define_function {
    ($name:ident, $args:expr, $variadic:expr) => {
        pub struct $name {
            signature: $crate::common::Signature,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    signature: $crate::common::Signature::new($args, $variadic),
                }
            }
        }
    };
}

/// Helper to create an Rcvar from a Variable
#[inline]
pub fn rcvar(v: Variable) -> Rcvar {
    Rc::new(v)
}
