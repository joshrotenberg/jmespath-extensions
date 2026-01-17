//! Type checking and conversion functions.
//!
//! This module provides type_conv functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category type_conv`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::type_conv;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! type_conv::register(&mut runtime);
//! ```

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::common::{ArgumentType, Context, Function, JmespathError, Rcvar, Runtime, Variable};
use crate::define_function;
use crate::register_if_enabled;

/// Register all type functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("to_string", Box::new(ToStringFn::new()));
    runtime.register_function("to_number", Box::new(ToNumberFn::new()));
    runtime.register_function("to_boolean", Box::new(ToBooleanFn::new()));
    runtime.register_function("type_of", Box::new(TypeOfFn::new()));
    runtime.register_function("is_string", Box::new(IsStringFn::new()));
    runtime.register_function("is_number", Box::new(IsNumberFn::new()));
    runtime.register_function("is_boolean", Box::new(IsBooleanFn::new()));
    runtime.register_function("is_array", Box::new(IsArrayFn::new()));
    runtime.register_function("is_object", Box::new(IsObjectFn::new()));
    runtime.register_function("is_null", Box::new(IsNullFn::new()));
    runtime.register_function("is_empty", Box::new(IsEmptyFn::new()));
    runtime.register_function("is_blank", Box::new(IsBlankFn::new()));
    runtime.register_function("is_json", Box::new(IsJsonFn::new()));
    runtime.register_function("parse_numbers", Box::new(ParseNumbersFn::new()));
    runtime.register_function("parse_booleans", Box::new(ParseBooleansFn::new()));
    runtime.register_function("parse_nulls", Box::new(ParseNullsFn::new()));
    runtime.register_function("auto_parse", Box::new(AutoParseFn::new()));
}

/// Register type functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled!(runtime, enabled, "to_string", Box::new(ToStringFn::new()));
    register_if_enabled!(runtime, enabled, "to_number", Box::new(ToNumberFn::new()));
    register_if_enabled!(runtime, enabled, "to_boolean", Box::new(ToBooleanFn::new()));
    register_if_enabled!(runtime, enabled, "type_of", Box::new(TypeOfFn::new()));
    register_if_enabled!(runtime, enabled, "is_string", Box::new(IsStringFn::new()));
    register_if_enabled!(runtime, enabled, "is_number", Box::new(IsNumberFn::new()));
    register_if_enabled!(runtime, enabled, "is_boolean", Box::new(IsBooleanFn::new()));
    register_if_enabled!(runtime, enabled, "is_array", Box::new(IsArrayFn::new()));
    register_if_enabled!(runtime, enabled, "is_object", Box::new(IsObjectFn::new()));
    register_if_enabled!(runtime, enabled, "is_null", Box::new(IsNullFn::new()));
    register_if_enabled!(runtime, enabled, "is_empty", Box::new(IsEmptyFn::new()));
    register_if_enabled!(runtime, enabled, "is_blank", Box::new(IsBlankFn::new()));
    register_if_enabled!(runtime, enabled, "is_json", Box::new(IsJsonFn::new()));
    register_if_enabled!(
        runtime,
        enabled,
        "parse_numbers",
        Box::new(ParseNumbersFn::new())
    );
    register_if_enabled!(
        runtime,
        enabled,
        "parse_booleans",
        Box::new(ParseBooleansFn::new())
    );
    register_if_enabled!(
        runtime,
        enabled,
        "parse_nulls",
        Box::new(ParseNullsFn::new())
    );
    register_if_enabled!(runtime, enabled, "auto_parse", Box::new(AutoParseFn::new()));
}

// =============================================================================
// to_string(any) -> string
// =============================================================================

define_function!(ToStringFn, vec![ArgumentType::Any], None);

impl Function for ToStringFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let result = match &*args[0] {
            Variable::String(s) => s.clone(),
            Variable::Number(n) => n.to_string(),
            Variable::Bool(b) => b.to_string(),
            Variable::Null => "null".to_string(),
            _ => serde_json::to_string(&*args[0]).unwrap_or_else(|_| "null".to_string()),
        };

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// to_number(any) -> number
// =============================================================================

define_function!(ToNumberFn, vec![ArgumentType::Any], None);

