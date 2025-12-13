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
    runtime.register_function("ngrams", Box::new(NgramsFn::new()));
    runtime.register_function("bigrams", Box::new(BigramsFn::new()));
    runtime.register_function("trigrams", Box::new(TrigramsFn::new()));
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

// =============================================================================
// ngrams(s, n, type?) -> array
// Generate n-grams from text. Type can be "word" (default) or "char".
// =============================================================================

pub struct NgramsFn {
    signature: Signature,
}

impl Default for NgramsFn {
    fn default() -> Self {
        Self::new()
    }
}

impl NgramsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![ArgumentType::String, ArgumentType::Number],
                Some(ArgumentType::String),
            ),
        }
    }
}

impl Function for NgramsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let n = args[1].as_number().unwrap() as usize;

        // Default to "word" if not specified
        let ngram_type = if args.len() > 2 {
            args[2].as_string().map(|s| s.as_str()).unwrap_or("word")
        } else {
            "word"
        };

        if n == 0 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let result = match ngram_type {
            "char" => {
                // Character n-grams
                let chars: Vec<char> = s.chars().collect();
                if chars.len() < n {
                    vec![]
                } else {
                    chars
                        .windows(n)
                        .map(|w| Rc::new(Variable::String(w.iter().collect())))
                        .collect()
                }
            }
            _ => {
                // Word n-grams (default)
                let words: Vec<&str> = s.split_whitespace().collect();
                if words.len() < n {
                    vec![]
                } else {
                    words
                        .windows(n)
                        .map(|w| {
                            let arr: Vec<Rcvar> = w
                                .iter()
                                .map(|word| Rc::new(Variable::String(word.to_string())))
                                .collect();
                            Rc::new(Variable::Array(arr))
                        })
                        .collect()
                }
            }
        };

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// bigrams(s) -> array
// Convenience function for word bigrams (2-grams).
// =============================================================================

pub struct BigramsFn {
    signature: Signature,
}

impl Default for BigramsFn {
    fn default() -> Self {
        Self::new()
    }
}

impl BigramsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for BigramsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() < 2 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let result: Vec<Rcvar> = words
            .windows(2)
            .map(|w| {
                let arr: Vec<Rcvar> = w
                    .iter()
                    .map(|word| Rc::new(Variable::String(word.to_string())))
                    .collect();
                Rc::new(Variable::Array(arr))
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// trigrams(s) -> array
// Convenience function for word trigrams (3-grams).
// =============================================================================

pub struct TrigramsFn {
    signature: Signature,
}

impl Default for TrigramsFn {
    fn default() -> Self {
        Self::new()
    }
}

impl TrigramsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for TrigramsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() < 3 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let result: Vec<Rcvar> = words
            .windows(3)
            .map(|w| {
                let arr: Vec<Rcvar> = w
                    .iter()
                    .map(|word| Rc::new(Variable::String(word.to_string())))
                    .collect();
                Rc::new(Variable::Array(arr))
            })
            .collect();

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

    #[test]
    fn test_ngrams_char() {
        let runtime = setup();
        let data = Variable::from_json(r#""hello""#).unwrap();
        let expr = runtime.compile("ngrams(@, `3`, 'char')").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "hel");
        assert_eq!(arr[1].as_string().unwrap(), "ell");
        assert_eq!(arr[2].as_string().unwrap(), "llo");
    }

    #[test]
    fn test_ngrams_word() {
        let runtime = setup();
        let data = Variable::from_json(r#""the quick brown fox""#).unwrap();
        let expr = runtime.compile("ngrams(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // Each element is an array of words
        let first = arr[0].as_array().unwrap();
        assert_eq!(first[0].as_string().unwrap(), "the");
        assert_eq!(first[1].as_string().unwrap(), "quick");
    }

    #[test]
    fn test_ngrams_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#""hi""#).unwrap();
        // Asking for 3-grams from a 2-char string
        let expr = runtime.compile("ngrams(@, `3`, 'char')").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_bigrams() {
        let runtime = setup();
        let data = Variable::from_json(r#""a b c d""#).unwrap();
        let expr = runtime.compile("bigrams(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // [["a", "b"], ["b", "c"], ["c", "d"]]
        let first = arr[0].as_array().unwrap();
        assert_eq!(first[0].as_string().unwrap(), "a");
        assert_eq!(first[1].as_string().unwrap(), "b");
        let last = arr[2].as_array().unwrap();
        assert_eq!(last[0].as_string().unwrap(), "c");
        assert_eq!(last[1].as_string().unwrap(), "d");
    }

    #[test]
    fn test_bigrams_single_word() {
        let runtime = setup();
        let data = Variable::from_json(r#""hello""#).unwrap();
        let expr = runtime.compile("bigrams(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_trigrams() {
        let runtime = setup();
        let data = Variable::from_json(r#""a b c d e""#).unwrap();
        let expr = runtime.compile("trigrams(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // [["a", "b", "c"], ["b", "c", "d"], ["c", "d", "e"]]
        let first = arr[0].as_array().unwrap();
        assert_eq!(first[0].as_string().unwrap(), "a");
        assert_eq!(first[1].as_string().unwrap(), "b");
        assert_eq!(first[2].as_string().unwrap(), "c");
    }

    #[test]
    fn test_trigrams_too_short() {
        let runtime = setup();
        let data = Variable::from_json(r#""a b""#).unwrap();
        let expr = runtime.compile("trigrams(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }
}
