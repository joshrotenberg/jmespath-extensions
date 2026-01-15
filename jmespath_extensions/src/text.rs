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
    runtime.register_function("tokens", Box::new(TokensFn::new()));
    runtime.register_function("tokenize", Box::new(TokenizeFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("stem", Box::new(StemFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("stems", Box::new(StemsFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("stopwords", Box::new(StopwordsFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("remove_stopwords", Box::new(RemoveStopwordsFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("is_stopword", Box::new(IsStopwordFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("normalize_unicode", Box::new(NormalizeUnicodeFn::new()));
    #[cfg(feature = "text")]
    runtime.register_function("remove_accents", Box::new(RemoveAccentsFn::new()));
    runtime.register_function("collapse_whitespace", Box::new(CollapseWhitespaceFn::new()));
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

// =============================================================================
// tokens(s) -> array
// Simple word tokenization: lowercase, strip punctuation
// =============================================================================

pub struct TokensFn {
    signature: Signature,
}

impl Default for TokensFn {
    fn default() -> Self {
        Self::new()
    }
}

impl TokensFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for TokensFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let tokens: Vec<Rcvar> = s
            .split_whitespace()
            .filter_map(|word| {
                let normalized: String = word
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase();

                if normalized.is_empty() {
                    None
                } else {
                    Some(Rc::new(Variable::String(normalized)))
                }
            })
            .collect();

        Ok(Rc::new(Variable::Array(tokens)))
    }
}

// =============================================================================
// tokenize(s, options?) -> array
// Configurable tokenization with options:
//   - case: "lower" (default), "upper", "preserve"
//   - punctuation: "strip" (default), "keep"
// =============================================================================

pub struct TokenizeFn {
    signature: Signature,
}

impl Default for TokenizeFn {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenizeFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], Some(ArgumentType::Object)),
        }
    }
}

impl Function for TokenizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        // Parse options
        let (case_mode, strip_punctuation) = if args.len() > 1 {
            if let Some(opts) = args[1].as_object() {
                let case_mode = opts
                    .get("case")
                    .and_then(|v| v.as_string())
                    .map(|s| s.as_str())
                    .unwrap_or("lower");

                let punctuation = opts
                    .get("punctuation")
                    .and_then(|v| v.as_string())
                    .map(|s| s.as_str())
                    .unwrap_or("strip");

                (case_mode.to_string(), punctuation != "keep")
            } else {
                ("lower".to_string(), true)
            }
        } else {
            ("lower".to_string(), true)
        };

        let tokens: Vec<Rcvar> = s
            .split_whitespace()
            .filter_map(|word| {
                let processed: String = if strip_punctuation {
                    word.chars().filter(|c| c.is_alphanumeric()).collect()
                } else {
                    word.to_string()
                };

                if processed.is_empty() {
                    return None;
                }

                let final_token = match case_mode.as_str() {
                    "upper" => processed.to_uppercase(),
                    "preserve" => processed,
                    _ => processed.to_lowercase(), // "lower" or default
                };

                Some(Rc::new(Variable::String(final_token)))
            })
            .collect();

        Ok(Rc::new(Variable::Array(tokens)))
    }
}

// =============================================================================
// stem(word, lang?) -> string
// Stem a single word using Porter/Snowball stemmer
// =============================================================================

#[cfg(feature = "text")]
pub struct StemFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for StemFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl StemFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], Some(ArgumentType::String)),
        }
    }
}

