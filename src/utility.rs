//! Utility and conditional functions.
//!
//! These functions provide utility operations like timestamps and conditionals.

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

        let var = Variable::from_json(s)
            .map_err(|e| JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e)))?;

        Ok(Rc::new(var))
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
}
