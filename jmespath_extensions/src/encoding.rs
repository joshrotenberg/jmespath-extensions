//! Encoding and decoding functions.
//!
//! This module provides encoding functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category encoding`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::encoding;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! encoding::register(&mut runtime);
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
    runtime.register_function("html_escape", Box::new(HtmlEscapeFn::new()));
    runtime.register_function("html_unescape", Box::new(HtmlUnescapeFn::new()));
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

// =============================================================================
// html_escape(string) -> string
// =============================================================================

define_function!(HtmlEscapeFn, vec![ArgumentType::String], None);

impl Function for HtmlEscapeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let escaped = s
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");

        Ok(Rc::new(Variable::String(escaped)))
    }
}

// =============================================================================
// html_unescape(string) -> string
// =============================================================================

define_function!(HtmlUnescapeFn, vec![ArgumentType::String], None);

impl Function for HtmlUnescapeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Order matters: decode &amp; last to avoid double-decoding
        let unescaped = s
            .replace("&#x27;", "'")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
            .replace("&quot;", "\"")
            .replace("&gt;", ">")
            .replace("&lt;", "<")
            .replace("&amp;", "&");

        Ok(Rc::new(Variable::String(unescaped)))
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

    #[test]
    fn test_html_escape_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_escape(@)").unwrap();
        let data = Variable::String("<div class=\"test\">Hello & goodbye</div>".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "&lt;div class=&quot;test&quot;&gt;Hello &amp; goodbye&lt;/div&gt;"
        );
    }

    #[test]
    fn test_html_escape_quotes() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_escape(@)").unwrap();
        let data = Variable::String("It's a \"test\"".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "It&#x27;s a &quot;test&quot;");
    }

    #[test]
    fn test_html_escape_no_change() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_escape(@)").unwrap();
        let data = Variable::String("Hello World".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello World");
    }

    #[test]
    fn test_html_unescape_basic() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_unescape(@)").unwrap();
        let data = Variable::String(
            "&lt;div class=&quot;test&quot;&gt;Hello &amp; goodbye&lt;/div&gt;".to_string(),
        );
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "<div class=\"test\">Hello & goodbye</div>"
        );
    }

    #[test]
    fn test_html_unescape_quotes() {
        let runtime = setup_runtime();
        let expr = runtime.compile("html_unescape(@)").unwrap();
        let data = Variable::String("It&#x27;s a &quot;test&quot;".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "It's a \"test\"");
    }

    #[test]
    fn test_html_roundtrip() {
        let runtime = setup_runtime();
        let escape = runtime.compile("html_escape(@)").unwrap();
        let unescape = runtime.compile("html_unescape(@)").unwrap();
        let original = "<script>alert('xss')</script>";
        let data = Variable::String(original.to_string());
        let escaped = escape.search(&data).unwrap();
        let roundtrip = unescape.search(&escaped).unwrap();
        assert_eq!(roundtrip.as_string().unwrap(), original);
    }
}
