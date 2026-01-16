//! Object/map manipulation functions.
//!
//! This module provides object functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category object`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::object;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! object::register(&mut runtime);
//! ```

use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTrainCase,
    ToUpperCamelCase,
};

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Signature,
    Variable,
};
use crate::define_function;

/// Register all object functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("items", Box::new(EntriesFn::new()));
    runtime.register_function("from_items", Box::new(FromEntriesFn::new()));
    runtime.register_function("from_entries", Box::new(FromEntriesFn::new())); // alias for jq/lodash users
    runtime.register_function("with_entries", Box::new(WithEntriesFn::new()));
    runtime.register_function("pick", Box::new(PickFn::new()));
    runtime.register_function("omit", Box::new(OmitFn::new()));
    runtime.register_function("invert", Box::new(InvertFn::new()));
    runtime.register_function("rename_keys", Box::new(RenameKeysFn::new()));
    runtime.register_function("flatten_keys", Box::new(FlattenKeysFn::new()));
    runtime.register_function("flatten", Box::new(FlattenKeysFn::new())); // alias
    runtime.register_function("unflatten_keys", Box::new(UnflattenKeysFn::new()));
    runtime.register_function("unflatten", Box::new(UnflattenKeysFn::new())); // alias
    runtime.register_function("flatten_array", Box::new(FlattenArrayFn::new()));
    runtime.register_function("deep_merge", Box::new(DeepMergeFn::new()));
    runtime.register_function("deep_equals", Box::new(DeepEqualsFn::new()));
    runtime.register_function("deep_diff", Box::new(DeepDiffFn::new()));
    runtime.register_function("get", Box::new(GetFn::new()));
    runtime.register_function("get_path", Box::new(GetFn::new())); // alias
    runtime.register_function("has", Box::new(HasFn::new()));
    runtime.register_function("has_path", Box::new(HasFn::new())); // alias
    runtime.register_function("defaults", Box::new(DefaultsFn::new()));
    runtime.register_function("defaults_deep", Box::new(DefaultsDeepFn::new()));
    runtime.register_function("set_path", Box::new(SetPathFn::new()));
    runtime.register_function("delete_path", Box::new(DeletePathFn::new()));
    runtime.register_function("paths", Box::new(PathsFn::new()));
    runtime.register_function("leaves", Box::new(LeavesFn::new()));
    runtime.register_function("leaves_with_paths", Box::new(LeavesWithPathsFn::new()));
    runtime.register_function("remove_nulls", Box::new(RemoveNullsFn::new()));
    runtime.register_function("remove_empty", Box::new(RemoveEmptyFn::new()));
    runtime.register_function(
        "remove_empty_strings",
        Box::new(RemoveEmptyStringsFn::new()),
    );
    runtime.register_function("compact_deep", Box::new(CompactDeepFn::new()));
    // Data quality functions
    runtime.register_function("completeness", Box::new(CompletenessFn::new()));
    runtime.register_function("type_consistency", Box::new(TypeConsistencyFn::new()));
    runtime.register_function("data_quality_score", Box::new(DataQualityScoreFn::new()));
    // Data redaction functions
    runtime.register_function("redact", Box::new(RedactFn::new()));
    runtime.register_function("mask", Box::new(MaskFn::new()));
    #[cfg(feature = "regex")]
    runtime.register_function("redact_keys", Box::new(RedactKeysFn::new()));
    // Recursive key search and transformation
    runtime.register_function("pluck_deep", Box::new(PluckDeepFn::new()));
    runtime.register_function("paths_to", Box::new(PathsToFn::new()));
    runtime.register_function("snake_keys", Box::new(SnakeKeysFn::new()));
    runtime.register_function("camel_keys", Box::new(CamelKeysFn::new()));
    runtime.register_function("kebab_keys", Box::new(KebabKeysFn::new()));
    runtime.register_function("pascal_keys", Box::new(PascalKeysFn::new()));
    runtime.register_function("shouty_snake_keys", Box::new(ShoutySnakeKeysFn::new()));
    runtime.register_function("shouty_kebab_keys", Box::new(ShoutyKebabKeysFn::new()));
    runtime.register_function("train_keys", Box::new(TrainKeysFn::new()));
    // Structural diff functions
    runtime.register_function("structural_diff", Box::new(StructuralDiffFn::new()));
    runtime.register_function("has_same_shape", Box::new(HasSameShapeFn::new()));
    // Schema inference
    runtime.register_function("infer_schema", Box::new(InferSchemaFn::new()));
    // Chunking functions
    runtime.register_function("chunk_by_size", Box::new(ChunkBySizeFn::new()));
    runtime.register_function("paginate", Box::new(PaginateFn::new()));
    runtime.register_function("estimate_size", Box::new(EstimateSizeFn::new()));
    runtime.register_function("truncate_to_size", Box::new(TruncateToSizeFn::new()));
    // Template functions
    runtime.register_function("template", Box::new(TemplateFn::new()));
    runtime.register_function("template_strict", Box::new(TemplateStrictFn::new()));
}

// =============================================================================
// items(object) -> array of [key, value] pairs (JEP-013)
// =============================================================================

define_function!(EntriesFn, vec![ArgumentType::Object], None);

impl Function for EntriesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        // JEP-013: Return array of [key, value] pairs (2-element arrays)
        let entries: Vec<Rcvar> = obj
            .iter()
            .map(|(k, v)| {
                let pair = vec![Rc::new(Variable::String(k.clone())) as Rcvar, v.clone()];
                Rc::new(Variable::Array(pair)) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(entries)))
    }
}

// =============================================================================
// from_items(array) -> object from array of [key, value] pairs (JEP-013)
// =============================================================================

define_function!(FromEntriesFn, vec![ArgumentType::Array], None);

impl Function for FromEntriesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut result = BTreeMap::new();

        for item in arr {
            // JEP-013: Each item should be a 2-element array [key, value]
            if let Some(pair) = item.as_array() {
                if pair.len() >= 2 {
                    // Key must be a string
                    if let Some(key_str) = pair[0].as_string() {
                        result.insert(key_str.to_string(), pair[1].clone());
                    }
                }
            }
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// with_entries(object, expr) -> object (transform entries, jq parity)
// =============================================================================

// Transform object entries using an expression.
// Equivalent to: from_items(map(expr, items(object)))
//
// The expression receives each entry as a [key, value] pair and should return
// a [key, value] pair (or null to remove the entry).
//
// # Arguments
// * `object` - The object to transform
// * `expr` - A JMESPath expression string that transforms [key, value] pairs
//
// # Returns
// A new object with transformed entries.
//
// # Example
// with_entries({a: 1, b: 2}, '[upper(@[`0`]), @[`1`] * `2`]') -> {A: 2, B: 4}

pub struct WithEntriesFn {
    signature: Signature,
}

impl Default for WithEntriesFn {
    fn default() -> Self {
        Self::new()
    }
}

impl WithEntriesFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::Object, ArgumentType::String], None),
        }
    }
}

impl Function for WithEntriesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let expr_str = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected expression string".to_owned()),
            )
        })?;

        // Compile the expression
        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in with_entries: {}", e)),
            )
        })?;

        let mut result = BTreeMap::new();

        // Apply expression to each [key, value] entry
        for (key, value) in obj.iter() {
            // Create [key, value] pair
            let entry = Rc::new(Variable::Array(vec![
                Rc::new(Variable::String(key.clone())) as Rcvar,
                value.clone(),
            ]));

            // Apply transformation
            let transformed = compiled.search(entry).map_err(|e| {
                JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse(format!("Expression error in with_entries: {}", e)),
                )
            })?;

            // Skip null results (allows filtering entries)
            if transformed.is_null() {
                continue;
            }

            // Result should be a [key, value] pair
            if let Some(pair) = transformed.as_array() {
                if pair.len() >= 2 {
                    if let Some(new_key) = pair[0].as_string() {
                        result.insert(new_key.to_string(), pair[1].clone());
                    }
                }
            }
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// pick(object, keys) -> object (select specific keys)
// =============================================================================

define_function!(
    PickFn,
    vec![ArgumentType::Object, ArgumentType::Array],
    None
);

impl Function for PickFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let keys_arr = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array of keys".to_owned()),
            )
        })?;

        let keys: HashSet<String> = keys_arr
            .iter()
            .filter_map(|k| k.as_string().map(|s| s.to_string()))
            .collect();

        let result: BTreeMap<String, Rcvar> = obj
            .iter()
            .filter(|(k, _)| keys.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// omit(object, keys) -> object (exclude specific keys)
// =============================================================================

define_function!(
    OmitFn,
    vec![ArgumentType::Object, ArgumentType::Array],
    None
);

impl Function for OmitFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let keys_arr = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array of keys".to_owned()),
            )
        })?;

        let keys: HashSet<String> = keys_arr
            .iter()
            .filter_map(|k| k.as_string().map(|s| s.to_string()))
            .collect();

        let result: BTreeMap<String, Rcvar> = obj
            .iter()
            .filter(|(k, _)| !keys.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// invert(object) -> object (swap keys and values)
// =============================================================================

define_function!(InvertFn, vec![ArgumentType::Object], None);

impl Function for InvertFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();

        for (k, v) in obj.iter() {
            let new_key = match &**v {
                Variable::String(s) => s.clone(),
                Variable::Number(n) => n.to_string(),
                Variable::Bool(b) => b.to_string(),
                Variable::Null => "null".to_string(),
                _ => continue,
            };
            result.insert(new_key, Rc::new(Variable::String(k.clone())));
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// rename_keys(object, mapping) -> object
// =============================================================================

define_function!(
    RenameKeysFn,
    vec![ArgumentType::Object, ArgumentType::Object],
    None
);

impl Function for RenameKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let mapping = args[1].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected mapping object".to_owned()),
            )
        })?;

        let rename_map: std::collections::HashMap<String, String> = mapping
            .iter()
            .filter_map(|(k, v)| v.as_string().map(|s| (k.clone(), s.to_string())))
            .collect();

        let result: BTreeMap<String, Rcvar> = obj
            .iter()
            .map(|(k, v)| {
                let new_key = rename_map.get(k).cloned().unwrap_or_else(|| k.clone());
                (new_key, v.clone())
            })
            .collect();

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// flatten_keys(object, separator?) -> object
// =============================================================================

define_function!(
    FlattenKeysFn,
    vec![ArgumentType::Object],
    Some(ArgumentType::String)
);

fn flatten_object(
    obj: &BTreeMap<String, Rcvar>,
    prefix: &str,
    separator: &str,
    result: &mut BTreeMap<String, Rcvar>,
) {
    for (k, v) in obj.iter() {
        let new_key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{}{}{}", prefix, separator, k)
        };

        if let Some(nested) = v.as_object() {
            flatten_object(nested, &new_key, separator, result);
        } else {
            result.insert(new_key, v.clone());
        }
    }
}

impl Function for FlattenKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let default_sep = ".".to_string();
        let separator = args
            .get(1)
            .and_then(|s| s.as_string().map(|s| s.to_string()))
            .unwrap_or(default_sep);

        let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();
        flatten_object(obj, "", &separator, &mut result);

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// unflatten_keys(object, separator?) -> object
// =============================================================================

define_function!(
    UnflattenKeysFn,
    vec![ArgumentType::Object],
    Some(ArgumentType::String)
);

fn insert_nested(obj: &mut BTreeMap<String, Rcvar>, parts: &[&str], value: Rcvar) {
    if parts.is_empty() {
        return;
    }

    if parts.len() == 1 {
        obj.insert(parts[0].to_string(), value);
        return;
    }

    let key = parts[0].to_string();
    let rest = &parts[1..];

    let nested = obj
        .entry(key.clone())
        .or_insert_with(|| Rc::new(Variable::Object(BTreeMap::new())));

    if let Some(nested_obj) = nested.as_object() {
        let mut new_obj = nested_obj.clone();
        insert_nested(&mut new_obj, rest, value);
        *obj.get_mut(&key).unwrap() = Rc::new(Variable::Object(new_obj));
    }
}

