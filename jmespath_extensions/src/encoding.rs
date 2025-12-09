//! Encoding functions.
//!
//! This module provides binary-to-text encoding and decoding capabilities for JMESPath expressions.
//! It supports Base64 and hexadecimal encoding schemes, allowing you to encode strings to these
//! formats and decode them back to their original form.
//!
//! **Note:** This module requires the `encoding` feature to be enabled.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description |
//! |----------|-----------|---------|-------------|
//! | `base64_encode` | `(text: string)` | `string` | Encode string to Base64 |
//! | `base64_decode` | `(base64: string)` | `string` | Decode Base64 to string |
//! | `hex_encode` | `(text: string)` | `string` | Encode string to hexadecimal |
//! | `hex_decode` | `(hex: string)` | `string` | Decode hexadecimal to string |
//!
//! # Examples
//!
//! ```rust
//! use jmespath_extensions::Runtime;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! jmespath_extensions::register_all(&mut runtime);
//!
//! let expr = runtime.compile("base64_encode(@)").unwrap();
//! let data = jmespath::Variable::String("hello".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(), "aGVsbG8=");
//! ```
//!
//! # Function Details
//!
//! ## Base64 Encoding
//!
//! ### `base64_encode(text: string) -> string`
//!
//! Encodes a string to Base64 format using the standard Base64 alphabet (RFC 4648).
//! The output uses padding characters (=) as needed.
//!
//! ```text
//! base64_encode('hello')                   // "aGVsbG8="
//! base64_encode('Hello World')             // "SGVsbG8gV29ybGQ="
//! base64_encode('')                        // ""
//! base64_encode('test123')                 // "dGVzdDEyMw=="
//! ```
//!
//! ### `base64_decode(base64: string) -> string`
//!
//! Decodes a Base64-encoded string back to its original form. Returns an error if the input
//! is not valid Base64 or if the decoded bytes are not valid UTF-8.
//!
//! ```text
//! base64_decode('aGVsbG8=')                // "hello"
//! base64_decode('SGVsbG8gV29ybGQ=')        // "Hello World"
//! base64_decode('dGVzdDEyMw==')            // "test123"
//! base64_decode('invalid!')                // Error: Invalid base64 input
//! ```
//!
//! ## Hexadecimal Encoding
//!
//! ### `hex_encode(text: string) -> string`
//!
//! Encodes a string to lowercase hexadecimal format. Each byte is represented by two
//! hexadecimal digits.
//!
//! ```text
//! hex_encode('hello')                      // "68656c6c6f"
//! hex_encode('Hello World')                // "48656c6c6f20576f726c64"
//! hex_encode('')                           // ""
//! hex_encode('ABC')                        // "414243"
//! ```
//!
//! ### `hex_decode(hex: string) -> string`
//!
//! Decodes a hexadecimal string back to its original form. Accepts both lowercase and
//! uppercase hex digits. Returns null if the input is not valid hexadecimal or if
//! the decoded bytes are not valid UTF-8.
//!
//! ```text
//! hex_decode('68656c6c6f')                 // "hello"
//! hex_decode('48656C6C6F20576F726C64')     // "Hello World" (uppercase works)
//! hex_decode('414243')                     // "ABC"
//! hex_decode('invalid')                    // null (invalid hex input)
//! hex_decode('123')                        // null (odd length hex string)
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};

/// Register all encoding functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("base64_encode", Box::new(Base64EncodeFn::new()));
    runtime.register_function("base64_decode", Box::new(Base64DecodeFn::new()));
    runtime.register_function("hex_encode", Box::new(HexEncodeFn::new()));
    runtime.register_function("hex_decode", Box::new(HexDecodeFn::new()));
}

// =============================================================================
// base64_encode(string) -> string
// =============================================================================

define_function!(Base64EncodeFn, vec![ArgumentType::String], None);

impl Function for Base64EncodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = BASE64_STANDARD.encode(input.as_bytes());
        Ok(Rc::new(Variable::String(encoded)))
    }
}

// =============================================================================
// base64_decode(string) -> string
// =============================================================================

define_function!(Base64DecodeFn, vec![ArgumentType::String], None);

impl Function for Base64DecodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match BASE64_STANDARD.decode(input.as_bytes()) {
            Ok(decoded) => {
                let s = String::from_utf8(decoded).map_err(|_| {
                    JmespathError::new(
                        ctx.expression,
                        0,
                        ErrorReason::Parse("Decoded bytes are not valid UTF-8".to_owned()),
                    )
                })?;
                Ok(Rc::new(Variable::String(s)))
            }
            Err(_) => Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Invalid base64 input".to_owned()),
            )),
        }
    }
}

// =============================================================================
// hex_encode(string) -> string
// =============================================================================

define_function!(HexEncodeFn, vec![ArgumentType::String], None);

impl Function for HexEncodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = hex::encode(input.as_bytes());
        Ok(Rc::new(Variable::String(encoded)))
    }
}

// =============================================================================
// hex_decode(string) -> string
// =============================================================================

define_function!(HexDecodeFn, vec![ArgumentType::String], None);

impl Function for HexDecodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match hex::decode(input) {
            Ok(decoded) => {
                // Return null if decoded bytes are not valid UTF-8
                match String::from_utf8(decoded) {
                    Ok(s) => Ok(Rc::new(Variable::String(s))),
                    Err(_) => Ok(Rc::new(Variable::Null)),
                }
            }
            // Return null for invalid hex input
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jmespath::Runtime;

    fn setup_runtime() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        runtime
    }

    #[test]
    fn test_base64_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64_encode(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "aGVsbG8=");
    }

    #[test]
    fn test_base64_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("base64_decode(@)").unwrap();
        let data = Variable::String("aGVsbG8=".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_hex_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_encode(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "68656c6c6f");
    }

    #[test]
    fn test_hex_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_decode(@)").unwrap();
        let data = Variable::String("68656c6c6f".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");
    }

    #[test]
    fn test_hex_decode_invalid_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_decode(@)").unwrap();
        let data = Variable::String("invalid".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_hex_decode_odd_length_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("hex_decode(@)").unwrap();
        let data = Variable::String("123".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }
}