impl Function for ToNumberFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let result = match &*args[0] {
            Variable::Number(n) => Some(n.clone()),
            Variable::String(s) => s.parse::<f64>().ok().and_then(serde_json::Number::from_f64),
            Variable::Bool(b) => Some(serde_json::Number::from(if *b { 1 } else { 0 })),
            _ => None,
        };

        match result {
            Some(n) => Ok(Rc::new(Variable::Number(n))),
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// to_boolean(any) -> boolean
// =============================================================================

define_function!(ToBooleanFn, vec![ArgumentType::Any], None);

impl Function for ToBooleanFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let result = match &*args[0] {
            Variable::Bool(b) => *b,
            Variable::Null => false,
            Variable::String(s) => !s.is_empty(),
            Variable::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
            Variable::Array(a) => !a.is_empty(),
            Variable::Object(o) => !o.is_empty(),
            Variable::Expref(_) => true,
        };

        Ok(Rc::new(Variable::Bool(result)))
    }
}

// =============================================================================
// type_of(any) -> string
// =============================================================================

define_function!(TypeOfFn, vec![ArgumentType::Any], None);

impl Function for TypeOfFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let type_name = match &*args[0] {
            Variable::String(_) => "string",
            Variable::Number(_) => "number",
            Variable::Bool(_) => "boolean",
            Variable::Null => "null",
            Variable::Array(_) => "array",
            Variable::Object(_) => "object",
            Variable::Expref(_) => "expref",
        };

        Ok(Rc::new(Variable::String(type_name.to_string())))
    }
}

// =============================================================================
// is_string(any) -> boolean
// =============================================================================

define_function!(IsStringFn, vec![ArgumentType::Any], None);

impl Function for IsStringFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(Variable::Bool(args[0].is_string())))
    }
}

// =============================================================================
// is_number(any) -> boolean
// =============================================================================

define_function!(IsNumberFn, vec![ArgumentType::Any], None);

impl Function for IsNumberFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(Variable::Bool(matches!(
            &*args[0],
            Variable::Number(_)
        ))))
    }
}

// =============================================================================
// is_boolean(any) -> boolean
// =============================================================================

define_function!(IsBooleanFn, vec![ArgumentType::Any], None);

impl Function for IsBooleanFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(Variable::Bool(matches!(
            &*args[0],
            Variable::Bool(_)
        ))))
    }
}

// =============================================================================
// is_array(any) -> boolean
// =============================================================================

define_function!(IsArrayFn, vec![ArgumentType::Any], None);

impl Function for IsArrayFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(Variable::Bool(args[0].is_array())))
    }
}

// =============================================================================
// is_object(any) -> boolean
// =============================================================================

define_function!(IsObjectFn, vec![ArgumentType::Any], None);

impl Function for IsObjectFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(Variable::Bool(args[0].is_object())))
    }
}

// =============================================================================
// is_null(any) -> boolean
// =============================================================================

define_function!(IsNullFn, vec![ArgumentType::Any], None);

impl Function for IsNullFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(Variable::Bool(args[0].is_null())))
    }
}

// =============================================================================
// is_empty(any) -> boolean (empty string, array, or object)
// =============================================================================

define_function!(IsEmptyFn, vec![ArgumentType::Any], None);

impl Function for IsEmptyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let is_empty = match &*args[0] {
            Variable::String(s) => s.is_empty(),
            Variable::Array(a) => a.is_empty(),
            Variable::Object(o) => o.is_empty(),
            Variable::Null => true,
            _ => false,
        };

        Ok(Rc::new(Variable::Bool(is_empty)))
    }
}

// =============================================================================
// is_blank(string) -> boolean (empty or whitespace only)
// =============================================================================

define_function!(IsBlankFn, vec![ArgumentType::Any], None);

impl Function for IsBlankFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        match &*args[0] {
            Variable::String(s) => Ok(Rc::new(Variable::Bool(s.trim().is_empty()))),
            Variable::Null => Ok(Rc::new(Variable::Bool(true))),
            // Return null for non-string types
            _ => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// is_json(any) -> boolean|null (valid JSON string)
// =============================================================================

define_function!(IsJsonFn, vec![ArgumentType::Any], None);

impl Function for IsJsonFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        // Return null for non-string types
        let s = match args[0].as_string() {
            Some(s) => s,
            None => return Ok(Rc::new(Variable::Null)),
        };

        let is_valid = serde_json::from_str::<serde_json::Value>(s).is_ok();

