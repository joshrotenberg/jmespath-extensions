//! Multi-pattern matching functions.
//!
//! This module provides multi_match functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category multi_match`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::multi_match;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! multi_match::register(&mut runtime);
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
    runtime.register_function("extract_all", Box::new(ExtractAllFn::new()));
    runtime.register_function("match_positions", Box::new(MatchPositionsFn::new()));
    runtime.register_function("tokenize", Box::new(TokenizeFn::new()));
    runtime.register_function("extract_between", Box::new(ExtractBetweenFn::new()));
    runtime.register_function("split_keep", Box::new(SplitKeepFn::new()));
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

// extract_all(string, patterns) -> array of matches with pattern info
// Returns all matches with pattern index and matched text
define_function!(
    ExtractAllFn,
    vec![ArgumentType::String, ArgumentType::Array],
    None
);

impl Function for ExtractAllFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr
            .iter()
            .filter_map(|p| p.as_string().map(|s| s.as_str()))
            .collect();

        if patterns.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let mut results: Vec<Rcvar> = Vec::new();

        for mat in ac.find_iter(text) {
            let mut obj = std::collections::BTreeMap::new();
            obj.insert(
                "pattern".to_string(),
                Rc::new(Variable::String(
                    patterns[mat.pattern().as_usize()].to_string(),
                )),
            );
            obj.insert(
                "match".to_string(),
                Rc::new(Variable::String(text[mat.start()..mat.end()].to_string())),
            );
            obj.insert(
                "start".to_string(),
                Rc::new(Variable::Number(serde_json::Number::from(mat.start()))),
            );
            obj.insert(
                "end".to_string(),
                Rc::new(Variable::Number(serde_json::Number::from(mat.end()))),
            );
            results.push(Rc::new(Variable::Object(obj)));
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// match_positions(string, patterns) -> array of positions
// Returns start/end positions of all matches
define_function!(
    MatchPositionsFn,
    vec![ArgumentType::String, ArgumentType::Array],
    None
);

impl Function for MatchPositionsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let patterns_arr = args[1].as_array().unwrap();

        let patterns: Vec<&str> = patterns_arr
            .iter()
            .filter_map(|p| p.as_string().map(|s| s.as_str()))
            .collect();

        if patterns.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let ac = AhoCorasick::new(&patterns).unwrap();
        let mut results: Vec<Rcvar> = Vec::new();

        for mat in ac.find_iter(text) {
            let mut obj = std::collections::BTreeMap::new();
            obj.insert(
                "pattern".to_string(),
                Rc::new(Variable::String(
                    patterns[mat.pattern().as_usize()].to_string(),
                )),
            );
            obj.insert(
                "start".to_string(),
                Rc::new(Variable::Number(serde_json::Number::from(mat.start()))),
            );
            obj.insert(
                "end".to_string(),
                Rc::new(Variable::Number(serde_json::Number::from(mat.end()))),
            );
            results.push(Rc::new(Variable::Object(obj)));
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// tokenize(string, options?) -> array of tokens
// Smart word tokenization with optional configuration
define_function!(
    TokenizeFn,
    vec![ArgumentType::String],
    Some(ArgumentType::Any)
);

impl Function for TokenizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();

        // Parse options if provided
        let lowercase = args
            .get(1)
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("lowercase"))
            .and_then(|v| v.as_boolean())
            .unwrap_or(false);

        let min_length = args
            .get(1)
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("min_length"))
            .and_then(|v| v.as_number())
            .map(|n| n as usize)
            .unwrap_or(1);

        // Split on non-alphanumeric characters
        let tokens: Vec<Rcvar> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty() && s.len() >= min_length)
            .map(|s| {
                let token = if lowercase {
                    s.to_lowercase()
                } else {
                    s.to_string()
                };
                Rc::new(Variable::String(token)) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(tokens)))
    }
}

// extract_between(string, start, end) -> string or null
// Extract text between two delimiters
define_function!(
    ExtractBetweenFn,
    vec![
        ArgumentType::String,
        ArgumentType::String,
        ArgumentType::String
    ],
    None
);

impl Function for ExtractBetweenFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let start_delim = args[1].as_string().unwrap();
        let end_delim = args[2].as_string().unwrap();

        if let Some(start_pos) = text.find(start_delim) {
            let after_start = start_pos + start_delim.len();
            if let Some(end_pos) = text[after_start..].find(end_delim) {
                let extracted = &text[after_start..after_start + end_pos];
                return Ok(Rc::new(Variable::String(extracted.to_string())));
            }
        }

        Ok(Rc::new(Variable::Null))
    }
}