#[cfg(feature = "text")]
impl Function for StemFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use rust_stemmers::{Algorithm, Stemmer};

        self.signature.validate(args, ctx)?;
        let word = args[0].as_string().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_string().map(|s| s.to_string())
        } else {
            None
        };

        let algorithm = match lang.as_deref() {
            Some("ar" | "arabic") => Algorithm::Arabic,
            Some("da" | "danish") => Algorithm::Danish,
            Some("nl" | "dutch") => Algorithm::Dutch,
            Some("fi" | "finnish") => Algorithm::Finnish,
            Some("fr" | "french") => Algorithm::French,
            Some("de" | "german") => Algorithm::German,
            Some("el" | "greek") => Algorithm::Greek,
            Some("hu" | "hungarian") => Algorithm::Hungarian,
            Some("it" | "italian") => Algorithm::Italian,
            Some("no" | "norwegian") => Algorithm::Norwegian,
            Some("pt" | "portuguese") => Algorithm::Portuguese,
            Some("ro" | "romanian") => Algorithm::Romanian,
            Some("ru" | "russian") => Algorithm::Russian,
            Some("es" | "spanish") => Algorithm::Spanish,
            Some("sv" | "swedish") => Algorithm::Swedish,
            Some("ta" | "tamil") => Algorithm::Tamil,
            Some("tr" | "turkish") => Algorithm::Turkish,
            _ => Algorithm::English, // Default to English
        };

        let stemmer = Stemmer::create(algorithm);
        let stemmed = stemmer.stem(word).to_string();

        Ok(Rc::new(Variable::String(stemmed)))
    }
}

// =============================================================================
// stems(tokens, lang?) -> array
// Stem an array of tokens
// =============================================================================

#[cfg(feature = "text")]
pub struct StemsFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for StemsFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl StemsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::Array], Some(ArgumentType::String)),
        }
    }
}

#[cfg(feature = "text")]
impl Function for StemsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use rust_stemmers::{Algorithm, Stemmer};

        self.signature.validate(args, ctx)?;
        let tokens = args[0].as_array().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_string().map(|s| s.to_string())
        } else {
            None
        };

        let algorithm = match lang.as_deref() {
            Some("ar" | "arabic") => Algorithm::Arabic,
            Some("da" | "danish") => Algorithm::Danish,
            Some("nl" | "dutch") => Algorithm::Dutch,
            Some("fi" | "finnish") => Algorithm::Finnish,
            Some("fr" | "french") => Algorithm::French,
            Some("de" | "german") => Algorithm::German,
            Some("el" | "greek") => Algorithm::Greek,
            Some("hu" | "hungarian") => Algorithm::Hungarian,
            Some("it" | "italian") => Algorithm::Italian,
            Some("no" | "norwegian") => Algorithm::Norwegian,
            Some("pt" | "portuguese") => Algorithm::Portuguese,
            Some("ro" | "romanian") => Algorithm::Romanian,
            Some("ru" | "russian") => Algorithm::Russian,
            Some("es" | "spanish") => Algorithm::Spanish,
            Some("sv" | "swedish") => Algorithm::Swedish,
            Some("ta" | "tamil") => Algorithm::Tamil,
            Some("tr" | "turkish") => Algorithm::Turkish,
            _ => Algorithm::English,
        };

        let stemmer = Stemmer::create(algorithm);

        let result: Vec<Rcvar> = tokens
            .iter()
            .filter_map(|t| {
                t.as_string()
                    .map(|s| Rc::new(Variable::String(stemmer.stem(s).to_string())))
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// stopwords(lang?) -> array
// Get stopwords list for a language
// =============================================================================

#[cfg(feature = "text")]
pub struct StopwordsFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for StopwordsFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl StopwordsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![], Some(ArgumentType::String)),
        }
    }
}

