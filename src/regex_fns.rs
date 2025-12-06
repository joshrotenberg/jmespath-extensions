//! Regular expression functions.
//!
//! These functions provide regex matching and manipulation.
//! Requires the `regex` feature.

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

use regex::Regex;

/// Register all regex functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("regex_match", Box::new(RegexMatchFn::new()));
    runtime.register_function("regex_extract", Box::new(RegexExtractFn::new()));
    runtime.register_function("regex_replace", Box::new(RegexReplaceFn::new()));
}

// =============================================================================
// regex_match(string, pattern) -> boolean
// =============================================================================

define_function!(
    RegexMatchFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for RegexMatchFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let pattern = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected pattern string".to_owned()),
            )
        })?;

        let re = Regex::new(pattern).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Invalid regex pattern".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Bool(re.is_match(input))))
    }
}

// =============================================================================
// regex_extract(string, pattern) -> array of matches
// =============================================================================

define_function!(
    RegexExtractFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for RegexExtractFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let pattern = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected pattern string".to_owned()),
            )
        })?;

        let re = Regex::new(pattern).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Invalid regex pattern".to_owned()),
            )
        })?;

        let matches: Vec<Rcvar> = re
            .find_iter(input)
            .map(|m| Rc::new(Variable::String(m.as_str().to_string())) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(matches)))
    }
}

// =============================================================================
// regex_replace(string, pattern, replacement) -> string
// =============================================================================

define_function!(
    RegexReplaceFn,
    vec![
        ArgumentType::String,
        ArgumentType::String,
        ArgumentType::String
    ],
    None
);

impl Function for RegexReplaceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let pattern = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected pattern string".to_owned()),
            )
        })?;

        let replacement = args[2].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected replacement string".to_owned()),
            )
        })?;

        let re = Regex::new(pattern).map_err(|_| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Invalid regex pattern".to_owned()),
            )
        })?;

        let result = re.replace_all(input, replacement);
        Ok(Rc::new(Variable::String(result.into_owned())))
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
    fn test_regex_match() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_match(@, '^hello')").unwrap();

        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("world hello".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_regex_extract() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_extract(@, '[0-9]+')").unwrap();
        let data = Variable::String("abc123def456".to_string());
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "123");
        assert_eq!(arr[1].as_string().unwrap(), "456");
    }

    #[test]
    fn test_regex_replace() {
        let runtime = setup_runtime();
        let expr = runtime.compile("regex_replace(@, '[0-9]+', 'X')").unwrap();
        let data = Variable::String("abc123def456".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "abcXdefX");
    }
}
