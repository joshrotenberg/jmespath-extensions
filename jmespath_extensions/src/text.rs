//! Text analysis functions.
//!
//! This module provides text functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category text`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::text;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! text::register(&mut runtime);
//! ```

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Signature, Variable};

/// Register all text functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("word_count", Box::new(WordCountFn::new()));
    runtime.register_function("char_count", Box::new(CharCountFn::new()));
    runtime.register_function("sentence_count", Box::new(SentenceCountFn::new()));
    runtime.register_function("paragraph_count", Box::new(ParagraphCountFn::new()));
    runtime.register_function("reading_time", Box::new(ReadingTimeFn::new()));
    runtime.register_function(
        "reading_time_seconds",
        Box::new(ReadingTimeSecondsFn::new()),
    );
    runtime.register_function("char_frequencies", Box::new(CharFrequenciesFn::new()));
    runtime.register_function("word_frequencies", Box::new(WordFrequenciesFn::new()));
}

// Average reading speed in words per minute
const WORDS_PER_MINUTE: f64 = 200.0;

// =============================================================================
// word_count(s) -> number
// =============================================================================

pub struct WordCountFn {
    signature: Signature,
}

impl Default for WordCountFn {
    fn default() -> Self {
        Self::new()
    }
}

impl WordCountFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for WordCountFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let count = s.split_whitespace().count();
        Ok(Rc::new(Variable::Number(serde_json::Number::from(count))))
    }
}

// =============================================================================
// char_count(s) -> number
// =============================================================================

pub struct CharCountFn {
    signature: Signature,
}

impl Default for CharCountFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CharCountFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CharCountFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let count = s.chars().filter(|c| !c.is_whitespace()).count();
        Ok(Rc::new(Variable::Number(serde_json::Number::from(count))))
    }
}

// =============================================================================
// sentence_count(s) -> number
// =============================================================================

pub struct SentenceCountFn {
    signature: Signature,
}

impl Default for SentenceCountFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SentenceCountFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SentenceCountFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        if s.trim().is_empty() {
            return Ok(Rc::new(Variable::Number(serde_json::Number::from(0))));
        }

        // Count sentence-ending punctuation
        let count = s
            .chars()
            .filter(|c| *c == '.' || *c == '!' || *c == '?')
            .count();

        // If no sentence-ending punctuation but has content, count as 1
        let count = if count == 0 && !s.trim().is_empty() {
            1
        } else {
            count
        };

        Ok(Rc::new(Variable::Number(serde_json::Number::from(count))))
    }
}

// =============================================================================
// paragraph_count(s) -> number
// =============================================================================

pub struct ParagraphCountFn {
    signature: Signature,
}

impl Default for ParagraphCountFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ParagraphCountFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for ParagraphCountFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        // Split by double newlines (paragraph separator)
        let count = s.split("\n\n").filter(|p| !p.trim().is_empty()).count();

        Ok(Rc::new(Variable::Number(serde_json::Number::from(count))))
    }
}

// =============================================================================
// reading_time(s) -> number (minutes)
// =============================================================================

pub struct ReadingTimeFn {
    signature: Signature,
}

impl Default for ReadingTimeFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadingTimeFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for ReadingTimeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let word_count = s.split_whitespace().count() as f64;
        let minutes = (word_count / WORDS_PER_MINUTE).ceil();
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(minutes).unwrap(),
        )))
    }
}

// =============================================================================
// reading_time_seconds(s) -> number (seconds)
// =============================================================================

pub struct ReadingTimeSecondsFn {
    signature: Signature,
}

impl Default for ReadingTimeSecondsFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadingTimeSecondsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for ReadingTimeSecondsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let word_count = s.split_whitespace().count() as f64;
        let seconds = (word_count / WORDS_PER_MINUTE) * 60.0;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(seconds.ceil()).unwrap(),
        )))
    }
}

// =============================================================================
// char_frequencies(s) -> object
// =============================================================================