#[cfg(feature = "text")]
impl Function for StopwordsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use stop_words::{LANGUAGE, get};

        self.signature.validate(args, ctx)?;

        let lang = if !args.is_empty() {
            args[0].as_string().map(|s| s.to_string())
        } else {
            None
        };

        let language = match lang.as_deref() {
            Some("ar" | "arabic") => LANGUAGE::Arabic,
            Some("bg" | "bulgarian") => LANGUAGE::Bulgarian,
            Some("ca" | "catalan") => LANGUAGE::Catalan,
            Some("cs" | "czech") => LANGUAGE::Czech,
            Some("da" | "danish") => LANGUAGE::Danish,
            Some("nl" | "dutch") => LANGUAGE::Dutch,
            Some("fi" | "finnish") => LANGUAGE::Finnish,
            Some("fr" | "french") => LANGUAGE::French,
            Some("de" | "german") => LANGUAGE::German,
            Some("he" | "hebrew") => LANGUAGE::Hebrew,
            Some("hi" | "hindi") => LANGUAGE::Hindi,
            Some("hu" | "hungarian") => LANGUAGE::Hungarian,
            Some("id" | "indonesian") => LANGUAGE::Indonesian,
            Some("it" | "italian") => LANGUAGE::Italian,
            Some("ja" | "japanese") => LANGUAGE::Japanese,
            Some("ko" | "korean") => LANGUAGE::Korean,
            Some("lv" | "latvian") => LANGUAGE::Latvian,
            Some("no" | "norwegian") => LANGUAGE::Norwegian,
            Some("fa" | "persian") => LANGUAGE::Persian,
            Some("pl" | "polish") => LANGUAGE::Polish,
            Some("pt" | "portuguese") => LANGUAGE::Portuguese,
            Some("ro" | "romanian") => LANGUAGE::Romanian,
            Some("ru" | "russian") => LANGUAGE::Russian,
            Some("sk" | "slovak") => LANGUAGE::Slovak,
            Some("es" | "spanish") => LANGUAGE::Spanish,
            Some("sv" | "swedish") => LANGUAGE::Swedish,
            Some("th" | "thai") => LANGUAGE::Thai,
            Some("tr" | "turkish") => LANGUAGE::Turkish,
            Some("uk" | "ukrainian") => LANGUAGE::Ukrainian,
            Some("vi" | "vietnamese") => LANGUAGE::Vietnamese,
            Some("zh" | "chinese") => LANGUAGE::Chinese,
            _ => LANGUAGE::English,
        };

        let words = get(language);
        let result: Vec<Rcvar> = words
            .iter()
            .map(|w| Rc::new(Variable::String(w.to_string())))
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// remove_stopwords(tokens, lang?) -> array
// Remove stopwords from token array
// =============================================================================

#[cfg(feature = "text")]
pub struct RemoveStopwordsFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for RemoveStopwordsFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl RemoveStopwordsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::Array], Some(ArgumentType::String)),
        }
    }
}

#[cfg(feature = "text")]
impl Function for RemoveStopwordsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use stop_words::{LANGUAGE, get};

        self.signature.validate(args, ctx)?;
        let tokens = args[0].as_array().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_string().map(|s| s.to_string())
        } else {
            None
        };

        let language = match lang.as_deref() {
            Some("ar" | "arabic") => LANGUAGE::Arabic,
            Some("bg" | "bulgarian") => LANGUAGE::Bulgarian,
            Some("ca" | "catalan") => LANGUAGE::Catalan,
            Some("cs" | "czech") => LANGUAGE::Czech,
            Some("da" | "danish") => LANGUAGE::Danish,
            Some("nl" | "dutch") => LANGUAGE::Dutch,
            Some("fi" | "finnish") => LANGUAGE::Finnish,
            Some("fr" | "french") => LANGUAGE::French,
            Some("de" | "german") => LANGUAGE::German,
            Some("he" | "hebrew") => LANGUAGE::Hebrew,
            Some("hi" | "hindi") => LANGUAGE::Hindi,
            Some("hu" | "hungarian") => LANGUAGE::Hungarian,
            Some("id" | "indonesian") => LANGUAGE::Indonesian,
            Some("it" | "italian") => LANGUAGE::Italian,
            Some("ja" | "japanese") => LANGUAGE::Japanese,
            Some("ko" | "korean") => LANGUAGE::Korean,
            Some("lv" | "latvian") => LANGUAGE::Latvian,
            Some("no" | "norwegian") => LANGUAGE::Norwegian,
            Some("fa" | "persian") => LANGUAGE::Persian,
            Some("pl" | "polish") => LANGUAGE::Polish,
            Some("pt" | "portuguese") => LANGUAGE::Portuguese,
            Some("ro" | "romanian") => LANGUAGE::Romanian,
            Some("ru" | "russian") => LANGUAGE::Russian,
            Some("sk" | "slovak") => LANGUAGE::Slovak,
            Some("es" | "spanish") => LANGUAGE::Spanish,
            Some("sv" | "swedish") => LANGUAGE::Swedish,
            Some("th" | "thai") => LANGUAGE::Thai,
            Some("tr" | "turkish") => LANGUAGE::Turkish,
            Some("uk" | "ukrainian") => LANGUAGE::Ukrainian,
            Some("vi" | "vietnamese") => LANGUAGE::Vietnamese,
            Some("zh" | "chinese") => LANGUAGE::Chinese,
            _ => LANGUAGE::English,
        };

        let stopwords = get(language);
        let stopwords_set: std::collections::HashSet<String> =
            stopwords.iter().map(|s| s.to_string()).collect();

        let result: Vec<Rcvar> = tokens
            .iter()
            .filter_map(|t| {
                t.as_string().and_then(|s| {
                    if stopwords_set.contains(&s.to_lowercase()) {
                        None
                    } else {
                        Some(Rc::new(Variable::String(s.to_string())))
                    }
                })
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// is_stopword(word, lang?) -> boolean
// Check if word is a stopword
// =============================================================================

#[cfg(feature = "text")]
pub struct IsStopwordFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for IsStopwordFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl IsStopwordFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], Some(ArgumentType::String)),
        }
    }
}