// split_keep(string, delimiter) -> array keeping delimiters
// Split string but keep the delimiters in the result
define_function!(
    SplitKeepFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for SplitKeepFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let text = args[0].as_string().unwrap();
        let delimiter = args[1].as_string().unwrap();

        if delimiter.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![Rc::new(Variable::String(
                text.to_string(),
            ))])));
        }

        let mut result: Vec<Rcvar> = Vec::new();
        let mut last_end = 0;

        for (start, part) in text.match_indices(delimiter) {
            if start > last_end {
                result.push(Rc::new(Variable::String(text[last_end..start].to_string())));
            }
            result.push(Rc::new(Variable::String(part.to_string())));
            last_end = start + part.len();
        }

        if last_end < text.len() {
            result.push(Rc::new(Variable::String(text[last_end..].to_string())));
        }

        Ok(Rc::new(Variable::Array(result)))
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
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_match_any_not_found() {
        let runtime = setup();
        let data = Variable::String("everything is fine".to_string());
        let expr = runtime
            .compile("match_any(@, ['error', 'warning', 'critical'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_match_any_empty_patterns() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "some text", "patterns": []}"#).unwrap();
        let expr = runtime.compile("match_any(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_match_any_multiple_matches() {
        let runtime = setup();
        let data = Variable::String("error and warning detected".to_string());
        let expr = runtime
            .compile("match_any(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
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
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_match_all_some_missing() {
        let runtime = setup();
        let data = Variable::String("error detected".to_string());
        let expr = runtime
            .compile("match_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_match_all_empty_patterns() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"text": "some text", "patterns": []}"#).unwrap();
        let expr = runtime.compile("match_all(text, patterns)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
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

    // extract_all tests

    #[test]
    fn test_extract_all_basic() {
        let runtime = setup();
        let data = Variable::String("error and warning detected".to_string());
        let expr = runtime
            .compile("extract_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_object().unwrap();
        assert_eq!(first.get("match").unwrap().as_string().unwrap(), "error");
        assert!(first.get("start").is_some());
        assert!(first.get("end").is_some());
    }

    #[test]
    fn test_extract_all_empty() {
        let runtime = setup();
        let data = Variable::String("no matches here".to_string());
        let expr = runtime
            .compile("extract_all(@, ['error', 'warning'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // match_positions tests

    #[test]
    fn test_match_positions_basic() {
        let runtime = setup();
        let data = Variable::String("The quick brown fox".to_string());
        let expr = runtime
            .compile("match_positions(@, ['quick', 'fox'])")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_object().unwrap();
        assert_eq!(first.get("pattern").unwrap().as_string().unwrap(), "quick");
        assert_eq!(first.get("start").unwrap().as_number().unwrap() as i64, 4);
        assert_eq!(first.get("end").unwrap().as_number().unwrap() as i64, 9);
    }

    // tokenize tests

    #[test]
    fn test_tokenize_basic() {
        let runtime = setup();
        let data = Variable::String("Hello, world! This is a test.".to_string());
        let expr = runtime.compile("tokenize(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.len() >= 6);
        assert_eq!(arr[0].as_string().unwrap(), "Hello");
    }

    #[test]
    fn test_tokenize_with_options() {
        let runtime = setup();
        let data = Variable::String("Hello, world! A test.".to_string());
        let expr = runtime
            .compile("tokenize(@, {lowercase: `true`, min_length: `2`})")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should have: hello, world, test (not "A" due to min_length)
        let tokens: Vec<String> = arr
            .iter()
            .map(|v| v.as_string().unwrap().to_string())
            .collect();
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(!tokens.iter().any(|t| t.len() < 2));
    }

    // extract_between tests

    #[test]
    fn test_extract_between_basic() {
        let runtime = setup();
        let data = Variable::String("<title>Page Title</title>".to_string());
        let expr = runtime
            .compile("extract_between(@, '<title>', '</title>')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Page Title");
    }

    #[test]
    fn test_extract_between_not_found() {
        let runtime = setup();
        let data = Variable::String("no delimiters here".to_string());
        let expr = runtime
            .compile("extract_between(@, '<start>', '<end>')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    // split_keep tests

    #[test]
    fn test_split_keep_basic() {
        let runtime = setup();
        let data = Variable::String("a-b-c".to_string());
        let expr = runtime.compile("split_keep(@, '-')").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_string().unwrap(), "a");
        assert_eq!(arr[1].as_string().unwrap(), "-");
        assert_eq!(arr[2].as_string().unwrap(), "b");
    }

    #[test]
    fn test_split_keep_no_delimiter() {
        let runtime = setup();
        let data = Variable::String("no delimiters".to_string());
        let expr = runtime.compile("split_keep(@, '-')").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_string().unwrap(), "no delimiters");
    }
}
