//! JSON Patch (RFC 6902) functions.
//!
//! This module provides jsonpatch functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category jsonpatch`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::jsonpatch;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! jsonpatch::register(&mut runtime);
//! ```

use std::rc::Rc;

use crate::common::{ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Variable};
use crate::define_function;
use jmespath::Runtime;

/// Register all JSON patch functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("json_patch", Box::new(JsonPatchFn::new()));
    runtime.register_function("json_merge_patch", Box::new(JsonMergePatchFn::new()));
    runtime.register_function("json_diff", Box::new(JsonDiffFn::new()));
}

// =============================================================================
// json_patch(obj, patch) -> object (RFC 6902)
// Apply a JSON Patch (RFC 6902) to an object.
// See: https://datatracker.ietf.org/doc/html/rfc6902
// =============================================================================

define_function!(
    JsonPatchFn,
    vec![ArgumentType::Any, ArgumentType::Array],
    None
);

impl Function for JsonPatchFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        // Convert the JMESPath Variable to serde_json::Value
        let obj_json: serde_json::Value = serde_json::to_value(&*args[0]).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert object: {}", e)),
            )
        })?;

        let patch_json: serde_json::Value = serde_json::to_value(&*args[1]).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert patch: {}", e)),
            )
        })?;

        // Parse the patch
        let patch: json_patch::Patch = serde_json::from_value(patch_json).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid JSON Patch format: {}", e)),
            )
        })?;

        // Apply the patch
        let mut result = obj_json;
        json_patch::patch(&mut result, &patch).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to apply patch: {}", e)),
            )
        })?;

        // Convert back to Variable
        let var = Variable::from_json(&result.to_string()).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert result: {}", e)),
            )
        })?;

        Ok(Rc::new(var))
    }
}

// =============================================================================
// json_merge_patch(obj, patch) -> object (RFC 7386)
// Apply a JSON Merge Patch (RFC 7386) to an object.
// See: https://datatracker.ietf.org/doc/html/rfc7386
// =============================================================================

define_function!(
    JsonMergePatchFn,
    vec![ArgumentType::Any, ArgumentType::Any],
    None
);

impl Function for JsonMergePatchFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        // Convert the JMESPath Variables to serde_json::Value
        let obj_json: serde_json::Value = serde_json::to_value(&*args[0]).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert object: {}", e)),
            )
        })?;

        let patch_json: serde_json::Value = serde_json::to_value(&*args[1]).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert patch: {}", e)),
            )
        })?;

        // Apply the merge patch
        let mut result = obj_json;
        json_patch::merge(&mut result, &patch_json);

        // Convert back to Variable
        let var = Variable::from_json(&result.to_string()).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert result: {}", e)),
            )
        })?;

        Ok(Rc::new(var))
    }
}

// =============================================================================
// json_diff(a, b) -> array (RFC 6902 JSON Patch)
// Generate a JSON Patch (RFC 6902) that transforms the first object into the second.
// See: https://datatracker.ietf.org/doc/html/rfc6902
// =============================================================================

define_function!(JsonDiffFn, vec![ArgumentType::Any, ArgumentType::Any], None);