        Ok(Rc::new(Variable::Bool(is_valid)))
    }
}

// =============================================================================
// parse_numbers(any) -> any (recursively convert numeric strings to numbers)
// =============================================================================

define_function!(ParseNumbersFn, vec![ArgumentType::Any], None);

impl Function for ParseNumbersFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(parse_numbers_recursive(&args[0])))
    }
}

fn parse_numbers_recursive(value: &Variable) -> Variable {
    match value {
        Variable::String(s) => {
            // Try to parse as number - be conservative, only pure numeric strings
            let trimmed = s.trim();
            if let Ok(n) = trimmed.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(n) {
                    return Variable::Number(num);
                }
            }
            value.clone()
        }
        Variable::Object(obj) => {
            let parsed: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| (k.clone(), Rc::new(parse_numbers_recursive(v))))
                .collect();
            Variable::Object(parsed)
        }
        Variable::Array(arr) => {
            let parsed: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(parse_numbers_recursive(v)))
                .collect();
            Variable::Array(parsed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// parse_booleans(any) -> any (recursively convert boolean strings to booleans)
// =============================================================================

define_function!(ParseBooleansFn, vec![ArgumentType::Any], None);

impl Function for ParseBooleansFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(parse_booleans_recursive(&args[0])))
    }
}

fn parse_booleans_recursive(value: &Variable) -> Variable {
    match value {
        Variable::String(s) => {
            let lower = s.trim().to_lowercase();
            match lower.as_str() {
                "true" | "yes" | "on" | "1" => Variable::Bool(true),
                "false" | "no" | "off" | "0" => Variable::Bool(false),
                _ => value.clone(),
            }
        }
        Variable::Object(obj) => {
            let parsed: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| (k.clone(), Rc::new(parse_booleans_recursive(v))))
                .collect();
            Variable::Object(parsed)
        }
        Variable::Array(arr) => {
            let parsed: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(parse_booleans_recursive(v)))
                .collect();
            Variable::Array(parsed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// parse_nulls(any) -> any (recursively convert null-like strings to null)
// =============================================================================

define_function!(ParseNullsFn, vec![ArgumentType::Any], None);

impl Function for ParseNullsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(parse_nulls_recursive(&args[0])))
    }
}

fn parse_nulls_recursive(value: &Variable) -> Variable {
    match value {
        Variable::String(s) => {
            let lower = s.trim().to_lowercase();
            match lower.as_str() {
                "null" | "none" | "nil" | "undefined" => Variable::Null,
                _ => value.clone(),
            }
        }
        Variable::Object(obj) => {
            let parsed: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| (k.clone(), Rc::new(parse_nulls_recursive(v))))
                .collect();
            Variable::Object(parsed)
        }
        Variable::Array(arr) => {
            let parsed: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(parse_nulls_recursive(v)))
                .collect();
            Variable::Array(parsed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// auto_parse(any) -> any (intelligently parse numbers, booleans, and nulls)
// =============================================================================

define_function!(AutoParseFn, vec![ArgumentType::Any], None);

impl Function for AutoParseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(auto_parse_recursive(&args[0])))
    }
}

