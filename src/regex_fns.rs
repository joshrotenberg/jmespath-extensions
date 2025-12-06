//! Regular expression functions.
//!
//! This module provides regular expression pattern matching and text manipulation capabilities
//! for JMESPath expressions. It includes functions for testing patterns, extracting matches,
//! and replacing matched text with new values.
//!
//! **Note:** This module requires the `regex` feature to be enabled.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description |
//! |----------|-----------|---------|-------------|
//! | `regex_match` | `(text: string, pattern: string)` | `boolean` | Test if pattern matches text |
//! | `regex_extract` | `(text: string, pattern: string)` | `array` | Extract all pattern matches |
//! | `regex_replace` | `(text: string, pattern: string, replacement: string)` | `string` | Replace matches with text |
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
//! let expr = runtime.compile("regex_match(@, '^[0-9]+$')").unwrap();
//! let data = jmespath::Variable::String("12345".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_boolean().unwrap(), true);
//! ```
//!
//! # Function Details
//!
//! ## Pattern Matching
//!
//! ### `regex_match(text: string, pattern: string) -> boolean`
//!
//! Tests whether a regular expression pattern matches anywhere in the input text.
//! Returns true if the pattern is found, false otherwise.
//!
//! ```text
//! regex_match('hello world', '^hello')     // true (starts with "hello")
//! regex_match('hello world', 'world$')     // true (ends with "world")
//! regex_match('test123', '[0-9]+')         // true (contains digits)
//! regex_match('hello', '^[a-z]+$')         // true (all lowercase letters)
//! regex_match('Hello', '^[a-z]+$')         // false (contains uppercase)
//! regex_match('email@example.com', '@')    // true (contains @)
//! ```
//!
//! ## Text Extraction
//!
//! ### `regex_extract(text: string, pattern: string) -> array`
//!
//! Extracts all non-overlapping matches of a pattern from the input text and returns them
//! as an array of strings. If no matches are found, returns an empty array.
//!
//! ```text
//! regex_extract('abc123def456', '[0-9]+')
//! // ["123", "456"]
//!
//! regex_extract('hello world', '\w+')
//! // ["hello", "world"]
//!
//! regex_extract('test@example.com', '[a-z]+')
//! // ["test", "example", "com"]
//!
//! regex_extract('no numbers here', '[0-9]+')
//! // []
//!
//! regex_extract('one1two2three3', '[a-z]+')
//! // ["one", "two", "three"]
//! ```
//!
//! ## Text Replacement
//!
//! ### `regex_replace(text: string, pattern: string, replacement: string) -> string`
//!
//! Replaces all occurrences of a pattern in the input text with the replacement string.
//! The replacement can include capture group references using `$1`, `$2`, etc.
//!
//! ```text
//! regex_replace('hello world', 'world', 'universe')
//! // "hello universe"
//!
//! regex_replace('abc123def456', '[0-9]+', 'X')
//! // "abcXdefX"
//!
//! regex_replace('  extra   spaces  ', '\s+', ' ')
//! // " extra spaces "
//!
//! regex_replace('name@example.com', '@.*', '@redacted.com')
//! // "name@redacted.com"
//!
//! regex_replace('test-2024-01-15', '([0-9]{4})-([0-9]{2})-([0-9]{2})', '$3/$2/$1')
//! // "test-15/01/2024" (reorder date parts)
//!
//! regex_replace('CamelCase', '([A-Z])', '_$1')
//! // "_Camel_Case"
//! ```
//!
//! ## Pattern Syntax
//!
//! The regex functions support standard Rust regex syntax, which is similar to Perl-style
//! regular expressions. Common patterns include:
//!
//! - `.` - Any character except newline
//! - `^` - Start of string
//! - `$` - End of string
//! - `*` - Zero or more repetitions
//! - `+` - One or more repetitions
//! - `?` - Zero or one repetition
//! - `[abc]` - Character class (a, b, or c)
//! - `[^abc]` - Negated character class (not a, b, or c)
//! - `\d` - Digit (same as `[0-9]`)
//! - `\w` - Word character (alphanumeric + underscore)
//! - `\s` - Whitespace character
//! - `(...)` - Capture group
//! - `|` - Alternation (or)

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