#[cfg(feature = "text")]
impl Function for IsStopwordFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use stop_words::{LANGUAGE, get};

        self.signature.validate(args, ctx)?;
        let word = args[0].as_string().unwrap();

        let lang = if args.len() > 1 {
            args[1].as_string().map(|s| s.to_string())
        } else {
            None
        };

        let language = match lang.as_deref() {
            Some("ar" | "arabic") => LANGUAGE::Arabic,
            Some("bg" | "bulgarian") => LANGUAGE::Bulgarian,
            Some("ca" | "catalan") => LANGUAGE::Catalan,
            Some("cs" | "czech") => LANGUAGE::Czech,
            Some("da" | "danish") => LANGUAGE::Danish,
            Some("nl" | "dutch") => LANGUAGE::Dutch,
            Some("fi" | "finnish") => LANGUAGE::Finnish,
            Some("fr" | "french") => LANGUAGE::French,
            Some("de" | "german") => LANGUAGE::German,
            Some("he" | "hebrew") => LANGUAGE::Hebrew,
            Some("hi" | "hindi") => LANGUAGE::Hindi,
            Some("hu" | "hungarian") => LANGUAGE::Hungarian,
            Some("id" | "indonesian") => LANGUAGE::Indonesian,
            Some("it" | "italian") => LANGUAGE::Italian,
            Some("ja" | "japanese") => LANGUAGE::Japanese,
            Some("ko" | "korean") => LANGUAGE::Korean,
            Some("lv" | "latvian") => LANGUAGE::Latvian,
            Some("no" | "norwegian") => LANGUAGE::Norwegian,
            Some("fa" | "persian") => LANGUAGE::Persian,
            Some("pl" | "polish") => LANGUAGE::Polish,
            Some("pt" | "portuguese") => LANGUAGE::Portuguese,
            Some("ro" | "romanian") => LANGUAGE::Romanian,
            Some("ru" | "russian") => LANGUAGE::Russian,
            Some("sk" | "slovak") => LANGUAGE::Slovak,
            Some("es" | "spanish") => LANGUAGE::Spanish,
            Some("sv" | "swedish") => LANGUAGE::Swedish,
            Some("th" | "thai") => LANGUAGE::Thai,
            Some("tr" | "turkish") => LANGUAGE::Turkish,
            Some("uk" | "ukrainian") => LANGUAGE::Ukrainian,
            Some("vi" | "vietnamese") => LANGUAGE::Vietnamese,
            Some("zh" | "chinese") => LANGUAGE::Chinese,
            _ => LANGUAGE::English,
        };

        let stopwords = get(language);
        let is_stop = stopwords.iter().any(|sw| sw.eq_ignore_ascii_case(word));

        Ok(Rc::new(Variable::Bool(is_stop)))
    }
}

