//! Utility functions.
//!
//! This module provides utility functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category utility`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::utility;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! utility::register(&mut runtime);
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all utility functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("now", Box::new(NowFn::new()));
    runtime.register_function("now_ms", Box::new(NowMsFn::new()));
    runtime.register_function("default", Box::new(DefaultFn::new()));
    runtime.register_function("if", Box::new(IfFn::new()));
    runtime.register_function("coalesce", Box::new(CoalesceFn::new()));
    runtime.register_function("json_encode", Box::new(JsonEncodeFn::new()));
    runtime.register_function("json_decode", Box::new(JsonDecodeFn::new()));
    runtime.register_function("json_pointer", Box::new(JsonPointerFn::new()));
    runtime.register_function("pretty", Box::new(PrettyFn::new()));
    #[cfg(feature = "env")]
    {
        runtime.register_function("env", Box::new(EnvFn::new()));
        runtime.register_function("get_env", Box::new(GetEnvFn::new()));
    }
}

// =============================================================================
// now(fallback?) -> number (Unix timestamp in seconds)
// =============================================================================

define_function!(NowFn, vec![], Some(ArgumentType::Number));

impl Function for NowFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        if let Some(fallback) = args.first() {
            if let Some(n) = fallback.as_number() {
                return Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0)),
                )));
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(Rc::new(Variable::Number(serde_json::Number::from(
            timestamp,
        ))))
    }
}

// =============================================================================
// now_ms(fallback?) -> number (Unix timestamp in milliseconds)
// =============================================================================

define_function!(NowMsFn, vec![], Some(ArgumentType::Number));

impl Function for NowMsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        if let Some(fallback) = args.first() {
            if let Some(n) = fallback.as_number() {
                return Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0)),
                )));
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(Rc::new(Variable::Number(serde_json::Number::from(
            timestamp,
        ))))
    }
}

// =============================================================================
// default(value, default_value) -> value if not null, else default
// =============================================================================

define_function!(DefaultFn, vec![ArgumentType::Any, ArgumentType::Any], None);

impl Function for DefaultFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        if args[0].is_null() {
            Ok(args[1].clone())
        } else {
            Ok(args[0].clone())
        }
    }
}

// =============================================================================
// if(condition, then_value, else_value) -> any
// =============================================================================

define_function!(
    IfFn,
    vec![ArgumentType::Any, ArgumentType::Any, ArgumentType::Any],
    None
);

impl Function for IfFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let condition = &args[0];
        let then_value = &args[1];
        let else_value = &args[2];

        let is_truthy = match &**condition {
            Variable::Bool(b) => *b,
            Variable::Null => false,
            _ => true,
        };

        if is_truthy {
            Ok(then_value.clone())
        } else {
            Ok(else_value.clone())
        }
    }
}

// =============================================================================
// coalesce(...) -> any (first non-null value)
// =============================================================================

define_function!(CoalesceFn, vec![], Some(ArgumentType::Any));

impl Function for CoalesceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        for arg in args {
            if !arg.is_null() {
                return Ok(arg.clone());
            }
        }
        Ok(Rc::new(Variable::Null))
    }
}

// =============================================================================
// json_encode(any) -> string
// =============================================================================

define_function!(JsonEncodeFn, vec![ArgumentType::Any], None);

impl Function for JsonEncodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let json_str = serde_json::to_string(&*args[0]).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Failed to encode as JSON".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::String(json_str)))
    }
}

// =============================================================================
// pretty(any, indent?) -> string
// =============================================================================

define_function!(
    PrettyFn,
    vec![ArgumentType::Any],
    Some(ArgumentType::Number)
);

impl Function for PrettyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let indent = if args.len() > 1 {
            args[1].as_number().unwrap_or(2.0) as usize
        } else {
            2
        };

        // For default indent of 2, use built-in to_string_pretty
        if indent == 2 {
            let pretty_str = serde_json::to_string_pretty(&*args[0]).map_err(|_| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Failed to serialize as JSON".to_owned()),
                )
            })?;
            return Ok(Rc::new(Variable::String(pretty_str)));
        }

        // For custom indent, manually format
        // Convert to compact JSON first, then parse as Value
        let json_str = serde_json::to_string(&*args[0]).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Failed to serialize as JSON".to_owned()),
            )
        })?;

        // Manually build pretty output with custom indent
        let pretty_str = pretty_print_json(&json_str, indent);

        Ok(Rc::new(Variable::String(pretty_str)))
    }
}