impl Function for JsonDiffFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        // Convert the JMESPath Variables to serde_json::Value
        let a_json: serde_json::Value = serde_json::to_value(&*args[0]).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert first argument: {}", e)),
            )
        })?;

        let b_json: serde_json::Value = serde_json::to_value(&*args[1]).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert second argument: {}", e)),
            )
        })?;

        // Generate the diff
        let patch = json_patch::diff(&a_json, &b_json);

        // Convert patch to JSON value
        let patch_json = serde_json::to_value(&patch).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to serialize patch: {}", e)),
            )
        })?;

        // Convert back to Variable
        let var = Variable::from_json(&patch_json.to_string()).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Failed to convert result: {}", e)),
            )
        })?;

        Ok(Rc::new(var))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_runtime() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        runtime
    }

    #[test]
    fn test_json_patch_add() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"doc": {"a": 1}, "patch": [{"op": "add", "path": "/b", "value": 2}]}"#,
        )
        .unwrap();
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_json_patch_remove() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"doc": {"a": 1, "b": 2}, "patch": [{"op": "remove", "path": "/b"}]}"#,
        )
        .unwrap();
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert!(obj.get("b").is_none());
    }

    #[test]
    fn test_json_patch_replace() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"doc": {"a": 1}, "patch": [{"op": "replace", "path": "/a", "value": 99}]}"#,
        )
        .unwrap();
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 99);
    }

    #[test]
    fn test_json_patch_multiple_ops() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"doc": {"a": 1}, "patch": [{"op": "add", "path": "/b", "value": 2}, {"op": "replace", "path": "/a", "value": 10}]}"#,
        )
        .unwrap();
        let expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 10);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_json_merge_patch_simple() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"doc": {"a": 1, "b": 2}, "patch": {"b": 3, "c": 4}}"#).unwrap();
        let expr = runtime.compile("json_merge_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap() as i64, 3);
        assert_eq!(obj.get("c").unwrap().as_number().unwrap() as i64, 4);
    }

    #[test]
    fn test_json_merge_patch_remove_with_null() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"doc": {"a": 1, "b": 2}, "patch": {"b": null}}"#).unwrap();
        let expr = runtime.compile("json_merge_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert!(obj.get("b").is_none());
    }

    #[test]
    fn test_json_merge_patch_nested() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"doc": {"a": {"x": 1}}, "patch": {"a": {"y": 2}}}"#).unwrap();
        let expr = runtime.compile("json_merge_patch(doc, patch)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.get("x").unwrap().as_number().unwrap() as i64, 1);
        assert_eq!(a.get("y").unwrap().as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_json_diff_add() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 1, "y": 2}}"#).unwrap();
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let op = arr[0].as_object().unwrap();
        assert_eq!(op.get("op").unwrap().as_string().unwrap(), "add");
        assert_eq!(op.get("path").unwrap().as_string().unwrap(), "/y");
    }

    #[test]
    fn test_json_diff_remove() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1, "y": 2}, "b": {"x": 1}}"#).unwrap();
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let op = arr[0].as_object().unwrap();
        assert_eq!(op.get("op").unwrap().as_string().unwrap(), "remove");
        assert_eq!(op.get("path").unwrap().as_string().unwrap(), "/y");
    }

    #[test]
    fn test_json_diff_replace() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 2}}"#).unwrap();
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        let op = arr[0].as_object().unwrap();
        assert_eq!(op.get("op").unwrap().as_string().unwrap(), "replace");
        assert_eq!(op.get("path").unwrap().as_string().unwrap(), "/x");
    }

    #[test]
    fn test_json_diff_no_changes() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 1}}"#).unwrap();
        let expr = runtime.compile("json_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_json_diff_roundtrip() {
        // Generate a diff and apply it - should get the same result
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 2, "y": 3}}"#).unwrap();

        // First get the diff
        let diff_expr = runtime.compile("json_diff(a, b)").unwrap();
        let diff_result = diff_expr.search(&data).unwrap();

        // Now apply the diff to a
        let data_with_patch = Variable::from_json(&format!(
            r#"{{"doc": {{"x": 1}}, "patch": {}}}"#,
            serde_json::to_string(&*diff_result).unwrap()
        ))
        .unwrap();
        let patch_expr = runtime.compile("json_patch(doc, patch)").unwrap();
        let patched = patch_expr.search(&data_with_patch).unwrap();

        // Should equal b
        let obj = patched.as_object().unwrap();
        assert_eq!(obj.get("x").unwrap().as_number().unwrap() as i64, 2);
        assert_eq!(obj.get("y").unwrap().as_number().unwrap() as i64, 3);
    }
}