// =============================================================================
// normalize_unicode(s, form?) -> string
// Unicode normalization (NFC, NFD, NFKC, NFKD)
// =============================================================================

#[cfg(feature = "text")]
pub struct NormalizeUnicodeFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for NormalizeUnicodeFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl NormalizeUnicodeFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], Some(ArgumentType::String)),
        }
    }
}

#[cfg(feature = "text")]
impl Function for NormalizeUnicodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use unicode_normalization::UnicodeNormalization;

        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let form = if args.len() > 1 {
            args[1].as_string().map(|s| s.to_uppercase())
        } else {
            None
        };

        let normalized = match form.as_deref() {
            Some("NFD") => s.nfd().collect::<String>(),
            Some("NFKC") => s.nfkc().collect::<String>(),
            Some("NFKD") => s.nfkd().collect::<String>(),
            _ => s.nfc().collect::<String>(), // Default to NFC
        };

        Ok(Rc::new(Variable::String(normalized)))
    }
}

// =============================================================================
// remove_accents(s) -> string
// Strip diacritics/accents from text
// =============================================================================

#[cfg(feature = "text")]
pub struct RemoveAccentsFn {
    signature: Signature,
}

#[cfg(feature = "text")]
impl Default for RemoveAccentsFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "text")]
impl RemoveAccentsFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

#[cfg(feature = "text")]
impl Function for RemoveAccentsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use unicode_normalization::UnicodeNormalization;

        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        // NFD normalize then filter out combining marks (diacritics)
        let result: String = s
            .nfd()
            .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
            .collect();

        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// collapse_whitespace(s) -> string
// Normalize whitespace (multiple spaces -> single, trim)
// =============================================================================

pub struct CollapseWhitespaceFn {
    signature: Signature,
}

