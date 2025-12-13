//! Object manipulation functions.
//!
//! This module provides extended object operations beyond the standard JMESPath built-ins.
//!
//! # Function Reference
//!
//! | Function | Signature | Description |
//! |----------|-----------|-------------|
//! | [`items`](#items) | `items(object) → array` | Convert to [[key, value], ...] (JEP-013) |
//! | [`from_items`](#from_items) | `from_items(array) → object` | Convert [[key, value], ...] to object (JEP-013) |
//! | [`pick`](#pick) | `pick(object, keys) → object` | Select specific keys |
//! | [`omit`](#omit) | `omit(object, keys) → object` | Remove specific keys |
//! | [`invert`](#invert) | `invert(object) → object` | Swap keys and values |
//! | [`rename_keys`](#rename_keys) | `rename_keys(object, mapping) → object` | Rename keys |
//! | [`flatten_keys`](#flatten_keys) | `flatten_keys(object, sep?) → object` | Flatten nested object |
//! | [`unflatten_keys`](#unflatten_keys) | `unflatten_keys(object, sep?) → object` | Unflatten to nested |
//! | [`deep_merge`](#deep_merge) | `deep_merge(obj1, obj2) → object` | Deep merge two objects |
//! | [`deep_equals`](#deep_equals) | `deep_equals(a, b) → boolean` | Deep equality check |
//! | [`deep_diff`](#deep_diff) | `deep_diff(a, b) → object` | Structural diff between values |
//! | [`get`](#get) | `get(object, path, default?) → any` | Get value at path with optional default |
//! | [`has`](#has) | `has(object, path) → boolean` | Check if path exists |
//! | [`defaults`](#defaults) | `defaults(object, defaults) → object` | Assign defaults for missing keys |
//! | [`defaults_deep`](#defaults_deep) | `defaults_deep(object, defaults) → object` | Recursive defaults |
//!
//! # Examples
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::object;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! object::register(&mut runtime);
//!
//! // Get object items
//! let expr = runtime.compile("items(@)").unwrap();
//! let data = Variable::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_array().unwrap().len(), 2);
//! ```
//!
//! # Function Details
//!
//! ## items
//!
//! Converts an object to an array of key-value pairs (2-element arrays).
//! Implements [JEP-013](https://github.com/jmespath-community/jmespath.spec/discussions/47).
//!
//! This function is the inverse of [`from_items`](#from_items).
//!
//! ```text
//! items(object) → array[array[any]]
//!
//! items({a: 1, b: 2})     → [["a", 1], ["b", 2]]
//! items({})               → []
//! items({x: "hello"})     → [["x", "hello"]]
//! ```
//!
//! ## from_items
//!
//! Converts an array of key-value pairs (2-element arrays) back to an object.
//! Implements [JEP-013](https://github.com/jmespath-community/jmespath.spec/discussions/47).
//!
//! This function is the inverse of [`items`](#items). When duplicate keys exist,
//! the last occurrence takes precedence.
//!
//! ```text
//! from_items(array[array[any]]) → object
//!
//! from_items([['a', 1], ['b', 2]])   → {a: 1, b: 2}
//! from_items([])                      → {}
//! from_items([['x', 1], ['x', 2]])   → {x: 2}  // Last value wins
//! ```
//!
//! ## pick
//!
//! Returns a new object with only the specified keys.
//!
//! ```text
//! pick(object, keys) → object
//!
//! pick({a: 1, b: 2, c: 3}, ['a', 'c'])   → {a: 1, c: 3}
//! pick({a: 1, b: 2}, ['x'])              → {}
//! pick({a: 1, b: 2}, [])                 → {}
//! ```
//!
//! ## omit
//!
//! Returns a new object without the specified keys.
//!
//! ```text
//! omit(object, keys) → object
//!
//! omit({a: 1, b: 2, c: 3}, ['b'])        → {a: 1, c: 3}
//! omit({a: 1, b: 2}, ['a', 'b'])         → {}
//! omit({a: 1, b: 2}, [])                 → {a: 1, b: 2}
//! ```
//!
//! ## invert
//!
//! Swaps keys and values in an object. Values must be strings or numbers.
//!
//! ```text
//! invert(object) → object
//!
//! invert({a: '1', b: '2'})       → {"1": "a", "2": "b"}
//! invert({x: 'foo', y: 'bar'})   → {foo: "x", bar: "y"}
//! ```
//!
//! ## rename_keys
//!
//! Renames object keys according to a mapping.
//!
//! ```text
//! rename_keys(object, mapping) → object
//!
//! rename_keys({a: 1, b: 2}, {a: 'x', b: 'y'})   → {x: 1, y: 2}
//! rename_keys({a: 1, b: 2}, {a: 'x'})           → {x: 1, b: 2}
//! ```
//!
//! ## flatten_keys
//!
//! Flattens a nested object into a single-level object with compound keys.
//!
//! ```text
//! flatten_keys(object, separator?) → object
//!
//! flatten_keys({a: {b: 1, c: 2}})           → {"a.b": 1, "a.c": 2}
//! flatten_keys({a: {b: {c: 1}}})            → {"a.b.c": 1}
//! flatten_keys({a: {b: 1}}, '_')            → {"a_b": 1}
//! ```
//!
//! ## unflatten_keys
//!
//! Unflattens a flat object with compound keys into a nested object.
//!
//! ```text
//! unflatten_keys(object, separator?) → object
//!
//! unflatten_keys({"a.b": 1, "a.c": 2})      → {a: {b: 1, c: 2}}
//! unflatten_keys({"a.b.c": 1})              → {a: {b: {c: 1}}}
//! unflatten_keys({"a_b": 1}, '_')           → {a: {b: 1}}
//! ```
//!
//! ## deep_merge
//!
//! Recursively merges two objects. Values from the second object override the first.
//!
//! ```text
//! deep_merge(object1, object2) → object
//!
//! deep_merge({a: 1}, {b: 2})                     → {a: 1, b: 2}
//! deep_merge({a: 1}, {a: 2})                     → {a: 2}
//! deep_merge({a: {b: 1}}, {a: {c: 2}})           → {a: {b: 1, c: 2}}
//! deep_merge({a: {b: 1}}, {a: {b: 2}})           → {a: {b: 2}}
//! ```
//!
//! ## deep_equals
//!
//! Deep equality check for any two values (objects, arrays, primitives).
//!
//! ```text
//! deep_equals(a, b) → boolean
//!
//! deep_equals({a: {b: 1}}, {a: {b: 1}})   → true
//! deep_equals([1, [2, 3]], [1, [2, 3]])   → true
//! deep_equals({a: 1}, {a: 2})             → false
//! deep_equals([1, 2], [2, 1])             → false  // Order matters for arrays
//! ```
//!
//! ## deep_diff
//!
//! Returns a structural diff between two objects, showing added, removed, and changed keys.
//!
//! ```text
//! deep_diff(a, b) → object
//!
//! deep_diff({a: 1, b: 2}, {a: 1, b: 3, c: 4})
//! → {
//!     added: {c: 4},
//!     removed: {},
//!     changed: {b: {from: 2, to: 3}}
//!   }
//!
//! deep_diff({x: {y: 1}}, {x: {y: 2, z: 3}})
//! → {
//!     added: {},
//!     removed: {},
//!     changed: {x: {added: {z: 3}, removed: {}, changed: {y: {from: 1, to: 2}}}}
//!   }
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
    runtime.register_function("unflatten_keys", Box::new(UnflattenKeysFn::new()));
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
}