impl Function for UnflattenKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let default_sep = ".".to_string();
        let separator = args
            .get(1)
            .and_then(|s| s.as_string().map(|s| s.to_string()))
            .unwrap_or(default_sep);

        let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();

        for (key, value) in obj.iter() {
            let parts: Vec<&str> = key.split(&separator).collect();
            insert_nested(&mut result, &parts, value.clone());
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// flatten_array(any, separator?) -> object
// Flattens nested objects AND arrays with numeric indices
// =============================================================================

define_function!(
    FlattenArrayFn,
    vec![ArgumentType::Any],
    Some(ArgumentType::String)
);

fn flatten_value(
    value: &Variable,
    prefix: &str,
    separator: &str,
    result: &mut BTreeMap<String, Rcvar>,
) {
    match value {
        Variable::Object(obj) => {
            if obj.is_empty() {
                // Empty object is a leaf
                if !prefix.is_empty() {
                    result.insert(prefix.to_string(), Rc::new(Variable::Object(obj.clone())));
                }
            } else {
                for (k, v) in obj.iter() {
                    let new_key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}{}{}", prefix, separator, k)
                    };
                    flatten_value(v, &new_key, separator, result);
                }
            }
        }
        Variable::Array(arr) => {
            if arr.is_empty() {
                // Empty array is a leaf
                if !prefix.is_empty() {
                    result.insert(prefix.to_string(), Rc::new(Variable::Array(arr.clone())));
                }
            } else {
                for (idx, v) in arr.iter().enumerate() {
                    let new_key = if prefix.is_empty() {
                        idx.to_string()
                    } else {
                        format!("{}{}{}", prefix, separator, idx)
                    };
                    flatten_value(v, &new_key, separator, result);
                }
            }
        }
        _ => {
            // Leaf value (string, number, bool, null)
            if !prefix.is_empty() {
                result.insert(prefix.to_string(), Rc::new(value.clone()));
            } else {
                // Top-level primitive - just return it as-is with empty key
                result.insert(String::new(), Rc::new(value.clone()));
            }
        }
    }
}

impl Function for FlattenArrayFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let default_sep = ".".to_string();
        let separator = args
            .get(1)
            .and_then(|s| s.as_string().map(|s| s.to_string()))
            .unwrap_or(default_sep);

        let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();
        flatten_value(&args[0], "", &separator, &mut result);

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// deep_merge(obj1, obj2) -> object
// =============================================================================

define_function!(
    DeepMergeFn,
    vec![ArgumentType::Object, ArgumentType::Object],
    None
);

fn deep_merge_objects(
    base: &BTreeMap<String, Rcvar>,
    overlay: &BTreeMap<String, Rcvar>,
) -> BTreeMap<String, Rcvar> {
    let mut result = base.clone();

    for (key, overlay_value) in overlay {
        if let Some(base_value) = result.get(key) {
            if let (Some(base_obj), Some(overlay_obj)) =
                (base_value.as_object(), overlay_value.as_object())
            {
                let merged = deep_merge_objects(base_obj, overlay_obj);
                result.insert(key.clone(), Rc::new(Variable::Object(merged)));
            } else {
                result.insert(key.clone(), overlay_value.clone());
            }
        } else {
            result.insert(key.clone(), overlay_value.clone());
        }
    }

    result
}

impl Function for DeepMergeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj1 = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let obj2 = args[1].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let merged = deep_merge_objects(obj1, obj2);
        Ok(Rc::new(Variable::Object(merged)))
    }
}

// =============================================================================
// deep_equals(a, b) -> boolean
// =============================================================================

define_function!(
    DeepEqualsFn,
    vec![ArgumentType::Any, ArgumentType::Any],
    None
);

impl Function for DeepEqualsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        // Use JSON serialization for deep comparison (same approach as includes())
        let a_json = serde_json::to_string(&*args[0]).unwrap_or_default();
        let b_json = serde_json::to_string(&*args[1]).unwrap_or_default();

        Ok(Rc::new(Variable::Bool(a_json == b_json)))
    }
}

// =============================================================================
// deep_diff(a, b) -> object with added, removed, changed
// =============================================================================

define_function!(
    DeepDiffFn,
    vec![ArgumentType::Object, ArgumentType::Object],
    None
);

fn compute_deep_diff(
    a: &BTreeMap<String, Rcvar>,
    b: &BTreeMap<String, Rcvar>,
) -> BTreeMap<String, Rcvar> {
    let mut added: BTreeMap<String, Rcvar> = BTreeMap::new();
    let mut removed: BTreeMap<String, Rcvar> = BTreeMap::new();
    let mut changed: BTreeMap<String, Rcvar> = BTreeMap::new();

    // Find removed and changed keys
    for (key, a_value) in a.iter() {
        match b.get(key) {
            None => {
                // Key was removed
                removed.insert(key.clone(), a_value.clone());
            }
            Some(b_value) => {
                // Key exists in both - check if changed
                let a_json = serde_json::to_string(&**a_value).unwrap_or_default();
                let b_json = serde_json::to_string(&**b_value).unwrap_or_default();

                if a_json != b_json {
                    // Values are different
                    if let (Some(a_obj), Some(b_obj)) = (a_value.as_object(), b_value.as_object()) {
                        // Both are objects - recurse
                        let nested_diff = compute_deep_diff(a_obj, b_obj);
                        changed.insert(key.clone(), Rc::new(Variable::Object(nested_diff)));
                    } else {
                        // Not both objects - show from/to
                        let mut change_obj: BTreeMap<String, Rcvar> = BTreeMap::new();
                        change_obj.insert("from".to_string(), a_value.clone());
                        change_obj.insert("to".to_string(), b_value.clone());
                        changed.insert(key.clone(), Rc::new(Variable::Object(change_obj)));
                    }
                }
            }
        }
    }

    // Find added keys
    for (key, b_value) in b.iter() {
        if !a.contains_key(key) {
            added.insert(key.clone(), b_value.clone());
        }
    }

    let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();
    result.insert("added".to_string(), Rc::new(Variable::Object(added)));
    result.insert("removed".to_string(), Rc::new(Variable::Object(removed)));
    result.insert("changed".to_string(), Rc::new(Variable::Object(changed)));

    result
}

impl Function for DeepDiffFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj_a = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let obj_b = args[1].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let diff = compute_deep_diff(obj_a, obj_b);
        Ok(Rc::new(Variable::Object(diff)))
    }
}

// =============================================================================
// get(object, path, default?) -> value at path or default
// =============================================================================

/// Get value at a dot-separated path with optional default.
///
/// Supports dot notation for nested access and bracket notation for array indices.
///
/// # Arguments
/// * `object` - The object to query
/// * `path` - Dot-separated path string (e.g., "a.b.c" or "a\[0\].b")
/// * `default` - Optional default value if path doesn't exist
///
/// # Examples
/// ```text
/// get({a: {b: {c: 1}}}, 'a.b.c') -> 1
/// get({a: 1}, 'b.c', 'default') -> 'default'
/// get({a: [{b: 1}]}, 'a\[0\].b') -> 1
/// ```
pub struct GetFn {
    signature: crate::Signature,
}

impl Default for GetFn {
    fn default() -> Self {
        Self::new()
    }
}

impl GetFn {
    pub fn new() -> Self {
        Self {
            signature: crate::Signature::new(
                vec![ArgumentType::Any, ArgumentType::String],
                Some(ArgumentType::Any),
            ),
        }
    }
}

// Navigate to a value using a path string like "a.b.c", "a[0].b", or "a.0.b"
fn get_at_path(value: &Variable, path: &str) -> Option<Rcvar> {
    if path.is_empty() {
        return Some(Rc::new(value.clone()));
    }

    let mut current: Rcvar = Rc::new(value.clone());

    // Split path by dots, but handle array indices
    let parts = parse_path_parts(path);

    for part in parts {
        if let Some(idx) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            // Bracket notation array index access: [0]
            if let Ok(index) = idx.parse::<usize>() {
                if let Some(arr) = current.as_array() {
                    if index < arr.len() {
                        current = arr[index].clone();
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else if let Ok(index) = part.parse::<usize>() {
            // Numeric key - try array index first, then object key
            if let Some(arr) = current.as_array() {
                if index < arr.len() {
                    current = arr[index].clone();
                } else {
                    return None;
                }
            } else if let Some(obj) = current.as_object() {
                // Try as object key (for objects with numeric string keys)
                if let Some(val) = obj.get(&part) {
                    current = val.clone();
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            // Object key access
            if let Some(obj) = current.as_object() {
                if let Some(val) = obj.get(&part) {
                    current = val.clone();
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }

    Some(current)
}

/// Parse path string into parts, handling both dot notation and bracket notation
fn parse_path_parts(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
                // Collect the bracket expression
                let mut bracket = String::from("[");
                while let Some(&next) = chars.peek() {
                    bracket.push(chars.next().unwrap());
                    if next == ']' {
                        break;
                    }
                }
                parts.push(bracket);
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

impl Function for GetFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string path argument".to_owned()),
            )
        })?;

        let default_val = if args.len() > 2 {
            args[2].clone()
        } else {
            Rc::new(Variable::Null)
        };

        match get_at_path(&args[0], path) {
            Some(val) => Ok(val),
            None => Ok(default_val),
        }
    }
}

// =============================================================================
// has(object, path) -> boolean
// =============================================================================

/// Check if a path exists in an object.
///
/// # Arguments
/// * `object` - The object to check
/// * `path` - Dot-separated path string
///
/// # Examples
/// ```text
/// has({a: {b: 1}}, 'a.b') -> true
/// has({a: 1}, 'a.b.c') -> false
/// ```
pub struct HasFn {
    signature: crate::Signature,
}

impl Default for HasFn {
    fn default() -> Self {
        Self::new()
    }
}

impl HasFn {
    pub fn new() -> Self {
        Self {
            signature: crate::Signature::new(vec![ArgumentType::Any, ArgumentType::String], None),
        }
    }
}

impl Function for HasFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string path argument".to_owned()),
            )
        })?;

        let exists = get_at_path(&args[0], path).is_some();
        Ok(Rc::new(Variable::Bool(exists)))
    }
}

// =============================================================================
// defaults(object, defaults) -> object with defaults applied
// =============================================================================

/// Assign default values for missing keys (shallow).
///
/// # Arguments
/// * `object` - The base object
/// * `defaults` - Object with default values
///
/// # Examples
/// ```text
/// defaults({a: 1}, {a: 2, b: 3}) -> {a: 1, b: 3}
/// defaults({}, {a: 1, b: 2}) -> {a: 1, b: 2}
/// ```
pub struct DefaultsFn {
    signature: crate::Signature,
}

impl Default for DefaultsFn {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultsFn {
    pub fn new() -> Self {
        Self {
            signature: crate::Signature::new(
                vec![ArgumentType::Object, ArgumentType::Object],
                None,
            ),
        }
    }
}

impl Function for DefaultsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let defaults = args[1].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let mut result = obj.clone();

        // Add keys from defaults that don't exist in obj
        for (key, value) in defaults.iter() {
            if !result.contains_key(key) {
                result.insert(key.clone(), value.clone());
            }
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// defaults_deep(object, defaults) -> object with deep defaults applied
// =============================================================================

/// Recursively assign default values for missing keys.
///
/// # Arguments
/// * `object` - The base object
/// * `defaults` - Object with default values (applied recursively)
///
/// # Examples
/// ```text
/// defaults_deep({a: {b: 1}}, {a: {b: 2, c: 3}}) -> {a: {b: 1, c: 3}}
/// defaults_deep({x: 1}, {x: 2, y: {z: 3}}) -> {x: 1, y: {z: 3}}
/// ```
pub struct DefaultsDeepFn {
    signature: crate::Signature,
}

impl Default for DefaultsDeepFn {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultsDeepFn {
    pub fn new() -> Self {
        Self {
            signature: crate::Signature::new(
                vec![ArgumentType::Object, ArgumentType::Object],
                None,
            ),
        }
    }
}

fn apply_defaults_deep(
    obj: &BTreeMap<String, Rcvar>,
    defaults: &BTreeMap<String, Rcvar>,
) -> BTreeMap<String, Rcvar> {
    let mut result = obj.clone();

    for (key, default_value) in defaults.iter() {
        if let Some(existing) = result.get(key) {
            // If both are objects, merge recursively
            if let (Some(existing_obj), Some(default_obj)) =
                (existing.as_object(), default_value.as_object())
            {
                let merged = apply_defaults_deep(existing_obj, default_obj);
                result.insert(key.clone(), Rc::new(Variable::Object(merged)));
            }
            // Otherwise keep existing value
        } else {
            // Key doesn't exist, use default
            result.insert(key.clone(), default_value.clone());
        }
    }

    result
}

impl Function for DefaultsDeepFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let defaults = args[1].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected object argument".to_owned()),
            )
        })?;

        let result = apply_defaults_deep(obj, defaults);
        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// set_path(object, path, value) -> new object with value set at path
