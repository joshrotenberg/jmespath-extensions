//! Phonetic encoding functions.
//!
//! This module provides phonetic functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category phonetic`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::phonetic;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! phonetic::register(&mut runtime);
//! ```

use std::rc::Rc;

use rphonetic::{
    Caverphone1, Caverphone2, Encoder, MatchRatingApproach, Metaphone, Nysiis, Soundex,
};

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Signature, Variable};

/// Register all phonetic functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("soundex", Box::new(SoundexFn::new()));
    runtime.register_function("metaphone", Box::new(MetaphoneFn::new()));
    runtime.register_function("double_metaphone", Box::new(DoubleMetaphoneFn::new()));
    runtime.register_function("nysiis", Box::new(NysiisFn::new()));
    runtime.register_function("match_rating_codex", Box::new(MatchRatingCodexFn::new()));
    runtime.register_function("caverphone", Box::new(CaverphoneFn::new()));
    runtime.register_function("caverphone2", Box::new(Caverphone2Fn::new()));
    runtime.register_function("sounds_like", Box::new(SoundsLikeFn::new()));
    runtime.register_function("phonetic_match", Box::new(PhoneticMatchFn::new()));
}

// =============================================================================
// soundex(string) -> string
// =============================================================================

pub struct SoundexFn {
    signature: Signature,
}

impl Default for SoundexFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundexFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SoundexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let soundex = Soundex::default();
        let result = soundex.encode(s);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// metaphone(string) -> string
// =============================================================================

pub struct MetaphoneFn {
    signature: Signature,
}

impl Default for MetaphoneFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaphoneFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for MetaphoneFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let metaphone = Metaphone::default();
        let result = metaphone.encode(s);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// double_metaphone(string) -> [primary, alternate]
// =============================================================================

pub struct DoubleMetaphoneFn {
    signature: Signature,
}

impl Default for DoubleMetaphoneFn {
    fn default() -> Self {
        Self::new()
    }
}

impl DoubleMetaphoneFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for DoubleMetaphoneFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let dm = rphonetic::DoubleMetaphone::default();
        let result = dm.double_metaphone(s);
        let primary = Rc::new(Variable::String(result.primary()));
        let alt = result.alternate();
        let alternate = if alt.is_empty() {
            Rc::new(Variable::Null)
        } else {
            Rc::new(Variable::String(alt))
        };
        Ok(Rc::new(Variable::Array(vec![primary, alternate])))
    }
}

// =============================================================================
// nysiis(string) -> string
// =============================================================================

pub struct NysiisFn {
    signature: Signature,
}

impl Default for NysiisFn {
    fn default() -> Self {
        Self::new()
    }
}

impl NysiisFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for NysiisFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let nysiis = Nysiis::default();
        let result = nysiis.encode(s);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// match_rating_codex(string) -> string
// =============================================================================

pub struct MatchRatingCodexFn {
    signature: Signature,
}

impl Default for MatchRatingCodexFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchRatingCodexFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for MatchRatingCodexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let mra = MatchRatingApproach;
        let result = mra.encode(s);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// caverphone(string) -> string
// =============================================================================

pub struct CaverphoneFn {
    signature: Signature,
}

impl Default for CaverphoneFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CaverphoneFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for CaverphoneFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let caverphone = Caverphone1;
        let result = caverphone.encode(s);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// caverphone2(string) -> string
// =============================================================================

pub struct Caverphone2Fn {
    signature: Signature,
}

impl Default for Caverphone2Fn {
    fn default() -> Self {
        Self::new()
    }
}

impl Caverphone2Fn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for Caverphone2Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let caverphone = Caverphone2;
        let result = caverphone.encode(s);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// sounds_like(s1, s2) -> bool
// =============================================================================

pub struct SoundsLikeFn {
    signature: Signature,
}

impl Default for SoundsLikeFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundsLikeFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::String], None),
        }
    }
}

impl Function for SoundsLikeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let soundex = Soundex::default();
        let result = soundex.is_encoded_equals(s1, s2);
        Ok(Rc::new(Variable::Bool(result)))
    }
}

// =============================================================================
// phonetic_match(s1, s2, algorithm?) -> bool
// =============================================================================

pub struct PhoneticMatchFn {
    signature: Signature,
}

impl Default for PhoneticMatchFn {
    fn default() -> Self {
        Self::new()
    }
}

impl PhoneticMatchFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![ArgumentType::String, ArgumentType::String],
                Some(ArgumentType::String),
            ),
        }
    }
}

