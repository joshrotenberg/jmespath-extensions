//! Type conversion and checking functions.
//!
//! This module provides comprehensive type introspection, conversion, and validation capabilities
//! for JMESPath expressions. It includes functions to convert between types, check type identity,
//! and validate data characteristics like emptiness and JSON validity.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description |
//! |----------|-----------|---------|-------------|
//! | `to_string` | `(any)` | `string` | Convert any value to string representation |
//! | `to_number` | `(any)` | `number\|null` | Convert value to number, or null if invalid |
//! | `to_boolean` | `(any)` | `boolean` | Convert value to boolean (truthiness) |
//! | `type_of` | `(any)` | `string` | Get type name of value |
//! | `is_string` | `(any)` | `boolean` | Check if value is a string |
//! | `is_number` | `(any)` | `boolean` | Check if value is a number |
//! | `is_boolean` | `(any)` | `boolean` | Check if value is a boolean |
//! | `is_array` | `(any)` | `boolean` | Check if value is an array |
//! | `is_object` | `(any)` | `boolean` | Check if value is an object |
//! | `is_null` | `(any)` | `boolean` | Check if value is null |
//! | `is_empty` | `(any)` | `boolean` | Check if string/array/object is empty |
//! | `is_blank` | `(any)` | `boolean` | Check if string is empty or whitespace-only |
//! | `is_json` | `(string)` | `boolean` | Check if string is valid JSON |
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
//! let expr = runtime.compile("type_of(@)").unwrap();
//! let data = jmespath::Variable::String("hello".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(), "string");
//! ```
//!
//! # Function Details
//!
//! ## Type Conversion Functions
//!
//! ### `to_string(value: any) -> string`
//!
//! Converts any value to its string representation.
//!
//! ```text
//! to_string(`42`)           // "42"
//! to_string(`true`)         // "true"
//! to_string(`null`)         // "null"
//! to_string(`[1, 2, 3]`)    // "[1,2,3]"
//! ```
//!
//! ### `to_number(value: any) -> number|null`
//!
//! Converts a value to a number. Returns null if conversion fails.
//!
//! ```text
//! to_number(`"42"`)         // 42
//! to_number(`"3.14"`)       // 3.14
//! to_number(`true`)         // 1
//! to_number(`false`)        // 0
//! to_number(`"abc"`)        // null
//! ```
//!
//! ### `to_boolean(value: any) -> boolean`
//!
//! Converts a value to boolean based on truthiness rules.
//!
//! ```text
//! to_boolean(`null`)        // false
//! to_boolean(`""`)          // false
//! to_boolean(`0`)           // false
//! to_boolean(`[]`)          // false
//! to_boolean(`{}`)          // false
//! to_boolean(`"hello"`)     // true
//! to_boolean(`42`)          // true
//! ```
//!
//! ## Type Checking Functions
//!
//! ### `type_of(value: any) -> string`
//!
//! Returns the type name of a value.
//!
//! ```text
//! type_of(`"hello"`)        // "string"
//! type_of(`42`)             // "number"
//! type_of(`true`)           // "boolean"
//! type_of(`null`)           // "null"
//! type_of(`[]`)             // "array"
//! type_of(`{}`)             // "object"
//! ```
//!
//! ### `is_string(value: any) -> boolean`
//!
//! Checks if value is a string type.
//!
//! ```text
//! is_string(`"hello"`)      // true
//! is_string(`42`)           // false
//! ```
//!
//! ### `is_number(value: any) -> boolean`
//!
//! Checks if value is a number type.
//!
//! ```text
//! is_number(`42`)           // true
//! is_number(`"42"`)         // false
//! ```
//!
//! ### `is_boolean(value: any) -> boolean`
//!
//! Checks if value is a boolean type.
//!
//! ```text
//! is_boolean(`true`)        // true
//! is_boolean(`1`)           // false
//! ```
//!
//! ### `is_array(value: any) -> boolean`
//!
//! Checks if value is an array type.
//!
//! ```text
//! is_array(`[1, 2, 3]`)     // true
//! is_array(`{}`)            // false
//! ```
//!
//! ### `is_object(value: any) -> boolean`
//!
//! Checks if value is an object type.
//!
//! ```text
//! is_object(`{a: 1}`)       // true
//! is_object(`[]`)           // false
//! ```
//!
//! ### `is_null(value: any) -> boolean`
//!
//! Checks if value is null.
//!
//! ```text
//! is_null(`null`)           // true
//! is_null(`""`)             // false
//! ```
//!
//! ## Content Validation Functions
//!
//! ### `is_empty(value: any) -> boolean`
//!
//! Checks if a string, array, or object is empty. Null values are considered empty.
//!
//! ```text
//! is_empty(`""`)            // true
//! is_empty(`[]`)            // true
//! is_empty(`{}`)            // true
//! is_empty(`null`)          // true
//! is_empty(`"hello"`)       // false
//! is_empty(`[1, 2]`)        // false
//! ```
//!
//! ### `is_blank(value: any) -> boolean`
//!
//! Checks if a string is empty or contains only whitespace. Null values are considered blank.
//!
//! ```text
//! is_blank(`""`)            // true
//! is_blank(`"   "`)         // true
//! is_blank(`" \t\n "`)      // true
//! is_blank(`null`)          // true
//! is_blank(`"hello"`)       // false
//! ```
//!
//! ### `is_json(value: string) -> boolean`
//!
//! Checks if a string contains valid JSON.
//!
//! ```text
//! is_json(`"{\"a\":1}"`)    // true
//! is_json(`"[1,2,3]"`)      // true
//! is_json(`"42"`)           // true
//! is_json(`"not json"`)     // false
//! is_json(`"{bad}"`)        // false
//! ```

use std::rc::Rc;

use crate::common::{ArgumentType, Context, Function, JmespathError, Rcvar, Runtime, Variable};
use crate::define_function;

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
}