// Supports both dot notation ("a.b.c") and JSON pointer ("/a/b/c")
// =============================================================================

define_function!(
    SetPathFn,
    vec![ArgumentType::Any, ArgumentType::String, ArgumentType::Any],
    None
);

impl Function for SetPathFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string path argument".to_owned()),
            )
        })?;

        let value = args[2].clone();

        // Parse path - auto-detect format
        let parts = parse_path_for_mutation(path);
        if parts.is_empty() {
            // Empty path means replace the entire value
            return Ok(value);
        }

        // Deep clone and set value at path
        let result = set_at_path(&args[0], &parts, value);
        Ok(result)
    }
}

/// Parse a path string for mutation operations (set_path, delete_path).
/// Supports both JSON pointer ("/a/b/c") and dot notation ("a.b.c" or "a[0].b").
fn parse_path_for_mutation(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec![];
    }

    // Detect format: JSON pointer starts with /
    if path.starts_with('/') {
        parse_json_pointer(path)
    } else {
        // Use dot notation parser, converting bracket notation to simple parts
        parse_path_parts_for_mutation(path)
    }
}

/// Parse dot notation path into parts for mutation operations
fn parse_path_parts_for_mutation(path: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
                // Collect the index (without brackets)
                let mut index = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ']' {
                        chars.next();
                        break;
                    }
                    index.push(chars.next().unwrap());
                }
                parts.push(index);
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn parse_json_pointer(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec![];
    }

    // RFC 6901: path must start with /
    let path = path.strip_prefix('/').unwrap_or(path);

    if path.is_empty() {
        return vec![];
    }

    path.split('/')
        .map(|s| {
            // RFC 6901: ~1 -> /, ~0 -> ~
            s.replace("~1", "/").replace("~0", "~")
        })
        .collect()
}

fn set_at_path(value: &Rcvar, parts: &[String], new_value: Rcvar) -> Rcvar {
    if parts.is_empty() {
        return new_value;
    }

    let key = &parts[0];
    let remaining = &parts[1..];

    match value.as_ref() {
        Variable::Object(obj) => {
            let mut new_obj = obj.clone();
            if remaining.is_empty() {
                new_obj.insert(key.clone(), new_value);
            } else {
                let existing = obj
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Variable::Null));
                new_obj.insert(key.clone(), set_at_path(&existing, remaining, new_value));
            }
            Rc::new(Variable::Object(new_obj))
        }
        Variable::Array(arr) => {
            if let Ok(idx) = key.parse::<usize>() {
                let mut new_arr = arr.clone();
                // Extend array if needed
                while new_arr.len() <= idx {
                    new_arr.push(Rc::new(Variable::Null));
                }
                if remaining.is_empty() {
                    new_arr[idx] = new_value;
                } else {
                    new_arr[idx] = set_at_path(
                        &arr.get(idx)
                            .cloned()
                            .unwrap_or_else(|| Rc::new(Variable::Null)),
                        remaining,
                        new_value,
                    );
                }
                Rc::new(Variable::Array(new_arr))
            } else {
                // Can't use non-numeric key on array, return unchanged
                value.clone()
            }
        }
        _ => {
            // Create object structure if we have path parts
            if remaining.is_empty() {
                let mut new_obj = BTreeMap::new();
                new_obj.insert(key.clone(), new_value);
                Rc::new(Variable::Object(new_obj))
            } else {
                let mut new_obj = BTreeMap::new();
                new_obj.insert(
                    key.clone(),
                    set_at_path(&Rc::new(Variable::Null), remaining, new_value),
                );
                Rc::new(Variable::Object(new_obj))
            }
        }
    }
}

// =============================================================================
// delete_path(object, path) -> new object with value removed at path
// Supports both dot notation ("a.b.c") and JSON pointer ("/a/b/c")
// =============================================================================

define_function!(
    DeletePathFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

impl Function for DeletePathFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let path = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string path argument".to_owned()),
            )
        })?;

        // Parse path - auto-detect format
        let parts = parse_path_for_mutation(path);
        if parts.is_empty() {
            // Empty path means delete everything -> return null
            return Ok(Rc::new(Variable::Null));
        }

        let result = delete_at_path(&args[0], &parts);
        Ok(result)
    }
}

fn delete_at_path(value: &Rcvar, parts: &[String]) -> Rcvar {
    if parts.is_empty() {
        return Rc::new(Variable::Null);
    }

    let key = &parts[0];
    let remaining = &parts[1..];

    match value.as_ref() {
        Variable::Object(obj) => {
            let mut new_obj = obj.clone();
            if remaining.is_empty() {
                new_obj.remove(key);
            } else if let Some(existing) = obj.get(key) {
                new_obj.insert(key.clone(), delete_at_path(existing, remaining));
            }
            Rc::new(Variable::Object(new_obj))
        }
        Variable::Array(arr) => {
            if let Ok(idx) = key.parse::<usize>() {
                if idx < arr.len() {
                    let mut new_arr = arr.clone();
                    if remaining.is_empty() {
                        new_arr.remove(idx);
                    } else {
                        new_arr[idx] = delete_at_path(&arr[idx], remaining);
                    }
                    Rc::new(Variable::Array(new_arr))
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

// =============================================================================
// paths(value) -> array of all JSON pointer paths in the value
// =============================================================================

define_function!(PathsFn, vec![ArgumentType::Any], None);

impl Function for PathsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let mut paths = Vec::new();
        collect_paths(&args[0], String::new(), &mut paths);

        let result: Vec<Rcvar> = paths
            .into_iter()
            .map(|p| Rc::new(Variable::String(p)) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

fn collect_paths(value: &Rcvar, current_path: String, paths: &mut Vec<String>) {
    match value.as_ref() {
        Variable::Object(obj) => {
            if !current_path.is_empty() {
                paths.push(current_path.clone());
            }
            for (key, val) in obj.iter() {
                // Escape special characters per RFC 6901
                let escaped_key = key.replace('~', "~0").replace('/', "~1");
                let new_path = format!("{}/{}", current_path, escaped_key);
                collect_paths(val, new_path, paths);
            }
        }
        Variable::Array(arr) => {
            if !current_path.is_empty() {
                paths.push(current_path.clone());
            }
            for (idx, val) in arr.iter().enumerate() {
                let new_path = format!("{}/{}", current_path, idx);
                collect_paths(val, new_path, paths);
            }
        }
        _ => {
            if !current_path.is_empty() {
                paths.push(current_path);
            }
        }
    }
}

// =============================================================================
// leaves(value) -> array of all leaf values (non-object, non-array)
// =============================================================================

define_function!(LeavesFn, vec![ArgumentType::Any], None);

impl Function for LeavesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let mut leaves = Vec::new();
        collect_leaves(&args[0], &mut leaves);

        Ok(Rc::new(Variable::Array(leaves)))
    }
}

fn collect_leaves(value: &Rcvar, leaves: &mut Vec<Rcvar>) {
    match value.as_ref() {
        Variable::Object(obj) => {
            for (_, val) in obj.iter() {
                collect_leaves(val, leaves);
            }
        }
        Variable::Array(arr) => {
            for val in arr.iter() {
                collect_leaves(val, leaves);
            }
        }
        _ => {
            leaves.push(value.clone());
        }
    }
}

// =============================================================================
// leaves_with_paths(value) -> array of {path, value} objects for all leaves
// =============================================================================

define_function!(LeavesWithPathsFn, vec![ArgumentType::Any], None);

impl Function for LeavesWithPathsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let mut leaves = Vec::new();
        collect_leaves_with_paths(&args[0], String::new(), &mut leaves);

        let result: Vec<Rcvar> = leaves
            .into_iter()
            .map(|(path, value)| {
                let mut obj = BTreeMap::new();
                obj.insert("path".to_string(), Rc::new(Variable::String(path)));
                obj.insert("value".to_string(), value);
                Rc::new(Variable::Object(obj)) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

fn collect_leaves_with_paths(
    value: &Rcvar,
    current_path: String,
    leaves: &mut Vec<(String, Rcvar)>,
) {
    match value.as_ref() {
        Variable::Object(obj) => {
            if obj.is_empty() && !current_path.is_empty() {
                // Empty object is a leaf
                leaves.push((current_path, value.clone()));
            } else {
                for (key, val) in obj.iter() {
                    let escaped_key = key.replace('~', "~0").replace('/', "~1");
                    let new_path = format!("{}/{}", current_path, escaped_key);
                    collect_leaves_with_paths(val, new_path, leaves);
                }
            }
        }
        Variable::Array(arr) => {
            if arr.is_empty() && !current_path.is_empty() {
                // Empty array is a leaf
                leaves.push((current_path, value.clone()));
            } else {
                for (idx, val) in arr.iter().enumerate() {
                    let new_path = format!("{}/{}", current_path, idx);
                    collect_leaves_with_paths(val, new_path, leaves);
                }
            }
        }
        _ => {
            let path = if current_path.is_empty() {
                "/".to_string()
            } else {
                current_path
            };
            leaves.push((path, value.clone()));
        }
    }
}

// =============================================================================
// remove_nulls(any) -> any (recursively remove null values)
// =============================================================================

define_function!(RemoveNullsFn, vec![ArgumentType::Any], None);

impl Function for RemoveNullsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(remove_nulls_recursive(&args[0])))
    }
}

fn is_null_value(value: &Variable) -> bool {
    matches!(value, Variable::Null)
}

fn remove_nulls_recursive(value: &Variable) -> Variable {
    match value {
        Variable::Object(obj) => {
            let cleaned: BTreeMap<String, Rcvar> = obj
                .iter()
                .filter(|(_, v)| !is_null_value(v))
                .map(|(k, v)| (k.clone(), Rc::new(remove_nulls_recursive(v))))
                .collect();
            Variable::Object(cleaned)
        }
        Variable::Array(arr) => {
            let cleaned: Vec<Rcvar> = arr
                .iter()
                .filter(|v| !is_null_value(v))
                .map(|v| Rc::new(remove_nulls_recursive(v)))
                .collect();
            Variable::Array(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// remove_empty(any) -> any (recursively remove nulls, empty strings, empty arrays, empty objects)
// =============================================================================

define_function!(RemoveEmptyFn, vec![ArgumentType::Any], None);

impl Function for RemoveEmptyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(remove_empty_recursive(&args[0])))
    }
}

fn is_empty_value(value: &Variable) -> bool {
    match value {
        Variable::Null => true,
        Variable::String(s) => s.is_empty(),
        Variable::Array(arr) => arr.is_empty(),
        Variable::Object(obj) => obj.is_empty(),
        _ => false,
    }
}

fn remove_empty_recursive(value: &Variable) -> Variable {
    match value {
        Variable::Object(obj) => {
            let cleaned: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| (k.clone(), Rc::new(remove_empty_recursive(v))))
                .filter(|(_, v)| !is_empty_value(v))
                .collect();
            Variable::Object(cleaned)
        }
        Variable::Array(arr) => {
            let cleaned: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(remove_empty_recursive(v)))
                .filter(|v| !is_empty_value(v))
                .collect();
            Variable::Array(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// remove_empty_strings(any) -> any (recursively remove empty string values)
// =============================================================================

define_function!(RemoveEmptyStringsFn, vec![ArgumentType::Any], None);

impl Function for RemoveEmptyStringsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(remove_empty_strings_recursive(&args[0])))
    }
}

fn is_empty_string(value: &Variable) -> bool {
    matches!(value, Variable::String(s) if s.is_empty())
}

fn remove_empty_strings_recursive(value: &Variable) -> Variable {
    match value {
        Variable::Object(obj) => {
            let cleaned: BTreeMap<String, Rcvar> = obj
                .iter()
                .filter(|(_, v)| !is_empty_string(v))
                .map(|(k, v)| (k.clone(), Rc::new(remove_empty_strings_recursive(v))))
                .collect();
            Variable::Object(cleaned)
        }
        Variable::Array(arr) => {
            let cleaned: Vec<Rcvar> = arr
                .iter()
                .filter(|v| !is_empty_string(v))
                .map(|v| Rc::new(remove_empty_strings_recursive(v)))
                .collect();
            Variable::Array(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// compact_deep(array) -> array (recursively compact arrays, removing nulls at all levels)
// =============================================================================

define_function!(CompactDeepFn, vec![ArgumentType::Array], None);

impl Function for CompactDeepFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(compact_deep_recursive(&args[0])))
    }
}

fn compact_deep_recursive(value: &Variable) -> Variable {
    match value {
        Variable::Array(arr) => {
            let cleaned: Vec<Rcvar> = arr
                .iter()
                .filter(|v| !is_null_value(v))
                .map(|v| Rc::new(compact_deep_recursive(v)))
                .collect();
            Variable::Array(cleaned)
        }
        Variable::Object(obj) => {
            let cleaned: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| (k.clone(), Rc::new(compact_deep_recursive(v))))
                .collect();
            Variable::Object(cleaned)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// completeness(object) -> number (0-100)
// =============================================================================

define_function!(CompletenessFn, vec![ArgumentType::Object], None);

impl Function for CompletenessFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let obj = args[0].as_object().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("completeness: expected object".to_string()),
            )
        })?;

        if obj.is_empty() {
            return Ok(Rc::new(Variable::Number(
                serde_json::Number::from_f64(100.0).unwrap(),
            )));
        }

        let mut total_fields = 0;
        let mut non_null_fields = 0;

        count_completeness(&args[0], &mut total_fields, &mut non_null_fields);

        let score = if total_fields > 0 {
            (non_null_fields as f64 / total_fields as f64) * 100.0
        } else {
            100.0
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(score).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

fn count_completeness(value: &Variable, total: &mut usize, non_null: &mut usize) {
    match value {
        Variable::Object(obj) => {
            for (_, v) in obj.iter() {
                *total += 1;
                if !matches!(**v, Variable::Null) {
                    *non_null += 1;
                }
                // Recursively check nested structures
                count_completeness(v, total, non_null);
            }
        }
        Variable::Array(arr) => {
            for item in arr.iter() {
                count_completeness(item, total, non_null);
            }
        }
        _ => {}
    }
}

// =============================================================================
// type_consistency(array) -> object
// =============================================================================

define_function!(TypeConsistencyFn, vec![ArgumentType::Array], None);

impl Function for TypeConsistencyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("type_consistency: expected array".to_string()),
            )
        })?;

        if arr.is_empty() {
            let mut result = BTreeMap::new();
            result.insert(
                "consistent".to_string(),
                Rc::new(Variable::Bool(true)) as Rcvar,
            );
            result.insert(
                "types".to_string(),
                Rc::new(Variable::Array(vec![])) as Rcvar,
            );
            result.insert(
                "inconsistencies".to_string(),
                Rc::new(Variable::Array(vec![])) as Rcvar,
            );
            return Ok(Rc::new(Variable::Object(result)));
        }

        // For arrays of objects, check field type consistency
        let first_element = &arr[0];
        if let Variable::Object(first_obj) = &**first_element {
            return check_object_array_consistency(arr, first_obj);
        }

        // For simple arrays, check element type consistency
        let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();
        for item in arr.iter() {
            let type_name = get_type_name(item);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        let types: Vec<Rcvar> = type_counts
            .keys()
            .map(|t| Rc::new(Variable::String(t.clone())) as Rcvar)
            .collect();

        let consistent = type_counts.len() == 1;

        let mut result = BTreeMap::new();
        result.insert(
            "consistent".to_string(),
            Rc::new(Variable::Bool(consistent)) as Rcvar,
        );
        result.insert("types".to_string(), Rc::new(Variable::Array(types)));
        result.insert(
            "inconsistencies".to_string(),
            Rc::new(Variable::Array(vec![])) as Rcvar,
        );

        Ok(Rc::new(Variable::Object(result)))
    }
}