pub struct CharFrequenciesFn {
    signature: Signature,
}

impl Default for CharFrequenciesFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CharFrequenciesFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CharFrequenciesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let mut freq: BTreeMap<char, usize> = BTreeMap::new();
        for c in s.chars() {
            if !c.is_whitespace() {
                *freq.entry(c).or_insert(0) += 1;
            }
        }

        let obj: serde_json::Map<String, serde_json::Value> = freq
            .into_iter()
            .map(|(k, v)| (k.to_string(), serde_json::Value::Number(v.into())))
            .collect();

        Ok(Rc::new(
            Variable::from_json(&serde_json::to_string(&obj).unwrap()).unwrap(),
        ))
    }
}

// =============================================================================
// word_frequencies(s) -> object
// =============================================================================

pub struct WordFrequenciesFn {
    signature: Signature,
}

impl Default for WordFrequenciesFn {
    fn default() -> Self {
        Self::new()
    }
}

impl WordFrequenciesFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for WordFrequenciesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let mut freq: BTreeMap<String, usize> = BTreeMap::new();
        for word in s.split_whitespace() {
            // Normalize: lowercase and remove punctuation
            let normalized: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            if !normalized.is_empty() {
                *freq.entry(normalized).or_insert(0) += 1;
            }
        }

        let obj: serde_json::Map<String, serde_json::Value> = freq
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect();

        Ok(Rc::new(
            Variable::from_json(&serde_json::to_string(&obj).unwrap()).unwrap(),
        ))
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

    #[test]
    fn test_word_count() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello world, this is a test.""#).unwrap();
        let expr = runtime.compile("word_count(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_word_count_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#""""#).unwrap();
        let expr = runtime.compile("word_count(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_char_count() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello world""#).unwrap();
        let expr = runtime.compile("char_count(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // "Hello world" without space = 10 characters
        assert_eq!(result.as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_sentence_count() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello world. How are you? I am fine!""#).unwrap();
        let expr = runtime.compile("sentence_count(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_sentence_count_no_punctuation() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello world""#).unwrap();
        let expr = runtime.compile("sentence_count(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_paragraph_count() {
        let runtime = setup();
        let data =
            Variable::from_json(r#""First paragraph.\n\nSecond paragraph.\n\nThird.""#).unwrap();
        let expr = runtime.compile("paragraph_count(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_reading_time() {
        let runtime = setup();
        // 200 words = 1 minute at 200 wpm
        let words: Vec<&str> = vec!["word"; 200];
        let text = words.join(" ");
        let data = Variable::String(text);
        let expr = runtime.compile("reading_time(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_reading_time_short() {
        let runtime = setup();
        let data = Variable::from_json(r#""Quick read""#).unwrap();
        let expr = runtime.compile("reading_time(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0); // Rounds up to 1 minute
    }

    #[test]
    fn test_reading_time_seconds() {
        let runtime = setup();
        // 100 words = 30 seconds at 200 wpm
        let words: Vec<&str> = vec!["word"; 100];
        let text = words.join(" ");
        let data = Variable::String(text);
        let expr = runtime.compile("reading_time_seconds(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 30.0);
    }

    #[test]
    fn test_char_frequencies() {
        let runtime = setup();
        let data = Variable::from_json(r#""aab""#).unwrap();
        let expr = runtime.compile("char_frequencies(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 2.0);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_word_frequencies() {
        let runtime = setup();
        let data = Variable::from_json(r#""hello world hello""#).unwrap();
        let expr = runtime.compile("word_frequencies(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("hello").unwrap().as_number().unwrap(), 2.0);
        assert_eq!(obj.get("world").unwrap().as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_word_frequencies_normalized() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, HELLO hello!""#).unwrap();
        let expr = runtime.compile("word_frequencies(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        // All normalized to "hello"
        assert_eq!(obj.get("hello").unwrap().as_number().unwrap(), 3.0);
    }
}
