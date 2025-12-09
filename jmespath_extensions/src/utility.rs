//! Utility and conditional functions.
//!
//! This module provides essential utility functions for JMESPath expressions, including timestamp
//! generation, default value handling, conditional logic, and JSON serialization. These functions
//! help with common data manipulation tasks and control flow.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description |
//! |----------|-----------|---------|-------------|
//! | `now` | `(fallback?: number)` | `number` | Current Unix timestamp in seconds |
//! | `now_ms` | `(fallback?: number)` | `number` | Current Unix timestamp in milliseconds |
//! | `default` | `(value: any, default: any)` | `any` | Return value if not null, else default |
//! | `if` | `(condition: any, then: any, else: any)` | `any` | Ternary conditional operator |
//! | `coalesce` | `(...values: any)` | `any` | Return first non-null value |
//! | `json_encode` | `(value: any)` | `string` | Serialize value to JSON string |
//! | `json_decode` | `(json: string)` | `any` | Parse JSON string to value |
//! | `json_pointer` | `(value: any, pointer: string)` | `any` | Access value using JSON Pointer (RFC 6901) |
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
//! let expr = runtime.compile("default(value, 'fallback')").unwrap();
//! let data = jmespath::Variable::from_json(r#"{"value": null}"#).unwrap();
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(), "fallback");
//! ```
//!
//! # Function Details
//!
//! ## Timestamp Functions
//!
//! ### `now(fallback?: number) -> number`
//!
//! Returns the current Unix timestamp in seconds. If an optional fallback is provided and is a
//! valid number, it will be returned instead (useful for testing).
//!
//! ```text
//! now()                     // 1701234567 (current time)
//! now(`1234567890`)         // 1234567890 (fallback value)
//! ```
//!
//! ### `now_ms(fallback?: number) -> number`
//!
//! Returns the current Unix timestamp in milliseconds. If an optional fallback is provided and is
//! a valid number, it will be returned instead.
//!
//! ```text
//! now_ms()                  // 1701234567890 (current time)
//! now_ms(`1234567890000`)   // 1234567890000 (fallback value)
//! ```
//!
//! ## Conditional Functions
//!
//! ### `default(value: any, default_value: any) -> any`
//!
//! Returns the first argument if it is not null, otherwise returns the default value.
//!
//! ```text
//! default(name, 'Unknown')           // "John" if name="John", else "Unknown"
//! default(null, 'fallback')          // "fallback"
//! default('', 'fallback')            // "" (empty string is not null)
//! default(`0`, `42`)                 // 0 (zero is not null)
//! ```
//!
//! ### `if(condition: any, then_value: any, else_value: any) -> any`
//!
//! Ternary conditional operator. Returns then_value if condition is truthy, else returns else_value.
//! Truthy values are everything except false and null.
//!
//! ```text
//! if(`true`, 'yes', 'no')            // "yes"
//! if(`false`, 'yes', 'no')           // "no"
//! if(null, 'yes', 'no')              // "no"
//! if(`0`, 'yes', 'no')               // "yes" (0 is truthy)
//! if(`""`, 'yes', 'no')              // "yes" (empty string is truthy)
//! if(count > `5`, 'many', 'few')     // conditional based on comparison
//! ```
//!
//! ### `coalesce(...values: any) -> any`
//!
//! Returns the first non-null value from the argument list. If all values are null, returns null.
//!
//! ```text
//! coalesce(null, null, 'value')      // "value"
//! coalesce(name, username, 'Guest')  // first non-null field or "Guest"
//! coalesce(null, null)               // null
//! coalesce(`42`, `100`)              // 42 (first value)
//! ```
//!
//! ## JSON Functions
//!
//! ### `json_encode(value: any) -> string`
//!
//! Serializes any value to a JSON string.
//!
//! ```text
//! json_encode(`42`)                  // "42"
//! json_encode(`"hello"`)             // "\"hello\""
//! json_encode(`[1, 2, 3]`)           // "[1,2,3]"
//! json_encode(`{a: 1, b: 2}`)        // "{\"a\":1,\"b\":2}"
//! json_encode(null)                  // "null"
//! ```
//!
//! ### `json_decode(json: string) -> any`
//!
//! Parses a JSON string and returns the corresponding value. Throws an error if the string
//! is not valid JSON.
//!
//! ```text
//! json_decode(`"42"`)                // 42
//! json_decode(`"\"hello\""`)         // "hello"
//! json_decode(`"[1,2,3]"`)           // [1, 2, 3]
//! json_decode(`"{\"a\":1}"`)         // {a: 1}
//! json_decode(`"true"`)              // true
//! ```
//!
//! ### `json_pointer(value: any, pointer: string) -> any`
//!
//! Access a value using JSON Pointer syntax (RFC 6901). Returns null if the pointer
//! doesn't resolve to a value.
//!
//! ```text
//! json_pointer({foo: {bar: [1,2,3]}}, '/foo/bar/1')   // 2
//! json_pointer({a: {b: 1}}, '/a/b')                   // 1
//! json_pointer([1, 2, 3], '/0')                       // 1
//! json_pointer({}, '/missing')                        // null
//! json_pointer({'a/b': 1}, '/a~1b')                   // 1 (/ escaped as ~1)
//! json_pointer({'a~b': 1}, '/a~0b')                   // 1 (~ escaped as ~0)
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

        let result = expr.search(&Variable::String("value".to_string())).unwrap();
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
}