fn check_object_array_consistency(
    arr: &[Rcvar],
    first_obj: &BTreeMap<String, Rcvar>,
) -> Result<Rcvar, JmespathError> {
    // Build expected types from first object
    let mut expected_types: BTreeMap<String, String> = BTreeMap::new();
    for (key, val) in first_obj.iter() {
        expected_types.insert(key.clone(), get_type_name(val));
    }

    let mut inconsistencies: Vec<Rcvar> = Vec::new();

    for (idx, item) in arr.iter().enumerate().skip(1) {
        if let Variable::Object(obj) = &**item {
            for (key, val) in obj.iter() {
                let actual_type = get_type_name(val);
                if let Some(expected) = expected_types.get(key) {
                    // Allow null as valid for any type
                    if &actual_type != expected && actual_type != "null" && expected != "null" {
                        let mut issue = BTreeMap::new();
                        issue.insert(
                            "index".to_string(),
                            Rc::new(Variable::Number((idx as i64).into())) as Rcvar,
                        );
                        issue.insert(
                            "field".to_string(),
                            Rc::new(Variable::String(key.clone())) as Rcvar,
                        );
                        issue.insert(
                            "expected".to_string(),
                            Rc::new(Variable::String(expected.clone())) as Rcvar,
                        );
                        issue.insert(
                            "got".to_string(),
                            Rc::new(Variable::String(actual_type)) as Rcvar,
                        );
                        inconsistencies.push(Rc::new(Variable::Object(issue)));
                    }
                }
            }
        }
    }

    let types: Vec<Rcvar> = expected_types
        .iter()
        .map(|(k, v)| {
            let mut obj = BTreeMap::new();
            obj.insert(
                "field".to_string(),
                Rc::new(Variable::String(k.clone())) as Rcvar,
            );
            obj.insert(
                "type".to_string(),
                Rc::new(Variable::String(v.clone())) as Rcvar,
            );
            Rc::new(Variable::Object(obj)) as Rcvar
        })
        .collect();

    let mut result = BTreeMap::new();
    result.insert(
        "consistent".to_string(),
        Rc::new(Variable::Bool(inconsistencies.is_empty())) as Rcvar,
    );
    result.insert("types".to_string(), Rc::new(Variable::Array(types)));
    result.insert(
        "inconsistencies".to_string(),
        Rc::new(Variable::Array(inconsistencies)),
    );

    Ok(Rc::new(Variable::Object(result)))
}

fn get_type_name(value: &Variable) -> String {
    match value {
        Variable::Null => "null".to_string(),
        Variable::Bool(_) => "boolean".to_string(),
        Variable::Number(_) => "number".to_string(),
        Variable::String(_) => "string".to_string(),
        Variable::Array(_) => "array".to_string(),
        Variable::Object(_) => "object".to_string(),
        Variable::Expref(_) => "expression".to_string(),
    }
}

// =============================================================================
// data_quality_score(any) -> object
// =============================================================================

define_function!(DataQualityScoreFn, vec![ArgumentType::Any], None);

impl Function for DataQualityScoreFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let value = &args[0];

        let mut stats = QualityStats::default();
        analyze_quality(value, String::new(), &mut stats);

        // Calculate score (0-100)
        // Deductions: nulls, empty strings, type inconsistencies
        let total_issues = stats.null_count + stats.empty_string_count + stats.type_issues.len();
        let score = if stats.total_fields == 0 {
            100.0
        } else {
            let issue_ratio = total_issues as f64 / stats.total_fields as f64;
            (100.0 * (1.0 - issue_ratio)).max(0.0)
        };

        // Build issues array
        let mut issues: Vec<Rcvar> = Vec::new();

        for path in &stats.null_paths {
            let mut issue = BTreeMap::new();
            issue.insert(
                "path".to_string(),
                Rc::new(Variable::String(path.clone())) as Rcvar,
            );
            issue.insert(
                "issue".to_string(),
                Rc::new(Variable::String("null".to_string())) as Rcvar,
            );
            issues.push(Rc::new(Variable::Object(issue)));
        }

        for path in &stats.empty_string_paths {
            let mut issue = BTreeMap::new();
            issue.insert(
                "path".to_string(),
                Rc::new(Variable::String(path.clone())) as Rcvar,
            );
            issue.insert(
                "issue".to_string(),
                Rc::new(Variable::String("empty_string".to_string())) as Rcvar,
            );
            issues.push(Rc::new(Variable::Object(issue)));
        }

        for ti in &stats.type_issues {
            let mut issue = BTreeMap::new();
            issue.insert(
                "path".to_string(),
                Rc::new(Variable::String(ti.path.clone())) as Rcvar,
            );
            issue.insert(
                "issue".to_string(),
                Rc::new(Variable::String("type_mismatch".to_string())) as Rcvar,
            );
            issue.insert(
                "expected".to_string(),
                Rc::new(Variable::String(ti.expected.clone())) as Rcvar,
            );
            issue.insert(
                "got".to_string(),
                Rc::new(Variable::String(ti.got.clone())) as Rcvar,
            );
            issues.push(Rc::new(Variable::Object(issue)));
        }

        let mut result = BTreeMap::new();
        result.insert(
            "score".to_string(),
            Rc::new(Variable::Number(
                serde_json::Number::from_f64(score).unwrap_or_else(|| serde_json::Number::from(0)),
            )) as Rcvar,
        );
        result.insert(
            "total_fields".to_string(),
            Rc::new(Variable::Number((stats.total_fields as i64).into())) as Rcvar,
        );
        result.insert(
            "null_count".to_string(),
            Rc::new(Variable::Number((stats.null_count as i64).into())) as Rcvar,
        );
        result.insert(
            "empty_string_count".to_string(),
            Rc::new(Variable::Number((stats.empty_string_count as i64).into())) as Rcvar,
        );
        result.insert(
            "type_inconsistencies".to_string(),
            Rc::new(Variable::Number((stats.type_issues.len() as i64).into())) as Rcvar,
        );
        result.insert("issues".to_string(), Rc::new(Variable::Array(issues)));

        Ok(Rc::new(Variable::Object(result)))
    }
}

#[derive(Default)]
struct QualityStats {
    total_fields: usize,
    null_count: usize,
    empty_string_count: usize,
    null_paths: Vec<String>,
    empty_string_paths: Vec<String>,
    type_issues: Vec<TypeIssue>,
}

struct TypeIssue {
    path: String,
    expected: String,
    got: String,
}

fn analyze_quality(value: &Variable, path: String, stats: &mut QualityStats) {
    match value {
        Variable::Object(obj) => {
            for (key, val) in obj.iter() {
                let field_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                stats.total_fields += 1;

                match &**val {
                    Variable::Null => {
                        stats.null_count += 1;
                        stats.null_paths.push(field_path.clone());
                    }
                    Variable::String(s) if s.is_empty() => {
                        stats.empty_string_count += 1;
                        stats.empty_string_paths.push(field_path.clone());
                    }
                    _ => {}
                }

                analyze_quality(val, field_path, stats);
            }
        }
        Variable::Array(arr) => {
            // For arrays of objects, check type consistency
            if arr.len() > 1 {
                if let Some(Variable::Object(first_obj)) = arr.first().map(|v| &**v) {
                    let expected_types: BTreeMap<String, String> = first_obj
                        .iter()
                        .map(|(k, v)| (k.clone(), get_type_name(v)))
                        .collect();

                    for (idx, item) in arr.iter().enumerate().skip(1) {
                        if let Variable::Object(obj) = &**item {
                            for (key, val) in obj.iter() {
                                let actual_type = get_type_name(val);
                                if let Some(expected) = expected_types.get(key) {
                                    if &actual_type != expected
                                        && actual_type != "null"
                                        && expected != "null"
                                    {
                                        stats.type_issues.push(TypeIssue {
                                            path: format!("{}[{}].{}", path, idx, key),
                                            expected: expected.clone(),
                                            got: actual_type,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Analyze each array element
            for (idx, item) in arr.iter().enumerate() {
                let item_path = format!("{}[{}]", path, idx);
                analyze_quality(item, item_path, stats);
            }
        }
        _ => {}
    }
}

// =============================================================================
// redact(any, keys) -> any (replace values at keys with [REDACTED])
// =============================================================================

define_function!(RedactFn, vec![ArgumentType::Any, ArgumentType::Array], None);

impl Function for RedactFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let keys_arr = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array of keys".to_owned()),
            )
        })?;

        let keys: HashSet<String> = keys_arr
            .iter()
            .filter_map(|k| k.as_string().map(|s| s.to_string()))
            .collect();

        Ok(Rc::new(redact_recursive(&args[0], &keys)))
    }
}

fn redact_recursive(value: &Variable, keys: &HashSet<String>) -> Variable {
    match value {
        Variable::Object(obj) => {
            let redacted: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| {
                    if keys.contains(k) {
                        (
                            k.clone(),
                            Rc::new(Variable::String("[REDACTED]".to_string())),
                        )
                    } else {
                        (k.clone(), Rc::new(redact_recursive(v, keys)))
                    }
                })
                .collect();
            Variable::Object(redacted)
        }
        Variable::Array(arr) => {
            let redacted: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(redact_recursive(v, keys)))
                .collect();
            Variable::Array(redacted)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// mask(string, show_last?) -> string (mask all but last N chars)
// =============================================================================

define_function!(
    MaskFn,
    vec![ArgumentType::String],
    Some(ArgumentType::Number)
);

impl Function for MaskFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let show_last = if args.len() > 1 {
            args[1].as_number().unwrap_or(4.0) as usize
        } else {
            4
        };

        let len = s.len();
        let masked = if len <= show_last {
            "*".repeat(len)
        } else {
            let mask_count = len - show_last;
            format!("{}{}", "*".repeat(mask_count), &s[mask_count..])
        };

        Ok(Rc::new(Variable::String(masked)))
    }
}

// =============================================================================
// redact_keys(any, pattern) -> any (redact keys matching pattern)
// =============================================================================

#[cfg(feature = "regex")]
define_function!(
    RedactKeysFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

#[cfg(feature = "regex")]
impl Function for RedactKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let pattern = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected pattern string".to_owned()),
            )
        })?;

        let regex = regex::Regex::new(pattern).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse(format!("Invalid regex pattern: {}", e)),
            )
        })?;

        Ok(Rc::new(redact_keys_recursive(&args[0], &regex)))
    }
}

