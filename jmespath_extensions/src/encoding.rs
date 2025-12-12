//! Encoding functions.
//!
//! This module provides binary-to-text encoding and decoding capabilities for JMESPath expressions.
//! It supports Base64 and hexadecimal encoding schemes, allowing you to encode strings to these
//! formats and decode them back to their original form. It also includes JWT (JSON Web Token)
//! decoding functions for extracting claims from tokens.
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
//! | `jwt_decode` | `(token: string)` | `object` | Decode JWT payload (no verification) |
//! | `jwt_header` | `(token: string)` | `object` | Decode JWT header |
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
//!
//! ## JWT Functions
//!
//! JWT (JSON Web Token) functions decode tokens to extract their contents. These functions
//! perform decoding only - they do NOT verify signatures. Use these for:
//! - Extracting claims for routing/filtering decisions
//! - Inspecting token contents for debugging
//! - Pre-processing before signature verification elsewhere
//!
//! **Security Note:** Never trust JWT contents without proper signature verification.
//! These functions are for inspection only, not authentication.
//!
//! ### `jwt_decode(token: string) -> object`
//!
//! Decodes a JWT and returns the payload (claims) as a JSON object. Returns null if the
//! token is malformed or cannot be decoded.
//!
//! ```text
//! jwt_decode('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c')
//! // Returns: {"sub": "1234567890", "name": "John Doe", "iat": 1516239022}
//!
//! // Extract a specific claim:
//! jwt_decode(token).sub                    // "1234567890"
//!
//! // Use in filtering:
//! requests[?jwt_decode(auth_token).role == `"admin"`]
//! ```
//!
//! ### `jwt_header(token: string) -> object`
//!
//! Decodes a JWT and returns the header as a JSON object. Returns null if the token
//! is malformed or cannot be decoded.
//!
//! ```text
//! jwt_header('eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U')
//! // Returns: {"alg": "HS256", "typ": "JWT"}
//!
//! // Check algorithm:
//! jwt_header(token).alg                    // "HS256"
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

use base64::{
    Engine,
    engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD as BASE64_URL_SAFE},
};

/// Register all encoding functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("base64_encode", Box::new(Base64EncodeFn::new()));
    runtime.register_function("base64_decode", Box::new(Base64DecodeFn::new()));
    runtime.register_function("hex_encode", Box::new(HexEncodeFn::new()));
    runtime.register_function("hex_decode", Box::new(HexDecodeFn::new()));
    runtime.register_function("jwt_decode", Box::new(JwtDecodeFn::new()));
    runtime.register_function("jwt_header", Box::new(JwtHeaderFn::new()));
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

// =============================================================================
// JWT Helper Functions
// =============================================================================

/// Decode a base64url-encoded JWT part (header or payload) to JSON
fn decode_jwt_part(part: &str) -> Option<serde_json::Value> {
    // JWT uses base64url encoding (no padding)
    let decoded = BASE64_URL_SAFE.decode(part).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;
    serde_json::from_str(&json_str).ok()
}

/// Convert serde_json::Value to jmespath Variable
fn json_to_variable(value: serde_json::Value) -> Variable {
    match value {
        serde_json::Value::Null => Variable::Null,
        serde_json::Value::Bool(b) => Variable::Bool(b),
        serde_json::Value::Number(n) => Variable::Number(n),
        serde_json::Value::String(s) => Variable::String(s),
        serde_json::Value::Array(arr) => Variable::Array(
            arr.into_iter()
                .map(|v| Rc::new(json_to_variable(v)))
                .collect(),
        ),
        serde_json::Value::Object(map) => Variable::Object(
            map.into_iter()
                .map(|(k, v)| (k, Rc::new(json_to_variable(v))))
                .collect(),
        ),
    }
}

// =============================================================================
// jwt_decode(token) -> object (JWT payload/claims)
// =============================================================================

define_function!(JwtDecodeFn, vec![ArgumentType::String], None);

impl Function for JwtDecodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let token = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(Rc::new(Variable::Null));
        }

        // Decode the payload (second part)
        match decode_jwt_part(parts[1]) {
            Some(json) => Ok(Rc::new(json_to_variable(json))),
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// jwt_header(token) -> object (JWT header)
// =============================================================================

define_function!(JwtHeaderFn, vec![ArgumentType::String], None);

impl Function for JwtHeaderFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let token = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // JWT format: header.payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(Rc::new(Variable::Null));
        }

        // Decode the header (first part)
        match decode_jwt_part(parts[0]) {
            Some(json) => Ok(Rc::new(json_to_variable(json))),
            None => Ok(Rc::new(Variable::Null)),
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

    // =========================================================================
    // JWT function tests
    // =========================================================================

    // Test JWT from jwt.io: {"sub": "1234567890", "name": "John Doe", "iat": 1516239022}
    const TEST_JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    #[test]
    fn test_jwt_decode_payload() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();
        let data = Variable::String(TEST_JWT.to_string());
        let result = expr.search(&data).unwrap();

        // Check it's an object
        let obj = result.as_object().expect("Should be an object");

        // Check claims
        assert_eq!(obj.get("sub").unwrap().as_string().unwrap(), "1234567890");
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "John Doe");
        assert_eq!(
            obj.get("iat").unwrap().as_number().unwrap() as i64,
            1516239022
        );
    }

    #[test]
    fn test_jwt_decode_extract_claim() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@).sub").unwrap();
        let data = Variable::String(TEST_JWT.to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "1234567890");
    }

    #[test]
    fn test_jwt_header() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_header(@)").unwrap();
        let data = Variable::String(TEST_JWT.to_string());
        let result = expr.search(&data).unwrap();

        // Check it's an object
        let obj = result.as_object().expect("Should be an object");

        // Check header fields
        assert_eq!(obj.get("alg").unwrap().as_string().unwrap(), "HS256");
        assert_eq!(obj.get("typ").unwrap().as_string().unwrap(), "JWT");
    }

    #[test]
    fn test_jwt_header_extract_alg() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_header(@).alg").unwrap();
        let data = Variable::String(TEST_JWT.to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "HS256");
    }

    #[test]
    fn test_jwt_decode_invalid_format() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();

        // Not a valid JWT (no dots)
        let data = Variable::String("not-a-jwt".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());

        // Only two parts
        let data = Variable::String("part1.part2".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_jwt_decode_invalid_base64() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();

        // Three parts but invalid base64
        let data = Variable::String("!!!.@@@.###".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_jwt_decode_invalid_json() {
        let runtime = setup_runtime();
        let expr = runtime.compile("jwt_decode(@)").unwrap();

        // Valid base64 but not valid JSON - "not json" encoded
        let data = Variable::String("eyJhbGciOiJIUzI1NiJ9.bm90IGpzb24.sig".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }
}
