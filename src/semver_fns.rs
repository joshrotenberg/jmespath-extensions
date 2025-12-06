//! Semantic versioning functions for JMESPath.
//!
//! This module provides functions for parsing and comparing semantic versions.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `semver_parse(s)` | Parse version into {major, minor, patch, pre, build} |
//! | `semver_major(s)` | Extract major version number |
//! | `semver_minor(s)` | Extract minor version number |
//! | `semver_patch(s)` | Extract patch version number |
//! | `semver_compare(v1, v2)` | Compare versions: -1, 0, or 1 |
//! | `semver_satisfies(version, requirement)` | Check if version matches requirement |
//! | `semver_is_valid(s)` | Check if string is valid semver |
//!
//! # Examples
//!
//! ```
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::semver_fns;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! semver_fns::register(&mut runtime);
//!
//! // Compare versions
//! let data = Variable::from_json(r#"["1.2.3", "2.0.0"]"#).unwrap();
//! let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
//! let result = expr.search(&data).unwrap();
//! // Result: -1 (first is less than second)
//! ```

use std::rc::Rc;

use semver_crate::{Version, VersionReq};

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Signature, Variable};

/// Register all semver functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("semver_parse", Box::new(SemverParseFn::new()));
    runtime.register_function("semver_major", Box::new(SemverMajorFn::new()));
    runtime.register_function("semver_minor", Box::new(SemverMinorFn::new()));
    runtime.register_function("semver_patch", Box::new(SemverPatchFn::new()));
    runtime.register_function("semver_compare", Box::new(SemverCompareFn::new()));
    runtime.register_function("semver_satisfies", Box::new(SemverSatisfiesFn::new()));
    runtime.register_function("semver_is_valid", Box::new(SemverIsValidFn::new()));
}

// =============================================================================
// semver_parse(s) -> object
// =============================================================================

pub struct SemverParseFn {
    signature: Signature,
}

impl Default for SemverParseFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverParseFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SemverParseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        match Version::parse(s) {
            Ok(v) => {
                let pre = if v.pre.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(v.pre.to_string())
                };
                let build = if v.build.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::String(v.build.to_string())
                };

                let obj = serde_json::json!({
                    "major": v.major,
                    "minor": v.minor,
                    "patch": v.patch,
                    "pre": pre,
                    "build": build
                });

                Ok(Rc::new(Variable::from_json(&obj.to_string()).unwrap()))
            }
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// semver_major(s) -> number
// =============================================================================

pub struct SemverMajorFn {
    signature: Signature,
}

impl Default for SemverMajorFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverMajorFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SemverMajorFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        match Version::parse(s) {
            Ok(v) => Ok(Rc::new(Variable::Number(serde_json::Number::from(v.major)))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// semver_minor(s) -> number
// =============================================================================

pub struct SemverMinorFn {
    signature: Signature,
}

impl Default for SemverMinorFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverMinorFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SemverMinorFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        match Version::parse(s) {
            Ok(v) => Ok(Rc::new(Variable::Number(serde_json::Number::from(v.minor)))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// semver_patch(s) -> number
// =============================================================================

pub struct SemverPatchFn {
    signature: Signature,
}

impl Default for SemverPatchFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverPatchFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SemverPatchFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();

        match Version::parse(s) {
            Ok(v) => Ok(Rc::new(Variable::Number(serde_json::Number::from(v.patch)))),
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
    }
}

// =============================================================================
// semver_compare(v1, v2) -> number (-1, 0, 1)
// =============================================================================

pub struct SemverCompareFn {
    signature: Signature,
}

impl Default for SemverCompareFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverCompareFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::String], None),
        }
    }
}

impl Function for SemverCompareFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s1 = args[0].as_string().unwrap();
        let s2 = args[1].as_string().unwrap();

        let v1 = match Version::parse(s1) {
            Ok(v) => v,
            Err(_) => return Ok(Rc::new(Variable::Null)),
        };
        let v2 = match Version::parse(s2) {
            Ok(v) => v,
            Err(_) => return Ok(Rc::new(Variable::Null)),
        };

        let result = match v1.cmp(&v2) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };

        Ok(Rc::new(Variable::Number(serde_json::Number::from(result))))
    }
}