#[cfg(feature = "regex")]
fn redact_keys_recursive(value: &Variable, pattern: &regex::Regex) -> Variable {
    match value {
        Variable::Object(obj) => {
            let redacted: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| {
                    if pattern.is_match(k) {
                        (
                            k.clone(),
                            Rc::new(Variable::String("[REDACTED]".to_string())),
                        )
                    } else {
                        (k.clone(), Rc::new(redact_keys_recursive(v, pattern)))
                    }
                })
                .collect();
            Variable::Object(redacted)
        }
        Variable::Array(arr) => {
            let redacted: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(redact_keys_recursive(v, pattern)))
                .collect();
            Variable::Array(redacted)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// pluck_deep(any, key) -> array (find all values for key anywhere in structure)
// =============================================================================

define_function!(
    PluckDeepFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

impl Function for PluckDeepFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let key = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected key string".to_owned()),
            )
        })?;

        let mut results: Vec<Rcvar> = Vec::new();
        pluck_deep_recursive(&args[0], key, &mut results);
        Ok(Rc::new(Variable::Array(results)))
    }
}

fn pluck_deep_recursive(value: &Variable, key: &str, results: &mut Vec<Rcvar>) {
    match value {
        Variable::Object(obj) => {
            if let Some(v) = obj.get(key) {
                results.push(v.clone());
            }
            for (_, v) in obj.iter() {
                pluck_deep_recursive(v, key, results);
            }
        }
        Variable::Array(arr) => {
            for v in arr {
                pluck_deep_recursive(v, key, results);
            }
        }
        _ => {}
    }
}

// =============================================================================
// paths_to(any, key) -> array (return all dot-notation paths to matching keys)
// =============================================================================

define_function!(
    PathsToFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

impl Function for PathsToFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let key = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected key string".to_owned()),
            )
        })?;

        let mut paths: Vec<String> = Vec::new();
        paths_to_recursive(&args[0], key, String::new(), &mut paths);

        let result: Vec<Rcvar> = paths
            .into_iter()
            .map(|p| Rc::new(Variable::String(p)) as Rcvar)
            .collect();
        Ok(Rc::new(Variable::Array(result)))
    }
}

fn paths_to_recursive(value: &Variable, key: &str, current_path: String, paths: &mut Vec<String>) {
    match value {
        Variable::Object(obj) => {
            for (k, v) in obj.iter() {
                let new_path = if current_path.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", current_path, k)
                };
                if k == key {
                    paths.push(new_path.clone());
                }
                paths_to_recursive(v, key, new_path, paths);
            }
        }
        Variable::Array(arr) => {
            for (idx, v) in arr.iter().enumerate() {
                let new_path = if current_path.is_empty() {
                    idx.to_string()
                } else {
                    format!("{}.{}", current_path, idx)
                };
                paths_to_recursive(v, key, new_path, paths);
            }
        }
        _ => {}
    }
}

// =============================================================================
// snake_keys(any) -> any (recursively convert all keys to snake_case)
// Uses heck crate for proper case conversion
// =============================================================================

define_function!(SnakeKeysFn, vec![ArgumentType::Any], None);

impl Function for SnakeKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_snake_case()
        })))
    }
}

// =============================================================================
// camel_keys(any) -> any (recursively convert all keys to camelCase)
// Uses heck crate for proper case conversion
// =============================================================================

define_function!(CamelKeysFn, vec![ArgumentType::Any], None);

impl Function for CamelKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_lower_camel_case()
        })))
    }
}

// =============================================================================
// kebab_keys(any) -> any (recursively convert all keys to kebab-case)
// Uses heck crate for proper case conversion
// =============================================================================

define_function!(KebabKeysFn, vec![ArgumentType::Any], None);

impl Function for KebabKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_kebab_case()
        })))
    }
}

// =============================================================================
// pascal_keys(any) -> any (recursively convert all keys to PascalCase)
// Uses heck crate - also known as UpperCamelCase
// =============================================================================

define_function!(PascalKeysFn, vec![ArgumentType::Any], None);

impl Function for PascalKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_upper_camel_case()
        })))
    }
}

// =============================================================================
// shouty_snake_keys(any) -> any (recursively convert all keys to SHOUTY_SNAKE_CASE)
// Uses heck crate - useful for constants
// =============================================================================

define_function!(ShoutySnakeKeysFn, vec![ArgumentType::Any], None);

impl Function for ShoutySnakeKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_shouty_snake_case()
        })))
    }
}

// =============================================================================
// shouty_kebab_keys(any) -> any (recursively convert all keys to SHOUTY-KEBAB-CASE)
// Uses heck crate
// =============================================================================

define_function!(ShoutyKebabKeysFn, vec![ArgumentType::Any], None);

impl Function for ShoutyKebabKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_shouty_kebab_case()
        })))
    }
}

// =============================================================================
// train_keys(any) -> any (recursively convert all keys to Train-Case)
// Uses heck crate - like HTTP headers (Content-Type)
// =============================================================================

define_function!(TrainKeysFn, vec![ArgumentType::Any], None);

impl Function for TrainKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], |s| {
            s.to_train_case()
        })))
    }
}

fn transform_keys_recursive<F>(value: &Variable, transform: F) -> Variable
where
    F: Fn(&str) -> String + Copy,
{
    match value {
        Variable::Object(obj) => {
            let transformed: BTreeMap<String, Rcvar> = obj
                .iter()
                .map(|(k, v)| {
                    (
                        transform(k),
                        Rc::new(transform_keys_recursive(v, transform)),
                    )
                })
                .collect();
            Variable::Object(transformed)
        }
        Variable::Array(arr) => {
            let transformed: Vec<Rcvar> = arr
                .iter()
                .map(|v| Rc::new(transform_keys_recursive(v, transform)))
                .collect();
            Variable::Array(transformed)
        }
        _ => value.clone(),
    }
}

// =============================================================================
// structural_diff(obj1, obj2) -> object (compare shapes/schemas)
// =============================================================================

define_function!(
    StructuralDiffFn,
    vec![ArgumentType::Any, ArgumentType::Any],
    None
);

impl Function for StructuralDiffFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let mut added: Vec<String> = Vec::new();
        let mut removed: Vec<String> = Vec::new();
        let mut type_changed: Vec<BTreeMap<String, Rcvar>> = Vec::new();
        let mut unchanged: Vec<String> = Vec::new();

        compare_structure(
            &args[0],
            &args[1],
            String::new(),
            &mut added,
            &mut removed,
            &mut type_changed,
            &mut unchanged,
        );

        let mut result = BTreeMap::new();
        result.insert(
            "added".to_string(),
            Rc::new(Variable::Array(
                added
                    .into_iter()
                    .map(|s| Rc::new(Variable::String(s)) as Rcvar)
                    .collect(),
            )) as Rcvar,
        );
        result.insert(
            "removed".to_string(),
            Rc::new(Variable::Array(
                removed
                    .into_iter()
                    .map(|s| Rc::new(Variable::String(s)) as Rcvar)
                    .collect(),
            )) as Rcvar,
        );
        result.insert(
            "type_changed".to_string(),
            Rc::new(Variable::Array(
                type_changed
                    .into_iter()
                    .map(|m| Rc::new(Variable::Object(m)) as Rcvar)
                    .collect(),
            )) as Rcvar,
        );
        result.insert(
            "unchanged".to_string(),
            Rc::new(Variable::Array(
                unchanged
                    .into_iter()
                    .map(|s| Rc::new(Variable::String(s)) as Rcvar)
                    .collect(),
            )) as Rcvar,
        );

        Ok(Rc::new(Variable::Object(result)))
    }
}

fn get_structural_type(value: &Variable) -> &'static str {
    match value {
        Variable::Null => "null",
        Variable::Bool(_) => "boolean",
        Variable::Number(_) => "number",
        Variable::String(_) => "string",
        Variable::Array(_) => "array",
        Variable::Object(_) => "object",
        Variable::Expref(_) => "expression",
    }
}

fn compare_structure(
    a: &Variable,
    b: &Variable,
    path: String,
    added: &mut Vec<String>,
    removed: &mut Vec<String>,
    type_changed: &mut Vec<BTreeMap<String, Rcvar>>,
    unchanged: &mut Vec<String>,
) {
    let type_a = get_structural_type(a);
    let type_b = get_structural_type(b);

    if type_a != type_b {
        let mut change = BTreeMap::new();
        change.insert(
            "path".to_string(),
            Rc::new(Variable::String(if path.is_empty() {
                "$".to_string()
            } else {
                path
            })) as Rcvar,
        );
        change.insert(
            "from".to_string(),
            Rc::new(Variable::String(type_a.to_string())) as Rcvar,
        );
        change.insert(
            "to".to_string(),
            Rc::new(Variable::String(type_b.to_string())) as Rcvar,
        );
        type_changed.push(change);
        return;
    }

    match (a, b) {
        (Variable::Object(obj_a), Variable::Object(obj_b)) => {
            for key in obj_a.keys() {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                if let Some(val_b) = obj_b.get(key) {
                    compare_structure(
                        obj_a.get(key).unwrap(),
                        val_b,
                        new_path,
                        added,
                        removed,
                        type_changed,
                        unchanged,
                    );
                } else {
                    removed.push(new_path);
                }
            }
            for key in obj_b.keys() {
                if !obj_a.contains_key(key) {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    added.push(new_path);
                }
            }
        }
        _ => {
            if !path.is_empty() {
                unchanged.push(path);
            }
        }
    }
}

// =============================================================================
// has_same_shape(obj1, obj2) -> boolean
// =============================================================================

define_function!(
    HasSameShapeFn,
    vec![ArgumentType::Any, ArgumentType::Any],
    None
);

impl Function for HasSameShapeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let same = check_same_shape(&args[0], &args[1]);
        Ok(Rc::new(Variable::Bool(same)))
    }
}

fn check_same_shape(a: &Variable, b: &Variable) -> bool {
    let type_a = get_structural_type(a);
    let type_b = get_structural_type(b);

    if type_a != type_b {
        return false;
    }

    match (a, b) {
        (Variable::Object(obj_a), Variable::Object(obj_b)) => {
            if obj_a.keys().collect::<HashSet<_>>() != obj_b.keys().collect::<HashSet<_>>() {
                return false;
            }
            for key in obj_a.keys() {
                if !check_same_shape(obj_a.get(key).unwrap(), obj_b.get(key).unwrap()) {
                    return false;
                }
            }
            true
        }
        (Variable::Array(arr_a), Variable::Array(arr_b)) => {
            // Arrays have same shape if both empty or both non-empty with same element types
            if arr_a.is_empty() && arr_b.is_empty() {
                return true;
            }
            if arr_a.is_empty() || arr_b.is_empty() {
                return true; // One empty, can't compare element types
            }
            // Check first element types match
            check_same_shape(&arr_a[0], &arr_b[0])
        }
        _ => true, // Same primitive type
    }
}

// =============================================================================
// infer_schema(any) -> object (JSON Schema-like type inference)
// =============================================================================

define_function!(InferSchemaFn, vec![ArgumentType::Any], None);

impl Function for InferSchemaFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let schema = infer_schema_recursive(&args[0]);
        Ok(Rc::new(schema))
    }
}

