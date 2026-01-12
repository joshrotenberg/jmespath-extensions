//! Fuzzy string matching functions.
//!
//! This module provides fuzzy functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category fuzzy`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::fuzzy;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! fuzzy::register(&mut runtime);
//! ```

use std::rc::Rc;

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Variable, define_function};

/// Register all fuzzy matching functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("levenshtein", Box::new(LevenshteinFn::new()));
    runtime.register_function(
        "normalized_levenshtein",
        Box::new(NormalizedLevenshteinFn::new()),
    );
    runtime.register_function("damerau_levenshtein", Box::new(DamerauLevenshteinFn::new()));
    runtime.register_function(
        "normalized_damerau_levenshtein",
        Box::new(NormalizedDamerauLevenshteinFn::new()),
    );
    runtime.register_function("jaro", Box::new(JaroFn::new()));
    runtime.register_function("jaro_winkler", Box::new(JaroWinklerFn::new()));
    runtime.register_function("sorensen_dice", Box::new(SorensenDiceFn::new()));
    runtime.register_function("hamming", Box::new(HammingFn::new()));
    runtime.register_function("osa_distance", Box::new(OsaDistanceFn::new()));
}

// levenshtein(s1, s2) -> number
define_function!(
    LevenshteinFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for LevenshteinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let dist = strsim::levenshtein(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(dist as f64).unwrap(),
        )))
    }
}

// normalized_levenshtein(s1, s2) -> number (0.0-1.0)
define_function!(
    NormalizedLevenshteinFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for NormalizedLevenshteinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let sim = strsim::normalized_levenshtein(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(sim).unwrap(),
        )))
    }
}

// damerau_levenshtein(s1, s2) -> number
define_function!(
    DamerauLevenshteinFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for DamerauLevenshteinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let dist = strsim::damerau_levenshtein(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(dist as f64).unwrap(),
        )))
    }
}

// jaro(s1, s2) -> number (0.0-1.0)
define_function!(
    JaroFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for JaroFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let sim = strsim::jaro(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(sim).unwrap(),
        )))
    }
}

// jaro_winkler(s1, s2) -> number (0.0-1.0)
define_function!(
    JaroWinklerFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for JaroWinklerFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let sim = strsim::jaro_winkler(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(sim).unwrap(),
        )))
    }
}

// sorensen_dice(s1, s2) -> number (0.0-1.0)
define_function!(
    SorensenDiceFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for SorensenDiceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let sim = strsim::sorensen_dice(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(sim).unwrap(),
        )))
    }
}

// normalized_damerau_levenshtein(s1, s2) -> number (0.0-1.0)
define_function!(
    NormalizedDamerauLevenshteinFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for NormalizedDamerauLevenshteinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let sim = strsim::normalized_damerau_levenshtein(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(sim).unwrap(),
        )))
    }
}

// hamming(s1, s2) -> number (returns null if strings have different lengths)
define_function!(
    HammingFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for HammingFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        match strsim::hamming(s1, s2) {
            Ok(dist) => Ok(Rc::new(Variable::Number(
                serde_json::Number::from_f64(dist as f64).unwrap(),
            ))),
            Err(_) => Ok(Rc::new(Variable::Null)), // Different lengths
        }
    }
}

// osa_distance(s1, s2) -> number (Optimal String Alignment distance)
define_function!(
    OsaDistanceFn,
    vec![ArgumentType::String, ArgumentType::String],
    None
);

impl Function for OsaDistanceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();
        let dist = strsim::osa_distance(s1, s2);
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(dist as f64).unwrap(),
        )))
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
    fn test_levenshtein() {
        let runtime = setup();
        let expr = runtime.compile("levenshtein('kitten', 'sitting')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_levenshtein_identical() {
        let runtime = setup();
        let expr = runtime.compile("levenshtein('hello', 'hello')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_normalized_levenshtein() {
        let runtime = setup();
        let expr = runtime
            .compile("normalized_levenshtein('hello', 'hello')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_normalized_levenshtein_different() {
        let runtime = setup();
        let expr = runtime
            .compile("normalized_levenshtein('hello', 'world')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let sim = result.as_number().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_damerau_levenshtein() {
        let runtime = setup();
        // Transposition: "ab" -> "ba" is 1 edit in Damerau-Levenshtein
        let expr = runtime.compile("damerau_levenshtein('ab', 'ba')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_jaro() {
        let runtime = setup();
        let expr = runtime.compile("jaro('hello', 'hallo')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let sim = result.as_number().unwrap();
        assert!(sim > 0.8);
    }

    #[test]
    fn test_jaro_identical() {
        let runtime = setup();
        let expr = runtime.compile("jaro('test', 'test')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_jaro_winkler() {
        let runtime = setup();
        // Jaro-Winkler boosts common prefixes
        let expr = runtime
            .compile("jaro_winkler('prefix_abc', 'prefix_xyz')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let sim = result.as_number().unwrap();
        assert!(sim > 0.7);
    }

    #[test]
    fn test_jaro_winkler_vs_jaro() {
        let runtime = setup();
        // Jaro-Winkler should be >= Jaro for strings with common prefix
        let jw_expr = runtime.compile("jaro_winkler('hello', 'hella')").unwrap();
        let j_expr = runtime.compile("jaro('hello', 'hella')").unwrap();
        let jw = jw_expr.search(&Variable::Null).unwrap();
        let j = j_expr.search(&Variable::Null).unwrap();
        assert!(jw.as_number().unwrap() >= j.as_number().unwrap());
    }

    #[test]
    fn test_sorensen_dice() {
        let runtime = setup();
        let expr = runtime.compile("sorensen_dice('night', 'nacht')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let sim = result.as_number().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_sorensen_dice_identical() {
        let runtime = setup();
        let expr = runtime.compile("sorensen_dice('test', 'test')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_normalized_damerau_levenshtein() {
        let runtime = setup();
        let expr = runtime
            .compile("normalized_damerau_levenshtein('hello', 'hello')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_normalized_damerau_levenshtein_transposition() {
        let runtime = setup();
        // "ab" vs "ba" - transposition
        let expr = runtime
            .compile("normalized_damerau_levenshtein('ab', 'ba')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let sim = result.as_number().unwrap();
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_hamming() {
        let runtime = setup();
        let expr = runtime.compile("hamming('karolin', 'kathrin')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_hamming_identical() {
        let runtime = setup();
        let expr = runtime.compile("hamming('hello', 'hello')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_hamming_different_lengths() {
        let runtime = setup();
        // Different lengths should return null
        let expr = runtime.compile("hamming('hello', 'hi')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_osa_distance() {
        let runtime = setup();
        // OSA allows transpositions
        let expr = runtime.compile("osa_distance('ab', 'ba')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_osa_distance_identical() {
        let runtime = setup();
        let expr = runtime.compile("osa_distance('hello', 'hello')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }
}