/// Pretty print JSON with custom indentation
fn pretty_print_json(json: &str, indent_size: usize) -> String {
    let mut result = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let indent = " ".repeat(indent_size);

    for ch in json.chars() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        if ch == '\\' && in_string {
            result.push(ch);
            escape_next = true;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }

        if in_string {
            result.push(ch);
            continue;
        }

        match ch {
            '{' | '[' => {
                result.push(ch);
                depth += 1;
                result.push('\n');
                for _ in 0..depth {
                    result.push_str(&indent);
                }
            }
            '}' | ']' => {
                depth -= 1;
                result.push('\n');
                for _ in 0..depth {
                    result.push_str(&indent);
                }
                result.push(ch);
            }
            ',' => {
                result.push(ch);
                result.push('\n');
                for _ in 0..depth {
                    result.push_str(&indent);
                }
            }
            ':' => {
                result.push_str(": ");
            }
            ' ' | '\n' | '\t' | '\r' => {
                // Skip whitespace in compact JSON
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

// =============================================================================
// json_decode(string) -> any
// =============================================================================

define_function!(JsonDecodeFn, vec![ArgumentType::String], None);

impl Function for JsonDecodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Return null for invalid JSON instead of erroring
        match Variable::from_json(s) {
            Ok(var) => Ok(Rc::new(var)),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// json_pointer(any, string) -> any (RFC 6901 JSON Pointer)
// =============================================================================

define_function!(
    JsonPointerFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

impl Function for JsonPointerFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let pointer = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string pointer argument".to_owned()),
            )
        })?;

        // Convert Variable to serde_json::Value for pointer resolution
        let json_str = serde_json::to_string(&*args[0]).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Failed to serialize value".to_owned()),
            )
        })?;

        let json_value: serde_json::Value = serde_json::from_str(&json_str).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Failed to parse value".to_owned()),
            )
        })?;

        // Use serde_json's built-in pointer method
        match json_value.pointer(pointer) {
            Some(result) => {
                let result_str = serde_json::to_string(result).map_err(|_| {
                    JmespathError::new(
                        ctx.expression,
                        0,
                        ErrorReason::Parse("Failed to serialize result".to_owned()),
                    )
                })?;
                let var = Variable::from_json(&result_str).map_err(|_| {
                    JmespathError::new(
                        ctx.expression,
                        0,
                        ErrorReason::Parse("Failed to convert result".to_owned()),
                    )
                })?;
                Ok(Rc::new(var))
            }
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// env() -> object (all environment variables)
// Requires "env" feature - opt-in for security
// =============================================================================

#[cfg(feature = "env")]
define_function!(EnvFn, vec![], None);

#[cfg(feature = "env")]
impl Function for EnvFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let mut map = std::collections::BTreeMap::new();
        for (key, value) in std::env::vars() {
            map.insert(key, Rc::new(Variable::String(value)));
        }

        Ok(Rc::new(Variable::Object(map)))
    }
}

// =============================================================================
// get_env(name) -> string | null (single environment variable)
// Requires "env" feature - opt-in for security
// =============================================================================

#[cfg(feature = "env")]
define_function!(GetEnvFn, vec![ArgumentType::String], None);

#[cfg(feature = "env")]
impl Function for GetEnvFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let name = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match std::env::var(name) {
            Ok(value) => Ok(Rc::new(Variable::String(value))),
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
    fn test_default() {
        let runtime = setup_runtime();
        let expr = runtime.compile("default(@, 'fallback')").unwrap();

        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "fallback");

        let result = expr.search(Variable::String("value".to_string())).unwrap();
        assert_eq!(result.as_string().unwrap(), "value");
    }

    #[test]
    fn test_if() {
        let runtime = setup_runtime();
        let expr = runtime.compile("if(`true`, 'yes', 'no')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");

        let expr = runtime.compile("if(`false`, 'yes', 'no')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "no");
    }

    #[test]
    fn test_json_decode_object() {
        let runtime = setup_runtime();
        // Test parsing a JSON object string
        let expr = runtime.compile("json_decode(@)").unwrap();
        let data = Variable::String(r#"{"a":1,"b":2}"#.to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("a"));
    }

    #[test]
    fn test_json_decode_from_field() {
        let runtime = setup_runtime();
        // This simulates: {"s": "{\"a\":1,\"b\":2}"}
        // When accessed as s, we get the string {"a":1,"b":2}
        let expr = runtime.compile("json_decode(s)").unwrap();

        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "s".to_string(),
            Rc::new(Variable::String(r#"{"a":1,"b":2}"#.to_string())),
        );
        let data = Variable::Object(map);

        let result = expr.search(&data);
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.is_object());
    }

    #[test]
    fn test_json_decode_invalid_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("json_decode(@)").unwrap();
        let data = Variable::String("not valid json".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_json_pointer_nested() {
        let runtime = setup_runtime();
        let expr = runtime.compile("json_pointer(@, '/foo/bar/1')").unwrap();
        let data = Variable::from_json(r#"{"foo": {"bar": [1, 2, 3]}}"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_json_pointer_root() {
        let runtime = setup_runtime();
        let expr = runtime.compile("json_pointer(@, '')").unwrap();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_object());
    }

    #[test]
    fn test_json_pointer_missing() {
        let runtime = setup_runtime();
        let expr = runtime.compile("json_pointer(@, '/missing')").unwrap();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_json_pointer_array() {
        let runtime = setup_runtime();
        let expr = runtime.compile("json_pointer(@, '/0')").unwrap();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_pretty_default_indent() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pretty(@)").unwrap();
        let data = Variable::from_json(r#"{"a":1,"b":[2,3]}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_string().unwrap();
        assert!(s.contains('\n'));
        assert!(s.contains("  ")); // 2-space indent
    }

    #[test]
    fn test_pretty_custom_indent() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pretty(@, `4`)").unwrap();
        let data = Variable::from_json(r#"{"a":1}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let s = result.as_string().unwrap();
        assert!(s.contains("    ")); // 4-space indent
    }

    #[test]
    fn test_pretty_simple_value() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pretty(@)").unwrap();
        let data = Variable::String("hello".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "\"hello\"");
    }

    #[cfg(feature = "env")]
    #[test]
    fn test_env_returns_object() {
        let runtime = setup_runtime();
        let expr = runtime.compile("env()").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!(result.is_object());
    }

    #[cfg(feature = "env")]
    #[test]
    fn test_get_env_existing() {
        // PATH should exist on all systems
        let runtime = setup_runtime();
        let expr = runtime.compile("get_env('PATH')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!(result.is_string());
    }

    #[cfg(feature = "env")]
    #[test]
    fn test_get_env_missing() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("get_env('THIS_ENV_VAR_SHOULD_NOT_EXIST_12345')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!(result.is_null());
    }
}
