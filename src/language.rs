//! Language detection functions.
//!
//! This module provides language detection functions for JMESPath queries
//! using the whatlang crate.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category language`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::language;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! language::register(&mut runtime);
//! ```

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::common::Function;
use crate::register_if_enabled;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Variable, define_function};

/// Register all language detection functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("detect_language", Box::new(DetectLanguageFn::new()));
    runtime.register_function("detect_language_iso", Box::new(DetectLanguageIsoFn::new()));
    runtime.register_function("detect_script", Box::new(DetectScriptFn::new()));
    runtime.register_function(
        "detect_language_confidence",
        Box::new(DetectLanguageConfidenceFn::new()),
    );
    runtime.register_function(
        "detect_language_info",
        Box::new(DetectLanguageInfoFn::new()),
    );
}

/// Register language detection functions with the runtime, filtered by the enabled set.
pub fn register_filtered(runtime: &mut Runtime, enabled: &HashSet<&str>) {
    register_if_enabled!(
        runtime,
        enabled,
        "detect_language",
        Box::new(DetectLanguageFn::new())
    );
    register_if_enabled!(
        runtime,
        enabled,
        "detect_language_iso",
        Box::new(DetectLanguageIsoFn::new())
    );
    register_if_enabled!(
        runtime,
        enabled,
        "detect_script",
        Box::new(DetectScriptFn::new())
    );
    register_if_enabled!(
        runtime,
        enabled,
        "detect_language_confidence",
        Box::new(DetectLanguageConfidenceFn::new())
    );
    register_if_enabled!(
        runtime,
        enabled,
        "detect_language_info",
        Box::new(DetectLanguageInfoFn::new())
    );
}

// =============================================================================
// detect_language(text) -> string (full name like "English")
// =============================================================================

define_function!(DetectLanguageFn, vec![ArgumentType::String], None);

impl Function for DetectLanguageFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_string().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let name = info.lang().to_string();
                Ok(Rc::new(Variable::String(name)))
            }
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// detect_language_iso(text) -> string (ISO 639-3 code like "eng")
// =============================================================================

define_function!(DetectLanguageIsoFn, vec![ArgumentType::String], None);

impl Function for DetectLanguageIsoFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_string().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let code = info.lang().code();
                Ok(Rc::new(Variable::String(code.to_string())))
            }
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// detect_script(text) -> string (script name like "Latin")
// =============================================================================

define_function!(DetectScriptFn, vec![ArgumentType::String], None);

impl Function for DetectScriptFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_string().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let script = format!("{:?}", info.script());
                Ok(Rc::new(Variable::String(script)))
            }
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// detect_language_confidence(text) -> number (0.0-1.0)
// =============================================================================

define_function!(DetectLanguageConfidenceFn, vec![ArgumentType::String], None);

impl Function for DetectLanguageConfidenceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_string().unwrap();

        match whatlang::detect(text) {
            Some(info) => Ok(Rc::new(Variable::Number(
                serde_json::Number::from_f64(info.confidence()).unwrap(),
            ))),
            None => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// detect_language_info(text) -> object with full detection info
// =============================================================================

define_function!(DetectLanguageInfoFn, vec![ArgumentType::String], None);

impl Function for DetectLanguageInfoFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let text = args[0].as_string().unwrap();

        match whatlang::detect(text) {
            Some(info) => {
                let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();

                result.insert(
                    "language".to_string(),
                    Rc::new(Variable::String(info.lang().to_string())),
                );
                result.insert(
                    "code".to_string(),
                    Rc::new(Variable::String(info.lang().code().to_string())),
                );
                result.insert(
                    "script".to_string(),
                    Rc::new(Variable::String(format!("{:?}", info.script()))),
                );
                result.insert(
                    "confidence".to_string(),
                    Rc::new(Variable::Number(
                        serde_json::Number::from_f64(info.confidence()).unwrap(),
                    )),
                );
                result.insert(
                    "reliable".to_string(),
                    Rc::new(Variable::Bool(info.is_reliable())),
                );

                Ok(Rc::new(Variable::Object(result)))
            }
            None => Ok(Rc::new(Variable::Null)),
        }
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
    fn test_detect_language_english() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language('This is a test of the language detection system.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "English");
    }

    #[test]
    fn test_detect_language_spanish() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language('Esto es una prueba del sistema de deteccion de idiomas.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        // whatlang returns native language names
        assert_eq!(result.as_string().unwrap(), "Español");
    }

    #[test]
    fn test_detect_language_french() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language('Ceci est un test du systeme de detection de langue.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        // whatlang returns native language names
        assert_eq!(result.as_string().unwrap(), "Français");
    }

    #[test]
    fn test_detect_language_german() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language('Dies ist ein Test des Spracherkennungssystems.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        // whatlang returns native language names
        assert_eq!(result.as_string().unwrap(), "Deutsch");
    }

    #[test]
    fn test_detect_language_iso_english() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language_iso('This is English text.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "eng");
    }

    #[test]
    fn test_detect_language_iso_spanish() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language_iso('Este es un texto en espanol.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "spa");
    }

    #[test]
    fn test_detect_script_latin() {
        let runtime = setup();
        let expr = runtime.compile("detect_script('Hello world')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "Latin");
    }

    #[test]
    fn test_detect_script_cyrillic() {
        let runtime = setup();
        let expr = runtime.compile("detect_script('Привет мир')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "Cyrillic");
    }

    #[test]
    fn test_detect_script_arabic() {
        let runtime = setup();
        let expr = runtime.compile("detect_script('مرحبا بالعالم')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "Arabic");
    }

    #[test]
    fn test_detect_language_confidence() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language_confidence('This is definitely English text for testing.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let confidence = result.as_number().unwrap();
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_detect_language_info() {
        let runtime = setup();
        let expr = runtime
            .compile("detect_language_info('This is a test.')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let obj = result.as_object().unwrap();

        assert!(obj.contains_key("language"));
        assert!(obj.contains_key("code"));
        assert!(obj.contains_key("script"));
        assert!(obj.contains_key("confidence"));
        assert!(obj.contains_key("reliable"));

        assert_eq!(obj.get("language").unwrap().as_string().unwrap(), "English");
        assert_eq!(obj.get("code").unwrap().as_string().unwrap(), "eng");
        assert_eq!(obj.get("script").unwrap().as_string().unwrap(), "Latin");
    }

    #[test]
    fn test_detect_language_empty_string() {
        let runtime = setup();
        let expr = runtime.compile("detect_language('')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        // Empty string may return null or a guess
        // Just verify it doesn't crash
        assert!(result.is_null() || result.as_string().is_some());
    }

    #[test]
    fn test_detect_language_short_text() {
        let runtime = setup();
        let expr = runtime.compile("detect_language('Hi')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        // Short text may have low confidence but should still work
        assert!(result.is_null() || result.as_string().is_some());
    }
}
