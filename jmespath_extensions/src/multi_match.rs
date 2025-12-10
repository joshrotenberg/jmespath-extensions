//! Multi-pattern matching functions using Aho-Corasick algorithm.
//!
//! This module provides efficient multi-pattern string matching functions
//! using the [aho-corasick](https://docs.rs/aho-corasick) crate. These functions
//! can find multiple patterns in a single pass through the text.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `match_any(string, patterns)` | Returns true if any pattern matches |
//! | `match_all(string, patterns)` | Returns true if all patterns match |
//! | `match_which(string, patterns)` | Returns array of patterns that matched |
//! | `match_count(string, patterns)` | Count total matches across all patterns |
//! | `replace_many(string, replacements)` | Replace multiple patterns at once |
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "multi-match")]
//! # fn main() {
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::multi_match;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! multi_match::register(&mut runtime);
//!
//! // Check if any pattern matches
//! let expr = runtime.compile("match_any(@, ['error', 'warning'])").unwrap();
//! let data = Variable::String("an error occurred".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_boolean().unwrap(), true);
//! # }
//! # #[cfg(not(feature = "multi-match"))]
//! # fn main() {}
//! ```

use std::rc::Rc;

use aho_corasick::AhoCorasick;

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Variable, define_function};

/// Register all multi-match functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("match_any", Box::new(MatchAnyFn::new()));
    runtime.register_function("match_all", Box::new(MatchAllFn::new()));
    runtime.register_function("match_which", Box::new(MatchWhichFn::new()));
    runtime.register_function("match_count", Box::new(MatchCountFn::new()));
    runtime.register_function("replace_many", Box::new(ReplaceManyFn::new()));
}

// match_any(string, patterns) -> boolean
// Returns true if any of the patterns match the string
define_function!(
    MatchAnyFn,
    vec![ArgumentType::String, ArgumentType::Array],
    None
);

impl Function for MatchAnyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        // Extract pattern strings
        let patterns: Vec<&str> = patterns_arr
            .iter()
            .filter_map(|p| p.as_string().map(|s| s.as_str()))
            .collect();

        if patterns.is_empty() {
            return Ok(Rc::new(Variable::Bool(false)));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let has_match = ac.find(text).is_some();

        Ok(Rc::new(Variable::Bool(has_match)))
    }
}

// match_all(string, patterns) -> boolean
// Returns true if all patterns match the string
define_function!(
    MatchAllFn,
    vec![ArgumentType::String, ArgumentType::Array],
    None
);

impl Function for MatchAllFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        // Extract pattern strings
        let patterns: Vec<&str> = patterns_arr
            .iter()
            .filter_map(|p| p.as_string().map(|s| s.as_str()))
            .collect();

        if patterns.is_empty() {
            return Ok(Rc::new(Variable::Bool(true)));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();

        // Track which patterns have been found
        let mut found = vec![false; patterns.len()];

        for mat in ac.find_iter(text) {
            found[mat.pattern().as_usize()] = true;
        }

        let all_found = found.iter().all(|&f| f);
        Ok(Rc::new(Variable::Bool(all_found)))
    }
}

// match_which(string, patterns) -> array
// Returns array of patterns that matched
define_function!(
    MatchWhichFn,
    vec![ArgumentType::String, ArgumentType::Array],
    None
);

impl Function for MatchWhichFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        // Extract pattern strings
        let patterns: Vec<&str> = patterns_arr
            .iter()
            .filter_map(|p| p.as_string().map(|s| s.as_str()))
            .collect();

        if patterns.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();

        // Track which patterns have been found
        let mut found = vec![false; patterns.len()];

        for mat in ac.find_iter(text) {
            found[mat.pattern().as_usize()] = true;
        }

        // Collect matched patterns
        let matched: Vec<Rcvar> = patterns
            .iter()
            .enumerate()
            .filter(|(i, _)| found[*i])
            .map(|(_, p)| Rc::new(Variable::String((*p).to_string())) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(matched)))
    }
}

// match_count(string, patterns) -> number
// Count total number of matches (non-overlapping) across all patterns
define_function!(
    MatchCountFn,
    vec![ArgumentType::String, ArgumentType::Array],
    None
);

impl Function for MatchCountFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        // Extract pattern strings
        let patterns: Vec<&str> = patterns_arr
            .iter()
            .filter_map(|p| p.as_string().map(|s| s.as_str()))
            .collect();

        if patterns.is_empty() {
            return Ok(Rc::new(Variable::Number(serde_json::Number::from(0))));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let count = ac.find_iter(text).count();

        Ok(Rc::new(Variable::Number(serde_json::Number::from(count))))
    }
}

