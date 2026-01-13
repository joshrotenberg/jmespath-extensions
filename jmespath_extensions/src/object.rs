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

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all object functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("items", Box::new(EntriesFn::new()));
    runtime.register_function("from_items", Box::new(FromEntriesFn::new()));
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
    runtime.register_function("has", Box::new(HasFn::new()));
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
    runtime.register_function("redact_keys", Box::new(RedactKeysFn::new()));
    // Recursive key search and transformation
    runtime.register_function("pluck_deep", Box::new(PluckDeepFn::new()));
    runtime.register_function("paths_to", Box::new(PathsToFn::new()));
    runtime.register_function("snake_keys", Box::new(SnakeKeysFn::new()));
    runtime.register_function("camel_keys", Box::new(CamelKeysFn::new()));
    runtime.register_function("kebab_keys", Box::new(KebabKeysFn::new()));
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

// Navigate to a value using a path string like "a.b.c" or "a[0].b"
fn get_at_path(value: &Variable, path: &str) -> Option<Rcvar> {
    if path.is_empty() {
        return Some(Rc::new(value.clone()));
    }

    let mut current: Rcvar = Rc::new(value.clone());

    // Split path by dots, but handle array indices
    let parts = parse_path_parts(path);

    for part in parts {
        if let Some(idx) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            // Array index access
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
// set_path(object, path, value) -> new object with value set at JSON pointer path
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

        // Parse JSON pointer path (RFC 6901)
        let parts = parse_json_pointer(path);
        if parts.is_empty() {
            // Empty path means replace the entire value
            return Ok(value);
        }

        // Deep clone and set value at path
        let result = set_at_path(&args[0], &parts, value);
        Ok(result)
    }
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
// delete_path(object, path) -> new object with value removed at JSON pointer path
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

        let parts = parse_json_pointer(path);
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

define_function!(
    RedactKeysFn,
    vec![ArgumentType::Any, ArgumentType::String],
    None
);

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
// =============================================================================

define_function!(SnakeKeysFn, vec![ArgumentType::Any], None);

impl Function for SnakeKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], to_snake_case)))
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

// =============================================================================
// camel_keys(any) -> any (recursively convert all keys to camelCase)
// =============================================================================

define_function!(CamelKeysFn, vec![ArgumentType::Any], None);

impl Function for CamelKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], to_camel_case)))
    }
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

// =============================================================================
// kebab_keys(any) -> any (recursively convert all keys to kebab-case)
// =============================================================================

define_function!(KebabKeysFn, vec![ArgumentType::Any], None);

impl Function for KebabKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        Ok(Rc::new(transform_keys_recursive(&args[0], to_kebab_case)))
    }
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else if c == '_' || c == ' ' {
            result.push('-');
        } else {
            result.push(c);
        }
    }
    result
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
}