fn auto_parse_recursive(value: &Variable) -> Variable {
    match value {
        Variable::String(s) => {
            let trimmed = s.trim();
            let lower = trimmed.to_lowercase();

            // Try null-like values first
            if matches!(lower.as_str(), "null" | "none" | "nil" | "undefined") {
                return Variable::Null;
            }

            // Try boolean values
            if matches!(lower.as_str(), "true" | "yes" | "on") {
                return Variable::Bool(true);
            }
            if matches!(lower.as_str(), "false" | "no" | "off") {
                return Variable::Bool(false);
            }

            // Try numeric values (but not "0" or "1" which might be intended as booleans in some contexts)
            // Actually, let's parse all numeric strings as numbers
            if let Ok(n) = trimmed.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(n) {
                    return Variable::Number(num);
                }
            }

            value.clone()
        }
        Variable::Object(obj) => {
            let parsed: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| (k.clone(), Rc::new(auto_parse_recursive(v))))
                .collect();
            Variable::Object(parsed)
        }
        Variable::Array(arr) => {
            let parsed: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(auto_parse_recursive(v)))
                .collect();
            Variable::Array(parsed)
        }
        _ => value.clone(),
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
    fn test_type_of() {
        let runtime = setup_runtime();
        let expr = runtime.compile("type_of(@)").unwrap();

        let result = expr.search(Variable::String("hello".to_string())).unwrap();
        assert_eq!(result.as_string().unwrap(), "string");

        let result = expr
            .search(Variable::Number(serde_json::Number::from(42)))
            .unwrap();
        assert_eq!(result.as_string().unwrap(), "number");
    }

    #[test]
    fn test_is_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_empty(@)").unwrap();

        let result = expr.search(Variable::String("".to_string())).unwrap();
        assert!(result.as_boolean().unwrap());

        let result = expr.search(Variable::String("hello".to_string())).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    // =========================================================================
    // parse_numbers tests
    // =========================================================================

    #[test]
    fn test_parse_numbers_basic() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"count": "42", "price": "19.99", "name": "test"}"#).unwrap();
        let expr = runtime.compile("parse_numbers(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("count").unwrap().as_number().unwrap() as i64, 42);
        assert!((obj.get("price").unwrap().as_number().unwrap() - 19.99).abs() < 0.01);
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "test");
    }

    #[test]
    fn test_parse_numbers_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"outer": {"inner": "123"}}"#).unwrap();
        let expr = runtime.compile("parse_numbers(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let inner = obj.get("outer").unwrap().as_object().unwrap();
        assert_eq!(inner.get("inner").unwrap().as_number().unwrap() as i64, 123);
    }

    #[test]
    fn test_parse_numbers_non_numeric() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"val": "42abc"}"#).unwrap();
        let expr = runtime.compile("parse_numbers(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        // Should remain as string since it's not a pure number
        assert_eq!(obj.get("val").unwrap().as_string().unwrap(), "42abc");
    }

    // =========================================================================
    // parse_booleans tests
    // =========================================================================

    #[test]
    fn test_parse_booleans_basic() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"active": "true", "verified": "false", "name": "test"}"#)
                .unwrap();
        let expr = runtime.compile("parse_booleans(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("active").unwrap().as_boolean().unwrap());
        assert!(!obj.get("verified").unwrap().as_boolean().unwrap());
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "test");
    }

    #[test]
    fn test_parse_booleans_variants() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"a": "YES", "b": "no", "c": "ON", "d": "off", "e": "1", "f": "0"}"#,
        )
        .unwrap();
        let expr = runtime.compile("parse_booleans(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("a").unwrap().as_boolean().unwrap());
        assert!(!obj.get("b").unwrap().as_boolean().unwrap());
        assert!(obj.get("c").unwrap().as_boolean().unwrap());
        assert!(!obj.get("d").unwrap().as_boolean().unwrap());
        assert!(obj.get("e").unwrap().as_boolean().unwrap());
        assert!(!obj.get("f").unwrap().as_boolean().unwrap());
    }

    // =========================================================================
    // parse_nulls tests
    // =========================================================================

    #[test]
    fn test_parse_nulls_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"a": "null", "b": "NULL", "c": "None", "d": "nil", "e": "hello"}"#,
        )
        .unwrap();
        let expr = runtime.compile("parse_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("a").unwrap().is_null());
        assert!(obj.get("b").unwrap().is_null());
        assert!(obj.get("c").unwrap().is_null());
        assert!(obj.get("d").unwrap().is_null());
        assert_eq!(obj.get("e").unwrap().as_string().unwrap(), "hello");
    }

    // =========================================================================
    // auto_parse tests
    // =========================================================================

    #[test]
    fn test_auto_parse_mixed() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"num": "42", "bool": "true", "nil": "null", "str": "hello"}"#)
                .unwrap();
        let expr = runtime.compile("auto_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("num").unwrap().as_number().unwrap() as i64, 42);
        assert!(obj.get("bool").unwrap().as_boolean().unwrap());
        assert!(obj.get("nil").unwrap().is_null());
        assert_eq!(obj.get("str").unwrap().as_string().unwrap(), "hello");
    }

    #[test]
    fn test_auto_parse_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["42", "true", "null", "hello"]"#).unwrap();
        let expr = runtime.compile("auto_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_number().unwrap() as i64, 42);
        assert!(arr[1].as_boolean().unwrap());
        assert!(arr[2].is_null());
        assert_eq!(arr[3].as_string().unwrap(), "hello");
    }
}