impl Default for CollapseWhitespaceFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CollapseWhitespaceFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CollapseWhitespaceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        let result: String = s.split_whitespace().collect::<Vec<_>>().join(" ");

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

    // =========================================================================
    // tokens tests
    // =========================================================================

    #[test]
    fn test_tokens_basic() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, World!""#).unwrap();
        let expr = runtime.compile("tokens(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "hello");
        assert_eq!(arr[1].as_string().unwrap(), "world");
    }

    #[test]
    fn test_tokens_punctuation_only() {
        let runtime = setup();
        let data = Variable::from_json(r#""... --- !!!""#).unwrap();
        let expr = runtime.compile("tokens(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_tokens_mixed() {
        let runtime = setup();
        let data = Variable::from_json(r#""The quick, brown fox!""#).unwrap();
        let expr = runtime.compile("tokens(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_string().unwrap(), "the");
        assert_eq!(arr[1].as_string().unwrap(), "quick");
        assert_eq!(arr[2].as_string().unwrap(), "brown");
        assert_eq!(arr[3].as_string().unwrap(), "fox");
    }

    #[test]
    fn test_tokens_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#""""#).unwrap();
        let expr = runtime.compile("tokens(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // tokenize tests
    // =========================================================================

    #[test]
    fn test_tokenize_default() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, World!""#).unwrap();
        let expr = runtime.compile("tokenize(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "hello");
        assert_eq!(arr[1].as_string().unwrap(), "world");
    }

    #[test]
    fn test_tokenize_preserve_case() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, World!""#).unwrap();
        let expr = runtime
            .compile(r#"tokenize(@, `{"case": "preserve"}`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "Hello");
        assert_eq!(arr[1].as_string().unwrap(), "World");
    }

    #[test]
    fn test_tokenize_upper_case() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, World!""#).unwrap();
        let expr = runtime
            .compile(r#"tokenize(@, `{"case": "upper"}`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "HELLO");
        assert_eq!(arr[1].as_string().unwrap(), "WORLD");
    }

    #[test]
    fn test_tokenize_keep_punctuation() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, World!""#).unwrap();
        let expr = runtime
            .compile(r#"tokenize(@, `{"punctuation": "keep"}`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "hello,");
        assert_eq!(arr[1].as_string().unwrap(), "world!");
    }

    #[test]
    fn test_tokenize_preserve_case_keep_punctuation() {
        let runtime = setup();
        let data = Variable::from_json(r#""Hello, World!""#).unwrap();
        let expr = runtime
            .compile(r#"tokenize(@, `{"case": "preserve", "punctuation": "keep"}`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "Hello,");
        assert_eq!(arr[1].as_string().unwrap(), "World!");
    }

    // =========================================================================
    // stem tests
    // =========================================================================

    #[test]
    #[cfg(feature = "text")]
    fn test_stem_basic() {
        let runtime = setup();
        let data = Variable::from_json(r#""running""#).unwrap();
        let expr = runtime.compile("stem(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "run");
    }

    #[test]
    #[cfg(feature = "text")]
    fn test_stem_plural() {
        let runtime = setup();
        let data = Variable::from_json(r#""cats""#).unwrap();
        let expr = runtime.compile("stem(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "cat");
    }

    #[test]
    #[cfg(feature = "text")]
    fn test_stems_array() {
        let runtime = setup();
        let data = Variable::from_json(r#"["running", "cats", "quickly"]"#).unwrap();
        let expr = runtime.compile("stems(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "run");
        assert_eq!(arr[1].as_string().unwrap(), "cat");
        assert_eq!(arr[2].as_string().unwrap(), "quick");
    }

    // =========================================================================
    // stopwords tests
    // =========================================================================

    #[test]
    #[cfg(feature = "text")]
    fn test_stopwords_english() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("stopwords()").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // English stopwords should include common words
        let words: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_string().map(|s| s.to_string()))
            .collect();
        assert!(words.contains(&"the".to_string()));
        assert!(words.contains(&"is".to_string()));
        assert!(words.contains(&"a".to_string()));
    }

    #[test]
    #[cfg(feature = "text")]
    fn test_remove_stopwords() {
        let runtime = setup();
        let data = Variable::from_json(r#"["the", "quick", "brown", "fox"]"#).unwrap();
        let expr = runtime.compile("remove_stopwords(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // "the" should be removed
        let words: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_string().map(|s| s.to_string()))
            .collect();
        assert!(!words.contains(&"the".to_string()));
        assert!(words.contains(&"quick".to_string()));
        assert!(words.contains(&"brown".to_string()));
        assert!(words.contains(&"fox".to_string()));
    }

    #[test]
    #[cfg(feature = "text")]
    fn test_is_stopword() {
        let runtime = setup();
        let data = Variable::from_json(r#""the""#).unwrap();
        let expr = runtime.compile("is_stopword(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::from_json(r#""elephant""#).unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    // =========================================================================
    // text normalization tests
    // =========================================================================

    #[test]
    #[cfg(feature = "text")]
    fn test_normalize_unicode_default() {
        let runtime = setup();
        // café with combining acute accent (e + ́)
        let data = Variable::from_json(r#""cafe\u0301""#).unwrap();
        let expr = runtime.compile("normalize_unicode(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // NFC should compose to é
        assert_eq!(result.as_string().unwrap(), "café");
    }

    #[test]
    #[cfg(feature = "text")]
    fn test_remove_accents() {
        let runtime = setup();
        let data = Variable::from_json(r#""café naïve résumé""#).unwrap();
        let expr = runtime.compile("remove_accents(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "cafe naive resume");
    }

    #[test]
    fn test_collapse_whitespace() {
        let runtime = setup();
        let data = Variable::from_json(r#""  hello   world  ""#).unwrap();
        let expr = runtime.compile("collapse_whitespace(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_collapse_whitespace_tabs_newlines() {
        let runtime = setup();
        let data = Variable::from_json(r#""hello\t\n\nworld""#).unwrap();
        let expr = runtime.compile("collapse_whitespace(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }
}