fn infer_schema_recursive(value: &Variable) -> Variable {
    match value {
        Variable::Null => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("null".to_string())) as Rcvar,
            );
            Variable::Object(schema)
        }
        Variable::Bool(_) => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("boolean".to_string())) as Rcvar,
            );
            Variable::Object(schema)
        }
        Variable::Number(_) => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("number".to_string())) as Rcvar,
            );
            Variable::Object(schema)
        }
        Variable::String(_) => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("string".to_string())) as Rcvar,
            );
            Variable::Object(schema)
        }
        Variable::Array(arr) => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("array".to_string())) as Rcvar,
            );
            if !arr.is_empty() {
                // Infer items schema from first element (simplified)
                let items_schema = infer_schema_recursive(&arr[0]);
                schema.insert("items".to_string(), Rc::new(items_schema) as Rcvar);
            }
            Variable::Object(schema)
        }
        Variable::Object(obj) => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("object".to_string())) as Rcvar,
            );

            let mut properties = BTreeMap::new();
            for (key, val) in obj.iter() {
                let prop_schema = infer_schema_recursive(val);
                properties.insert(key.clone(), Rc::new(prop_schema) as Rcvar);
            }
            schema.insert(
                "properties".to_string(),
                Rc::new(Variable::Object(properties)) as Rcvar,
            );
            Variable::Object(schema)
        }
        Variable::Expref(_) => {
            let mut schema = BTreeMap::new();
            schema.insert(
                "type".to_string(),
                Rc::new(Variable::String("expression".to_string())) as Rcvar,
            );
            Variable::Object(schema)
        }
    }
}

// =============================================================================
// chunk_by_size(array, max_bytes) -> array of arrays
// =============================================================================

define_function!(
    ChunkBySizeFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for ChunkBySizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array".to_owned()),
            )
        })?;

        let max_bytes = args[1].as_number().unwrap_or(4000.0) as usize;

        let mut chunks: Vec<Rcvar> = Vec::new();
        let mut current_chunk: Vec<Rcvar> = Vec::new();
        let mut current_size: usize = 2; // Account for []

        for item in arr {
            let item_size = serde_json::to_string(&**item).map(|s| s.len()).unwrap_or(0);

            if current_size + item_size + 1 > max_bytes && !current_chunk.is_empty() {
                chunks.push(Rc::new(Variable::Array(current_chunk)) as Rcvar);
                current_chunk = Vec::new();
                current_size = 2;
            }

            current_chunk.push(item.clone());
            current_size += item_size + 1; // +1 for comma
        }

        if !current_chunk.is_empty() {
            chunks.push(Rc::new(Variable::Array(current_chunk)) as Rcvar);
        }

        Ok(Rc::new(Variable::Array(chunks)))
    }
}

// =============================================================================
// paginate(array, page, per_page) -> object with pagination metadata
// =============================================================================

define_function!(
    PaginateFn,
    vec![
        ArgumentType::Array,
        ArgumentType::Number,
        ArgumentType::Number
    ],
    None
);

impl Function for PaginateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array".to_owned()),
            )
        })?;

        let page = args[1].as_number().unwrap_or(1.0).max(1.0) as usize;
        let per_page = args[2].as_number().unwrap_or(10.0).max(1.0) as usize;

        let total = arr.len();
        let total_pages = total.div_ceil(per_page);
        let start = (page - 1) * per_page;
        let end = (start + per_page).min(total);

        let data: Vec<Rcvar> = if start < total {
            arr[start..end].to_vec()
        } else {
            vec![]
        };

        let mut result = BTreeMap::new();
        result.insert("data".to_string(), Rc::new(Variable::Array(data)) as Rcvar);
        result.insert(
            "page".to_string(),
            Rc::new(Variable::Number((page as i64).into())) as Rcvar,
        );
        result.insert(
            "per_page".to_string(),
            Rc::new(Variable::Number((per_page as i64).into())) as Rcvar,
        );
        result.insert(
            "total".to_string(),
            Rc::new(Variable::Number((total as i64).into())) as Rcvar,
        );
        result.insert(
            "total_pages".to_string(),
            Rc::new(Variable::Number((total_pages as i64).into())) as Rcvar,
        );
        result.insert(
            "has_next".to_string(),
            Rc::new(Variable::Bool(page < total_pages)) as Rcvar,
        );
        result.insert(
            "has_prev".to_string(),
            Rc::new(Variable::Bool(page > 1)) as Rcvar,
        );

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// estimate_size(any) -> number (JSON serialized byte size)
// =============================================================================

define_function!(EstimateSizeFn, vec![ArgumentType::Any], None);

impl Function for EstimateSizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let size = serde_json::to_string(&*args[0])
            .map(|s| s.len())
            .unwrap_or(0);
        Ok(Rc::new(Variable::Number((size as i64).into())))
    }
}

// =============================================================================
// truncate_to_size(any, max_bytes) -> any (truncate to fit within byte limit)
// =============================================================================

define_function!(
    TruncateToSizeFn,
    vec![ArgumentType::Any, ArgumentType::Number],
    None
);

impl Function for TruncateToSizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let max_bytes = args[1].as_number().unwrap_or(1000.0) as usize;
        let current_size = serde_json::to_string(&*args[0])
            .map(|s| s.len())
            .unwrap_or(0);

        if current_size <= max_bytes {
            return Ok(args[0].clone());
        }

        // For arrays, truncate elements
        if let Some(arr) = args[0].as_array() {
            let mut result: Vec<Rcvar> = Vec::new();
            let mut size = 2; // []

            for item in arr {
                let item_size = serde_json::to_string(&**item).map(|s| s.len()).unwrap_or(0);
                if size + item_size + 1 > max_bytes {
                    break;
                }
                result.push(item.clone());
                size += item_size + 1;
            }
            return Ok(Rc::new(Variable::Array(result)));
        }

        // For strings, truncate characters
        if let Some(s) = args[0].as_string() {
            let target_len = max_bytes.saturating_sub(2); // Account for quotes
            let truncated: String = s.chars().take(target_len).collect();
            return Ok(Rc::new(Variable::String(truncated)));
        }

        // For other types, return as-is (can't easily truncate)
        Ok(args[0].clone())
    }
}

// =============================================================================
// template(object, template_string) -> string (mustache-style expansion)
// =============================================================================

define_function!(
    TemplateFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

impl Function for TemplateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let template = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected template string".to_owned()),
            )
        })?;

        let result = expand_template(&args[0], template, false)
            .map_err(|e| JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e)))?;
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// template_strict(object, template_string) -> string (error on missing vars)
// =============================================================================

define_function!(
    TemplateStrictFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

impl Function for TemplateStrictFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let template = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected template string".to_owned()),
            )
        })?;

        let result = expand_template(&args[0], template, true)
            .map_err(|e| JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e)))?;
        Ok(Rc::new(Variable::String(result)))
    }
}