impl Function for PhoneticMatchFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();

        let algorithm = if args.len() > 2 {
            args[2]
                .as_string()
                .map(|s| s.to_lowercase())
                .unwrap_or_else(|| "soundex".to_string())
        } else {
            "soundex".to_string()
        };

        let result = match algorithm.as_str() {
            "soundex" => {
                let encoder = Soundex::default();
                encoder.is_encoded_equals(s1, s2)
            }
            "metaphone" => {
                let encoder = Metaphone::default();
                encoder.encode(s1) == encoder.encode(s2)
            }
            "double_metaphone" | "doublemetaphone" => {
                let encoder = rphonetic::DoubleMetaphone::default();
                let r1 = encoder.double_metaphone(s1);
                let r2 = encoder.double_metaphone(s2);
                // Match if primary codes match, or if any combination matches
                r1.primary() == r2.primary()
                    || (!r1.alternate().is_empty() && r1.alternate() == r2.primary())
                    || (!r2.alternate().is_empty() && r2.alternate() == r1.primary())
                    || (!r1.alternate().is_empty()
                        && !r2.alternate().is_empty()
                        && r1.alternate() == r2.alternate())
            }
            "nysiis" => {
                let encoder = Nysiis::default();
                encoder.encode(s1) == encoder.encode(s2)
            }
            "match_rating" | "mra" => {
                let encoder = MatchRatingApproach;
                encoder.is_encoded_equals(s1, s2)
            }
            "caverphone" | "caverphone1" => {
                let encoder = Caverphone1;
                encoder.encode(s1) == encoder.encode(s2)
            }
            "caverphone2" => {
                let encoder = Caverphone2;
                encoder.encode(s1) == encoder.encode(s2)
            }
            _ => {
                // Default to soundex for unknown algorithms
                let encoder = Soundex::default();
                encoder.is_encoded_equals(s1, s2)
            }
        };

        Ok(Rc::new(Variable::Bool(result)))
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
    fn test_soundex() {
        let runtime = setup();
        let data = Variable::from_json(r#""Robert""#).unwrap();
        let expr = runtime.compile("soundex(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "R163");
    }

    #[test]
    fn test_soundex_similar_names() {
        let runtime = setup();
        // Robert and Rupert should have the same Soundex code
        let data = Variable::from_json(r#""Rupert""#).unwrap();
        let expr = runtime.compile("soundex(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "R163");
    }

    #[test]
    fn test_metaphone() {
        let runtime = setup();
        let data = Variable::from_json(r#""Smith""#).unwrap();
        let expr = runtime.compile("metaphone(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "SM0");
    }

    #[test]
    fn test_double_metaphone() {
        let runtime = setup();
        let data = Variable::from_json(r#""Schmidt""#).unwrap();
        let expr = runtime.compile("double_metaphone(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Primary encoding
        assert!(!arr[0].as_string().unwrap().is_empty());
    }

    #[test]
    fn test_nysiis() {
        let runtime = setup();
        let data = Variable::from_json(r#""Johnson""#).unwrap();
        let expr = runtime.compile("nysiis(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_string().unwrap().is_empty());
    }

    #[test]
    fn test_match_rating_codex() {
        let runtime = setup();
        let data = Variable::from_json(r#""Smith""#).unwrap();
        let expr = runtime.compile("match_rating_codex(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_string().unwrap().is_empty());
    }

    #[test]
    fn test_caverphone() {
        let runtime = setup();
        let data = Variable::from_json(r#""Thompson""#).unwrap();
        let expr = runtime.compile("caverphone(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_string().unwrap().is_empty());
    }

    #[test]
    fn test_caverphone2() {
        let runtime = setup();
        let data = Variable::from_json(r#""Thompson""#).unwrap();
        let expr = runtime.compile("caverphone2(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_string().unwrap().is_empty());
    }

    #[test]
    fn test_sounds_like_true() {
        let runtime = setup();
        let data = Variable::from_json(r#"["Robert", "Rupert"]"#).unwrap();
        let expr = runtime.compile("sounds_like(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_sounds_like_false() {
        let runtime = setup();
        let data = Variable::from_json(r#"["Robert", "Smith"]"#).unwrap();
        let expr = runtime.compile("sounds_like(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_phonetic_match_default() {
        let runtime = setup();
        let data = Variable::from_json(r#"["Robert", "Rupert"]"#).unwrap();
        let expr = runtime.compile("phonetic_match(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_phonetic_match_metaphone() {
        let runtime = setup();
        let data = Variable::from_json(r#"["Smith", "Smyth"]"#).unwrap();
        let expr = runtime
            .compile("phonetic_match(@[0], @[1], 'metaphone')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Both should encode to SM0
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_phonetic_match_nysiis() {
        let runtime = setup();
        let data = Variable::from_json(r#"["Johnson", "Jonson"]"#).unwrap();
        let expr = runtime
            .compile("phonetic_match(@[0], @[1], 'nysiis')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }
}
