//! Encoding functions.
//!
//! These functions provide base64 and hex encoding/decoding.
//! Requires the `encoding` feature.

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};

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
                ErrorReason::Parse("Invalid hex input".to_owned()),
            )),
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
}