// replace_many(string, replacements) -> string
// Replace multiple patterns at once. replacements is an object {pattern: replacement, ...}
define_function!(
    ReplaceManyFn,
    vec![ArgumentType::String, ArgumentType::Object],
    None
);

impl Function for ReplaceManyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let replacements_obj = args[1].as_object().unwrap();

        if replacements_obj.is_empty() {
            return Ok(Rc::new(Variable::String(text.to_string())));
        }

        // Extract patterns and their replacements
        let mut patterns: Vec<&str> = Vec::new();
        let mut replacements: Vec<String> = Vec::new();

        for (pattern, replacement) in replacements_obj.iter() {
            patterns.push(pattern);
            if let Some(s) = replacement.as_string() {
                replacements.push(s.to_string());
            } else {
                // Convert non-string values to string
                replacements.push(replacement.to_string());
            }
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let result = ac.replace_all(text, &replacements);

        Ok(Rc::new(Variable::String(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        runtime
    }

    // match_any tests

    #[test]
    fn test_match_any_found() {
        let runtime = setup();
        let data = Variable::String("an error occurred in the system".to_string());
        let expr = runtime
            .compile("match_any(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_match_any_not_found() {
        let runtime = setup();
        let data = Variable::String("everything is fine".to_string());
        let expr = runtime
            .compile("match_any(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_match_any_empty_patterns() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "some text", "patterns": []}"#).unwrap();
        let expr = runtime.compile("match_any(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_match_any_multiple_matches() {
        let runtime = setup();
        let data = Variable::String("error and warning detected".to_string());
        let expr = runtime
            .compile("match_any(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    // match_all tests

    #[test]
    fn test_match_all_all_found() {
        let runtime = setup();
        let data = Variable::String("error and warning detected".to_string());
        let expr = runtime
            .compile("match_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_match_all_some_missing() {
        let runtime = setup();
        let data = Variable::String("error detected".to_string());
        let expr = runtime
            .compile("match_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_match_all_empty_patterns() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "some text", "patterns": []}"#).unwrap();
        let expr = runtime.compile("match_all(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    // match_which tests

    #[test]
    fn test_match_which_some_found() {
        let runtime = setup();
        let data = Variable::String("error and warning detected".to_string());
        let expr = runtime
            .compile("match_which(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let strs: Vec<&str> = arr
            .iter()
            .map(|v| v.as_string().unwrap().as_str())
            .collect();
        assert!(strs.contains(&"error"));
        assert!(strs.contains(&"warning"));
    }

    #[test]
    fn test_match_which_none_found() {
        let runtime = setup();
        let data = Variable::String("everything is fine".to_string());
        let expr = runtime
            .compile("match_which(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_match_which_preserves_order() {
        let runtime = setup();
        let data = Variable::String("warning then error".to_string());
        let expr = runtime
            .compile("match_which(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should return in pattern order, not match order
        assert_eq!(arr[0].as_string().unwrap(), "error");
        assert_eq!(arr[1].as_string().unwrap(), "warning");
    }

    // match_count tests

    #[test]
    fn test_match_count_multiple() {
        let runtime = setup();
        let data = Variable::String("error error warning error".to_string());
        let expr = runtime
            .compile("match_count(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);
    }

    #[test]
    fn test_match_count_none() {
        let runtime = setup();
        let data = Variable::String("everything is fine".to_string());
        let expr = runtime
            .compile("match_count(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_match_count_empty_patterns() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "some text", "patterns": []}"#).unwrap();
        let expr = runtime.compile("match_count(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    // replace_many tests

    #[test]
    fn test_replace_many_basic() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "hello world"}"#).unwrap();
        let expr = runtime
            .compile("replace_many(text, {hello: 'hi', world: 'there'})")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hi there");
    }

    #[test]
    fn test_replace_many_no_matches() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "hello world"}"#).unwrap();
        let expr = runtime.compile("replace_many(text, {foo: 'bar'})").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_replace_many_empty_replacements() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "hello world", "replacements": {}}"#).unwrap();
        let expr = runtime.compile("replace_many(text, replacements)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_replace_many_multiple_occurrences() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "error: connection error"}"#).unwrap();
        let expr = runtime
            .compile("replace_many(text, {error: 'ERROR', connection: 'CONN'})")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "ERROR: CONN ERROR");
    }
}
