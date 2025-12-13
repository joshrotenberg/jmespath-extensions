//! File path manipulation functions.
//!
//! This module provides path functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category path`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::path;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! path::register(&mut runtime);
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all path functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("path_basename", Box::new(PathBasenameFn::new()));
    runtime.register_function("path_dirname", Box::new(PathDirnameFn::new()));
    runtime.register_function("path_ext", Box::new(PathExtFn::new()));
    runtime.register_function("path_join", Box::new(PathJoinFn::new()));
}

// =============================================================================
// path_basename(string) -> string (filename from path)
// =============================================================================

define_function!(PathBasenameFn, vec![ArgumentType::String], None);

impl Function for PathBasenameFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let basename = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        Ok(Rc::new(Variable::String(basename.to_string())))
    }
}

// =============================================================================
// path_dirname(string) -> string (directory from path)
// =============================================================================

define_function!(PathDirnameFn, vec![ArgumentType::String], None);

impl Function for PathDirnameFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let dirname = std::path::Path::new(path)
            .parent()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        Ok(Rc::new(Variable::String(dirname.to_string())))
    }
}

// =============================================================================
// path_ext(string) -> string (extension from path, with dot)
// =============================================================================

define_function!(PathExtFn, vec![ArgumentType::String], None);

impl Function for PathExtFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s))
            .unwrap_or_default();

        Ok(Rc::new(Variable::String(ext)))
    }
}

// =============================================================================
// path_join(array_of_parts) -> string
// =============================================================================

define_function!(PathJoinFn, vec![ArgumentType::Array], None);

impl Function for PathJoinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut path = std::path::PathBuf::new();
        for part in arr {
            if let Some(s) = part.as_string() {
                path.push(s);
            }
        }

        let result = path.to_str().unwrap_or("").to_string();
        Ok(Rc::new(Variable::String(result)))
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
    fn test_path_basename() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_basename(@)").unwrap();
        let data = Variable::String("/path/to/file.txt".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "file.txt");
    }

    #[test]
    fn test_path_dirname() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_dirname(@)").unwrap();
        let data = Variable::String("/path/to/file.txt".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "/path/to");
    }

    #[test]
    fn test_path_ext() {
        let runtime = setup_runtime();
        let expr = runtime.compile("path_ext(@)").unwrap();
        let data = Variable::String("/path/to/file.txt".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), ".txt");
    }
}