// =============================================================================
// semver_satisfies(version, requirement) -> bool
// =============================================================================

pub struct SemverSatisfiesFn {
    signature: Signature,
}

impl Default for SemverSatisfiesFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverSatisfiesFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::String], None),
        }
    }
}

impl Function for SemverSatisfiesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let version_str = args[0].as_string().unwrap();
        let req_str = args[1].as_string().unwrap();

        let version = match Version::parse(version_str) {
            Ok(v) => v,
            Err(_) => return Ok(Rc::new(Variable::Null)),
        };
        let req = match VersionReq::parse(req_str) {
            Ok(r) => r,
            Err(_) => return Ok(Rc::new(Variable::Null)),
        };

        Ok(Rc::new(Variable::Bool(req.matches(&version))))
    }
}

// =============================================================================
// semver_is_valid(s) -> bool
// =============================================================================

pub struct SemverIsValidFn {
    signature: Signature,
}

impl Default for SemverIsValidFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SemverIsValidFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for SemverIsValidFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let s = args[0].as_string().unwrap();
        let is_valid = Version::parse(s).is_ok();
        Ok(Rc::new(Variable::Bool(is_valid)))
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
    fn test_semver_parse() {
        let runtime = setup();
        let data = Variable::from_json(r#""1.2.3""#).unwrap();
        let expr = runtime.compile("semver_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("major").unwrap().as_number().unwrap(), 1.0);
        assert_eq!(obj.get("minor").unwrap().as_number().unwrap(), 2.0);
        assert_eq!(obj.get("patch").unwrap().as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_semver_parse_with_pre() {
        let runtime = setup();
        let data = Variable::from_json(r#""1.0.0-alpha.1""#).unwrap();
        let expr = runtime.compile("semver_parse(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("major").unwrap().as_number().unwrap(), 1.0);
        assert_eq!(obj.get("pre").unwrap().as_string().unwrap(), "alpha.1");
    }

    #[test]
    fn test_semver_major() {
        let runtime = setup();
        let data = Variable::from_json(r#""2.3.4""#).unwrap();
        let expr = runtime.compile("semver_major(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_semver_minor() {
        let runtime = setup();
        let data = Variable::from_json(r#""2.3.4""#).unwrap();
        let expr = runtime.compile("semver_minor(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_semver_patch() {
        let runtime = setup();
        let data = Variable::from_json(r#""2.3.4""#).unwrap();
        let expr = runtime.compile("semver_patch(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);
    }

    #[test]
    fn test_semver_compare_less() {
        let runtime = setup();
        let data = Variable::from_json(r#"["1.0.0", "2.0.0"]"#).unwrap();
        let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), -1.0);
    }

    #[test]
    fn test_semver_compare_equal() {
        let runtime = setup();
        let data = Variable::from_json(r#"["1.0.0", "1.0.0"]"#).unwrap();
        let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_semver_compare_greater() {
        let runtime = setup();
        let data = Variable::from_json(r#"["2.0.0", "1.0.0"]"#).unwrap();
        let expr = runtime.compile("semver_compare(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_semver_satisfies_true() {
        let runtime = setup();
        let data = Variable::from_json(r#"["1.2.3", "^1.0.0"]"#).unwrap();
        let expr = runtime.compile("semver_satisfies(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_semver_satisfies_false() {
        let runtime = setup();
        let data = Variable::from_json(r#"["2.0.0", "^1.0.0"]"#).unwrap();
        let expr = runtime.compile("semver_satisfies(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_semver_satisfies_tilde() {
        let runtime = setup();
        let data = Variable::from_json(r#"["1.2.5", "~1.2.0"]"#).unwrap();
        let expr = runtime.compile("semver_satisfies(@[0], @[1])").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_semver_is_valid_true() {
        let runtime = setup();
        let data = Variable::from_json(r#""1.2.3""#).unwrap();
        let expr = runtime.compile("semver_is_valid(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_semver_is_valid_false() {
        let runtime = setup();
        let data = Variable::from_json(r#""not-a-version""#).unwrap();
        let expr = runtime.compile("semver_is_valid(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }
}