fn expand_template(data: &Variable, template: &str, strict: bool) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {

            // Collect variable name
            let mut var_name = String::new();
            let mut fallback: Option<String> = None;

            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next();
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        break;
                    }
                } else if next == '|' {
                    chars.next();
                    // Collect fallback value
                    let mut fb = String::new();
                    while let Some(&fc) = chars.peek() {
                        if fc == '}' {
                            break;
                        }
                        fb.push(chars.next().unwrap());
                    }
                    fallback = Some(fb);
                } else {
                    var_name.push(chars.next().unwrap());
                }
            }

            // Look up variable
            let value = get_template_value(data, &var_name);

            match value {
                Some(v) => result.push_str(&variable_to_string(&v)),
                None => {
                    if strict {
                        return Err(format!("missing variable '{}'", var_name));
                    }
                    if let Some(fb) = fallback {
                        result.push_str(&fb);
                    }
                }
            }
        } else if c == '\\' && chars.peek() == Some(&'{') {
            // Escape sequence
            result.push(chars.next().unwrap());
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn get_template_value(data: &Variable, path: &str) -> Option<Rcvar> {
    let parts: Vec<&str> = path.trim().split('.').collect();
    let mut current: Rcvar = Rc::new(data.clone());

    for part in parts {
        // Check for array index
        if let Ok(idx) = part.parse::<usize>() {
            if let Some(arr) = current.as_array() {
                if idx < arr.len() {
                    current = arr[idx].clone();
                    continue;
                }
            }
            return None;
        }

        // Object key access
        if let Some(obj) = current.as_object() {
            if let Some(val) = obj.get(part) {
                current = val.clone();
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    if current.is_null() {
        None
    } else {
        Some(current)
    }
}

fn variable_to_string(value: &Variable) -> String {
    match value {
        Variable::String(s) => s.clone(),
        Variable::Number(n) => n.to_string(),
        Variable::Bool(b) => b.to_string(),
        Variable::Null => String::new(),
        _ => serde_json::to_string(value).unwrap_or_default(),
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
    fn test_items() {
        let runtime = setup_runtime();
        let expr = runtime.compile("items(@)").unwrap();
        let data = Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // JEP-013: Each item should be a [key, value] pair
        let first = arr[0].as_array().unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].as_string().unwrap(), "a");
        assert_eq!(first[1].as_number().unwrap() as i64, 1);
    }

    #[test]
    fn test_items_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("items(@)").unwrap();
        let data = Variable::from_json(r#"{}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_from_items() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(@)").unwrap();
        let data = Variable::from_json(r#"[["a", 1], ["b", 2]]"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_from_items_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(@)").unwrap();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_from_items_duplicate_keys() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(@)").unwrap();
        let data = Variable::from_json(r#"[["x", 1], ["x", 2]]"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        // Last value wins
        assert_eq!(obj.get("x").unwrap().as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_items_from_items_roundtrip() {
        let runtime = setup_runtime();
        let expr = runtime.compile("from_items(items(@))").unwrap();
        let data = Variable::from_json(r#"{"a": 1, "b": "hello", "c": true}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_string().unwrap(), "hello");
        assert!(obj.get("c").unwrap().as_boolean().unwrap());
    }

    #[test]
    fn test_with_entries_identity() {
        let runtime = setup_runtime();
        let expr = runtime.compile("with_entries(@, '[@[0], @[1]]')").unwrap();
        let data = Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 1);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_with_entries_transform_keys() {
        // Test that we can transform keys by prepending a prefix
        // Using join to concatenate strings (built-in)
        let runtime = setup_runtime();
        let expr = runtime
            .compile(r#"with_entries(@, '[join(`""`, [`"prefix_"`, @[0]]), @[1]]')"#)
            .unwrap();
        let data = Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("prefix_a"));
        assert!(obj.contains_key("prefix_b"));
    }

    #[test]
    fn test_with_entries_swap_key_value() {
        // Test swapping keys and values (for string values)
        let runtime = setup_runtime();
        let expr = runtime.compile("with_entries(@, '[@[1], @[0]]')").unwrap();
        let data = Variable::from_json(r#"{"a": "x", "b": "y"}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("x").unwrap().as_string().unwrap(), "a");
        assert_eq!(obj.get("y").unwrap().as_string().unwrap(), "b");
    }

    #[test]
    fn test_with_entries_filter_null() {
        // Test that returning null from the expression skips entries
        // This tests the filtering behavior of with_entries
        let runtime = setup_runtime();
        // Return null for all entries - result should be empty object
        let expr = runtime.compile(r#"with_entries(@, '`null`')"#).unwrap();
        let data = Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_with_entries_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("with_entries(@, '[@[0], @[1]]')").unwrap();
        let data = Variable::from_json(r#"{}"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_pick() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pick(@, `[\"a\"]`)").unwrap();
        let mut obj = BTreeMap::new();
        obj.insert(
            "a".to_string(),
            Rc::new(Variable::Number(serde_json::Number::from(1))),
        );
        obj.insert(
            "b".to_string(),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        );
        let data = Variable::Object(obj);
        let result = expr.search(&data).unwrap();
        let result_obj = result.as_object().unwrap();
        assert_eq!(result_obj.len(), 1);
        assert!(result_obj.contains_key("a"));
    }

    #[test]
    fn test_deep_equals_objects() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}, "c": {"b": 1}}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_deep_equals_objects_different() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}, "c": {"b": 2}}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_deep_equals_arrays() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, [2, 3]], "b": [1, [2, 3]]}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_deep_equals_arrays_order_matters() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2], "b": [2, 1]}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_deep_equals_primitives() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": "hello", "b": "hello", "c": "world"}"#).unwrap();

        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_deep_diff_added() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 1, "y": 2}}"#).unwrap();
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        let added = diff.get("added").unwrap().as_object().unwrap();
        assert!(added.contains_key("y"));
        assert!(diff.get("removed").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("changed").unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_deep_diff_removed() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1, "y": 2}, "b": {"x": 1}}"#).unwrap();
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        let removed = diff.get("removed").unwrap().as_object().unwrap();
        assert!(removed.contains_key("y"));
        assert!(diff.get("added").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("changed").unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_deep_diff_changed() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 2}}"#).unwrap();
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        let changed = diff.get("changed").unwrap().as_object().unwrap();
        assert!(changed.contains_key("x"));
        let x_change = changed.get("x").unwrap().as_object().unwrap();
        assert!(x_change.contains_key("from"));
        assert!(x_change.contains_key("to"));
    }

    #[test]
    fn test_deep_diff_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": {"y": 1}}, "b": {"x": {"y": 2}}}"#).unwrap();
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        // The change should be nested under x
        let changed = diff.get("changed").unwrap().as_object().unwrap();
        assert!(changed.contains_key("x"));
    }

    #[test]
    fn test_deep_diff_no_changes() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 1}}"#).unwrap();
        let expr = runtime.compile("deep_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let diff = result.as_object().unwrap();

        assert!(diff.get("added").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("removed").unwrap().as_object().unwrap().is_empty());
        assert!(diff.get("changed").unwrap().as_object().unwrap().is_empty());
    }

    #[test]
    fn test_get_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": {"c": 1}}}"#).unwrap();
        let expr = runtime.compile("get(@, 'a.b.c')").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_get_with_default() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let expr = runtime.compile("get(@, 'b.c', 'default')").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "default");
    }

    #[test]
    fn test_get_array_index() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [{"b": 1}, {"b": 2}]}"#).unwrap();
        let expr = runtime.compile("get(@, 'a[0].b')").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_get_missing_returns_null() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let expr = runtime.compile("get(@, 'x.y.z')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_has_exists() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}}"#).unwrap();
        let expr = runtime.compile("has(@, 'a.b')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_has_not_exists() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let expr = runtime.compile("has(@, 'a.b.c')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_has_array_index() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("has(@, 'a[1]')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_has_array_index_out_of_bounds() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2]}"#).unwrap();
        let expr = runtime.compile("has(@, 'a[5]')").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_defaults_shallow() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"obj": {"a": 1}, "defs": {"a": 2, "b": 3}}"#).unwrap();
        let expr = runtime.compile("defaults(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 1.0); // original kept
        assert_eq!(obj.get("b").unwrap().as_number().unwrap(), 3.0); // default added
    }

    #[test]
    fn test_defaults_empty_object() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"obj": {}, "defs": {"a": 1, "b": 2}}"#).unwrap();
        let expr = runtime.compile("defaults(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 1.0);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_defaults_deep_nested() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"obj": {"a": {"b": 1}}, "defs": {"a": {"b": 2, "c": 3}}}"#)
                .unwrap();
        let expr = runtime.compile("defaults_deep(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.get("b").unwrap().as_number().unwrap(), 1.0); // original kept
        assert_eq!(a.get("c").unwrap().as_number().unwrap(), 3.0); // default added
    }

    #[test]
    fn test_defaults_deep_new_nested() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"obj": {"x": 1}, "defs": {"x": 2, "y": {"z": 3}}}"#).unwrap();
        let expr = runtime.compile("defaults_deep(obj, defs)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("x").unwrap().as_number().unwrap(), 1.0); // original kept
        let y = obj.get("y").unwrap().as_object().unwrap();
        assert_eq!(y.get("z").unwrap().as_number().unwrap(), 3.0); // default added
    }

    #[test]
    fn test_set_path_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        let expr = runtime.compile("set_path(@, '/c', `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 1.0);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap(), 2.0);
        assert_eq!(obj.get("c").unwrap().as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_set_path_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}}"#).unwrap();
        let expr = runtime.compile("set_path(@, '/a/c', `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.get("b").unwrap().as_number().unwrap(), 1.0);
        assert_eq!(a.get("c").unwrap().as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_set_path_create_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{}"#).unwrap();
        let expr = runtime
            .compile("set_path(@, '/a/b/c', `\"deep\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        let b = a.get("b").unwrap().as_object().unwrap();
        assert_eq!(b.get("c").unwrap().as_string().unwrap(), "deep");
    }

    #[test]
    fn test_set_path_array_index() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("set_path(@, '/items/1', `99`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items[0].as_number().unwrap(), 1.0);
        assert_eq!(items[1].as_number().unwrap(), 99.0);
        assert_eq!(items[2].as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_delete_path_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
        let expr = runtime.compile("delete_path(@, '/b')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        assert!(!obj.contains_key("b"));
    }

    #[test]
    fn test_delete_path_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1, "c": 2}}"#).unwrap();
        let expr = runtime.compile("delete_path(@, '/a/b')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a.len(), 1);
        assert!(a.contains_key("c"));
        assert!(!a.contains_key("b"));
    }

    #[test]
    fn test_delete_path_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("delete_path(@, '/items/1')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_number().unwrap(), 1.0);
        assert_eq!(items[1].as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_paths_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}, "c": 2}"#).unwrap();
        let expr = runtime.compile("paths(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let paths = result.as_array().unwrap();
        assert!(paths.len() >= 3); // /a, /a/b, /c
    }

    #[test]
    fn test_paths_with_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"items": [1, 2]}"#).unwrap();
        let expr = runtime.compile("paths(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let paths: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_string().unwrap().to_string())
            .collect();
        assert!(paths.contains(&"/items".to_string()));
        assert!(paths.contains(&"/items/0".to_string()));
        assert!(paths.contains(&"/items/1".to_string()));
    }

    #[test]
    fn test_leaves_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": {"c": 2}, "d": [3, 4]}"#).unwrap();
        let expr = runtime.compile("leaves(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let leaves = result.as_array().unwrap();
        assert_eq!(leaves.len(), 4); // 1, 2, 3, 4
    }

    #[test]
    fn test_leaves_strings() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"name": "alice", "tags": ["a", "b"]}"#).unwrap();
        let expr = runtime.compile("leaves(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let leaves = result.as_array().unwrap();
        assert_eq!(leaves.len(), 3); // "alice", "a", "b"
    }

    #[test]
    fn test_leaves_with_paths_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": {"c": 2}}"#).unwrap();
        let expr = runtime.compile("leaves_with_paths(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let leaves = result.as_array().unwrap();
        assert_eq!(leaves.len(), 2);
        // Each leaf should have path and value
        let first = leaves[0].as_object().unwrap();
        assert!(first.contains_key("path"));
        assert!(first.contains_key("value"));
    }

    #[test]
    fn test_set_path_immutable() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let expr = runtime.compile("set_path(@, '/b', `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        // Original should be unchanged (immutable semantics)
        let original = data.as_object().unwrap();
        assert!(!original.contains_key("b"));
        // Result should have the new key
        let new_obj = result.as_object().unwrap();
        assert!(new_obj.contains_key("b"));
    }

    // =========================================================================
    // Dot notation path tests (for set_path, delete_path, get_path, has_path)
    // =========================================================================

    #[test]
    fn test_set_path_dot_notation() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"c": 1}}"#).unwrap();
        let expr = runtime.compile("set_path(@, `\"a.b\"`, `99`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let nested = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(nested.get("b").unwrap().as_number().unwrap() as i64, 99);
        assert_eq!(nested.get("c").unwrap().as_number().unwrap() as i64, 1);
    }

    #[test]
    fn test_set_path_dot_notation_deep() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{}"#).unwrap();
        let expr = runtime
            .compile("set_path(@, `\"a.b.c\"`, `\"deep\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let a = obj.get("a").unwrap().as_object().unwrap();
        let b = a.get("b").unwrap().as_object().unwrap();
        assert_eq!(b.get("c").unwrap().as_string().unwrap(), "deep");
    }

    #[test]
    fn test_set_path_dot_notation_array_index() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("set_path(@, `\"items.1\"`, `99`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items[1].as_number().unwrap() as i64, 99);
    }

    #[test]
    fn test_delete_path_dot_notation() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1, "c": 2}}"#).unwrap();
        let expr = runtime.compile("delete_path(@, `\"a.b\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let nested = obj.get("a").unwrap().as_object().unwrap();
        assert!(!nested.contains_key("b"));
        assert!(nested.contains_key("c"));
    }

    #[test]
    fn test_delete_path_dot_notation_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("delete_path(@, `\"items.1\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_number().unwrap() as i64, 1);
        assert_eq!(items[1].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_get_path_alias() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": {"c": 42}}}"#).unwrap();
        let expr = runtime.compile("get_path(@, `\"a.b.c\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 42);
    }

    #[test]
    fn test_get_path_with_default() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let expr = runtime
            .compile("get_path(@, `\"a.b.c\"`, `\"default\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "default");
    }

    #[test]
    fn test_get_path_array_index() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#).unwrap();
        let expr = runtime.compile("get_path(@, `\"users.0.name\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "alice");
    }

    #[test]
    fn test_get_path_array_index_out_of_bounds() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"users": [{"name": "alice"}]}"#).unwrap();
        let expr = runtime
            .compile("get_path(@, `\"users.5.name\"`, `\"unknown\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "unknown");
    }

    #[test]
    fn test_has_path_alias() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}}"#).unwrap();
        let expr = runtime.compile("has_path(@, `\"a.b\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_has_path_missing() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}}"#).unwrap();
        let expr = runtime.compile("has_path(@, `\"a.c\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_has_path_array_index() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"items": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("has_path(@, `\"items.1\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    // =========================================================================
    // remove_nulls tests
    // =========================================================================

    #[test]
    fn test_remove_nulls_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": null, "c": 2}"#).unwrap();
        let expr = runtime.compile("remove_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        assert!(!obj.contains_key("b"));
    }

    #[test]
    fn test_remove_nulls_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": {"c": null, "d": 2}}"#).unwrap();
        let expr = runtime.compile("remove_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let nested = obj.get("b").unwrap().as_object().unwrap();
        assert_eq!(nested.len(), 1);
        assert!(nested.contains_key("d"));
        assert!(!nested.contains_key("c"));
    }

    #[test]
    fn test_remove_nulls_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, null, 2, null, 3]"#).unwrap();
        let expr = runtime.compile("remove_nulls(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    // =========================================================================
    // remove_empty tests
    // =========================================================================

    #[test]
    fn test_remove_empty_basic() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"a": "", "b": [], "c": {}, "d": null, "e": "hello"}"#).unwrap();
        let expr = runtime.compile("remove_empty(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("e"));
    }

    #[test]
    fn test_remove_empty_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": "", "c": 1}, "d": []}"#).unwrap();
        let expr = runtime.compile("remove_empty(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        let nested = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(nested.len(), 1);
        assert!(nested.contains_key("c"));
    }

    #[test]
    fn test_remove_empty_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["", "hello", [], null, "world"]"#).unwrap();
        let expr = runtime.compile("remove_empty(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    // =========================================================================
    // remove_empty_strings tests
    // =========================================================================

    #[test]
    fn test_remove_empty_strings_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"name": "alice", "bio": "", "age": 30}"#).unwrap();
        let expr = runtime.compile("remove_empty_strings(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("age"));
        assert!(!obj.contains_key("bio"));
    }

    #[test]
    fn test_remove_empty_strings_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["hello", "", "world", ""]"#).unwrap();
        let expr = runtime.compile("remove_empty_strings(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    // =========================================================================
    // compact_deep tests
    // =========================================================================

    #[test]
    fn test_compact_deep_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, null], [null, 2]]"#).unwrap();
        let expr = runtime.compile("compact_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let first = arr[0].as_array().unwrap();
        assert_eq!(first.len(), 1);
        let second = arr[1].as_array().unwrap();
        assert_eq!(second.len(), 1);
    }

    #[test]
    fn test_compact_deep_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, null], [null, [2, null, 3]]]"#).unwrap();
        let expr = runtime.compile("compact_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let second = arr[1].as_array().unwrap();
        let inner = second[0].as_array().unwrap();
        assert_eq!(inner.len(), 2); // [2, 3]
    }

    // =========================================================================
    // completeness tests
    // =========================================================================

    #[test]
    fn test_completeness_all_filled() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": "hello", "c": true}"#).unwrap();
        let expr = runtime.compile("completeness(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let score = result.as_number().unwrap();
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_completeness_with_nulls() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": null, "c": null}"#).unwrap();
        let expr = runtime.compile("completeness(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let score = result.as_number().unwrap();
        // 1 out of 3 fields is non-null = 33.33%
        assert!((score - 33.33).abs() < 1.0);
    }

    #[test]
    fn test_completeness_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": {"c": null, "d": 2}, "e": null}"#).unwrap();
        let expr = runtime.compile("completeness(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let score = result.as_number().unwrap();
        // 5 total fields: a(1), b(obj), b.c(null), b.d(2), e(null)
        // 3 non-null: a, b, b.d
        // 3/5 = 60%
        assert!((score - 60.0).abs() < 1.0);
    }

    // =========================================================================
    // type_consistency tests
    // =========================================================================

    #[test]
    fn test_type_consistency_consistent() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("type_consistency(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("consistent").unwrap().as_boolean().unwrap());
    }

    #[test]
    fn test_type_consistency_inconsistent() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, "two", 3]"#).unwrap();
        let expr = runtime.compile("type_consistency(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(!obj.get("consistent").unwrap().as_boolean().unwrap());
    }

    #[test]
    fn test_type_consistency_object_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": "unknown"}]"#,
        )
        .unwrap();
        let expr = runtime.compile("type_consistency(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(!obj.get("consistent").unwrap().as_boolean().unwrap());
        let inconsistencies = obj.get("inconsistencies").unwrap().as_array().unwrap();
        assert_eq!(inconsistencies.len(), 1);
    }

    // =========================================================================
    // data_quality_score tests
    // =========================================================================

    #[test]
    fn test_data_quality_score_perfect() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": "hello"}"#).unwrap();
        let expr = runtime.compile("data_quality_score(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let score = obj.get("score").unwrap().as_number().unwrap();
        assert_eq!(score, 100.0);
        assert_eq!(
            obj.get("null_count").unwrap().as_number().unwrap() as i64,
            0
        );
    }

    #[test]
    fn test_data_quality_score_with_issues() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1, "b": null, "c": ""}"#).unwrap();
        let expr = runtime.compile("data_quality_score(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("null_count").unwrap().as_number().unwrap() as i64,
            1
        );
        assert_eq!(
            obj.get("empty_string_count").unwrap().as_number().unwrap() as i64,
            1
        );
        let issues = obj.get("issues").unwrap().as_array().unwrap();
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_data_quality_score_type_mismatch() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"users": [{"age": 30}, {"age": "thirty"}]}"#).unwrap();
        let expr = runtime.compile("data_quality_score(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("type_inconsistencies")
                .unwrap()
                .as_number()
                .unwrap() as i64,
            1
        );
    }

    // =========================================================================
    // redact tests
    // =========================================================================

    #[test]
    fn test_redact_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"{"name": "alice", "password": "secret123", "ssn": "123-45-6789"}"#,
        )
        .unwrap();
        let expr = runtime
            .compile(r#"redact(@, `["password", "ssn"]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "alice");
        assert_eq!(
            obj.get("password").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(obj.get("ssn").unwrap().as_string().unwrap(), "[REDACTED]");
    }

    #[test]
    fn test_redact_nested() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"user": {"name": "bob", "password": "secret"}}"#).unwrap();
        let expr = runtime.compile(r#"redact(@, `["password"]`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let user = obj.get("user").unwrap().as_object().unwrap();
        assert_eq!(user.get("name").unwrap().as_string().unwrap(), "bob");
        assert_eq!(
            user.get("password").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
    }

    #[test]
    fn test_redact_array_of_objects() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"[{"name": "alice", "token": "abc"}, {"name": "bob", "token": "xyz"}]"#,
        )
        .unwrap();
        let expr = runtime.compile(r#"redact(@, `["token"]`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let first = arr[0].as_object().unwrap();
        assert_eq!(
            first.get("token").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
    }

    // =========================================================================
    // mask tests
    // =========================================================================

    #[test]
    fn test_mask_default() {
        let runtime = setup_runtime();
        let data = Variable::String("4111111111111111".to_string());
        let expr = runtime.compile("mask(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "************1111");
    }

    #[test]
    fn test_mask_custom_length() {
        let runtime = setup_runtime();
        let data = Variable::String("555-123-4567".to_string());
        let expr = runtime.compile("mask(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "*********567");
    }

    #[test]
    fn test_mask_short_string() {
        let runtime = setup_runtime();
        let data = Variable::String("abc".to_string());
        let expr = runtime.compile("mask(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // If string is shorter than show_last, mask everything
        assert_eq!(result.as_string().unwrap(), "***");
    }

    // =========================================================================
    // redact_keys tests
    // =========================================================================

    #[test]
    #[cfg(feature = "regex")]
    fn test_redact_keys_basic() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"password": "secret", "api_key": "abc123", "name": "test"}"#)
                .unwrap();
        let expr = runtime
            .compile(r#"redact_keys(@, `"password|api_key"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("password").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(
            obj.get("api_key").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "test");
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_redact_keys_pattern() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"secret_key": "a", "secret_token": "b", "name": "test"}"#)
                .unwrap();
        let expr = runtime.compile(r#"redact_keys(@, `"secret.*"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("secret_key").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(
            obj.get("secret_token").unwrap().as_string().unwrap(),
            "[REDACTED]"
        );
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "test");
    }

    // =========================================================================
    // pluck_deep tests
    // =========================================================================

    #[test]
    fn test_pluck_deep_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"users": [{"id": 1}, {"id": 2}], "meta": {"id": 99}}"#)
            .unwrap();
        let expr = runtime.compile(r#"pluck_deep(@, `"id"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_pluck_deep_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": {"c": 1}}, "d": {"c": 2}}"#).unwrap();
        let expr = runtime.compile(r#"pluck_deep(@, `"c"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_pluck_deep_not_found() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": 1}"#).unwrap();
        let expr = runtime.compile(r#"pluck_deep(@, `"x"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // paths_to tests
    // =========================================================================

    #[test]
    fn test_paths_to_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"id": 1}, "b": {"id": 2}}"#).unwrap();
        let expr = runtime.compile(r#"paths_to(@, `"id"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let paths: Vec<String> = arr
            .iter()
            .map(|p| p.as_string().unwrap().to_string())
            .collect();
        assert!(paths.contains(&"a.id".to_string()));
        assert!(paths.contains(&"b.id".to_string()));
    }

    #[test]
    fn test_paths_to_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"users": [{"id": 1}]}"#).unwrap();
        let expr = runtime.compile(r#"paths_to(@, `"id"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_string().unwrap(), "users.0.id");
    }

    // =========================================================================
    // snake_keys tests
    // =========================================================================

    #[test]
    fn test_snake_keys_camel() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"userName": "alice"}"#).unwrap();
        let expr = runtime.compile("snake_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user_name"));
        assert_eq!(obj.get("user_name").unwrap().as_string().unwrap(), "alice");
    }

    #[test]
    fn test_snake_keys_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"userInfo": {"firstName": "bob"}}"#).unwrap();
        let expr = runtime.compile("snake_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user_info"));
        let nested = obj.get("user_info").unwrap().as_object().unwrap();
        assert!(nested.contains_key("first_name"));
    }

    // =========================================================================
    // camel_keys tests
    // =========================================================================

    #[test]
    fn test_camel_keys_snake() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"user_name": "alice"}"#).unwrap();
        let expr = runtime.compile("camel_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("userName"));
        assert_eq!(obj.get("userName").unwrap().as_string().unwrap(), "alice");
    }

    #[test]
    fn test_camel_keys_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"user_info": {"first_name": "bob"}}"#).unwrap();
        let expr = runtime.compile("camel_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("userInfo"));
        let nested = obj.get("userInfo").unwrap().as_object().unwrap();
        assert!(nested.contains_key("firstName"));
    }

    // =========================================================================
    // kebab_keys tests
    // =========================================================================

    #[test]
    fn test_kebab_keys_camel() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"userName": "alice"}"#).unwrap();
        let expr = runtime.compile("kebab_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user-name"));
        assert_eq!(obj.get("user-name").unwrap().as_string().unwrap(), "alice");
    }

    #[test]
    fn test_kebab_keys_snake() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"user_name": "bob"}"#).unwrap();
        let expr = runtime.compile("kebab_keys(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("user-name"));
        assert_eq!(obj.get("user-name").unwrap().as_string().unwrap(), "bob");
    }

    // =========================================================================
    // structural_diff tests
    // =========================================================================

    #[test]
    fn test_structural_diff_added() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 1, "y": 2}}"#).unwrap();
        let expr = runtime.compile("structural_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let added = obj.get("added").unwrap().as_array().unwrap();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].as_string().unwrap(), "y");
    }

    #[test]
    fn test_structural_diff_removed() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1, "y": 2}, "b": {"x": 1}}"#).unwrap();
        let expr = runtime.compile("structural_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let removed = obj.get("removed").unwrap().as_array().unwrap();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].as_string().unwrap(), "y");
    }

    #[test]
    fn test_structural_diff_type_changed() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": "string"}}"#).unwrap();
        let expr = runtime.compile("structural_diff(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let type_changed = obj.get("type_changed").unwrap().as_array().unwrap();
        assert_eq!(type_changed.len(), 1);
    }

    #[test]
    fn test_has_same_shape_true() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"x": 2}}"#).unwrap();
        let expr = runtime.compile("has_same_shape(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_has_same_shape_false() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"x": 1}, "b": {"y": 2}}"#).unwrap();
        let expr = runtime.compile("has_same_shape(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    // =========================================================================
    // infer_schema tests
    // =========================================================================

    #[test]
    fn test_infer_schema_object() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"name": "alice", "age": 30}"#).unwrap();
        let expr = runtime.compile("infer_schema(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let schema = result.as_object().unwrap();
        assert_eq!(schema.get("type").unwrap().as_string().unwrap(), "object");
        let props = schema.get("properties").unwrap().as_object().unwrap();
        assert!(props.contains_key("name"));
        assert!(props.contains_key("age"));
    }

    #[test]
    fn test_infer_schema_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("infer_schema(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let schema = result.as_object().unwrap();
        assert_eq!(schema.get("type").unwrap().as_string().unwrap(), "array");
        let items = schema.get("items").unwrap().as_object().unwrap();
        assert_eq!(items.get("type").unwrap().as_string().unwrap(), "number");
    }

    // =========================================================================
    // chunk_by_size tests
    // =========================================================================

    #[test]
    fn test_chunk_by_size() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("chunk_by_size(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let chunks = result.as_array().unwrap();
        assert!(chunks.len() > 1); // Should be split into multiple chunks
    }

    // =========================================================================
    // paginate tests
    // =========================================================================

    #[test]
    fn test_paginate() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#).unwrap();
        let expr = runtime.compile("paginate(@, `2`, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();

        let page_data = obj.get("data").unwrap().as_array().unwrap();
        assert_eq!(page_data.len(), 3);
        assert_eq!(page_data[0].as_number().unwrap() as i64, 4); // Page 2 starts at index 3

        assert_eq!(obj.get("page").unwrap().as_number().unwrap() as i64, 2);
        assert_eq!(obj.get("total").unwrap().as_number().unwrap() as i64, 10);
        assert_eq!(
            obj.get("total_pages").unwrap().as_number().unwrap() as i64,
            4
        );
        assert!(obj.get("has_next").unwrap().as_boolean().unwrap());
        assert!(obj.get("has_prev").unwrap().as_boolean().unwrap());
    }

    // =========================================================================
    // estimate_size tests
    // =========================================================================

    #[test]
    fn test_estimate_size() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"hello": "world"}"#).unwrap();
        let expr = runtime.compile("estimate_size(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let size = result.as_number().unwrap() as i64;
        assert!(size > 0);
    }

    // =========================================================================
    // truncate_to_size tests
    // =========================================================================

    #[test]
    fn test_truncate_to_size_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("truncate_to_size(@, `5`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.len() < 5); // Should be truncated
    }

    // =========================================================================
    // template tests
    // =========================================================================

    #[test]
    fn test_template_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"name": "alice", "age": 30}"#).unwrap();
        let expr = runtime
            .compile(r#"template(@, `"Hello {{name}}, you are {{age}} years old"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(
            result.as_string().unwrap(),
            "Hello alice, you are 30 years old"
        );
    }

    #[test]
    fn test_template_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"user": {"name": "bob"}}"#).unwrap();
        let expr = runtime
            .compile(r#"template(@, `"Welcome {{user.name}}!"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Welcome bob!");
    }

    #[test]
    fn test_template_missing_default() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"name": "alice"}"#).unwrap();
        let expr = runtime
            .compile(r#"template(@, `"{{name}} - {{title}}"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "alice - ");
    }

    #[test]
    fn test_template_fallback() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{}"#).unwrap();
        let expr = runtime
            .compile(r#"template(@, `"Hello {{name|Guest}}"`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "Hello Guest");
    }
}
