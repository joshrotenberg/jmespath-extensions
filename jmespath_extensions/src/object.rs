//! Object manipulation functions.
//!
//! This module provides extended object operations beyond the standard JMESPath built-ins.
//!
//! # Function Reference
//!
//! | Function | Signature | Description |
//! |----------|-----------|-------------|
//! | [`items`](#items) | `items(object) → array` | Convert to [{key, value}, ...] |
//! | [`from_items`](#from_items) | `from_items(array) → object` | Convert [{key, value}, ...] to object |
//! | [`pick`](#pick) | `pick(object, keys) → object` | Select specific keys |
//! | [`omit`](#omit) | `omit(object, keys) → object` | Remove specific keys |
//! | [`invert`](#invert) | `invert(object) → object` | Swap keys and values |
//! | [`rename_keys`](#rename_keys) | `rename_keys(object, mapping) → object` | Rename keys |
//! | [`flatten_keys`](#flatten_keys) | `flatten_keys(object, sep?) → object` | Flatten nested object |
//! | [`unflatten_keys`](#unflatten_keys) | `unflatten_keys(object, sep?) → object` | Unflatten to nested |
//! | [`deep_merge`](#deep_merge) | `deep_merge(obj1, obj2) → object` | Deep merge two objects |
//! | [`deep_equals`](#deep_equals) | `deep_equals(a, b) → boolean` | Deep equality check |
//! | [`deep_diff`](#deep_diff) | `deep_diff(a, b) → object` | Structural diff between values |
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
//! Converts an object to an array of `{key, value}` objects.
//!
//! ```text
//! items(object) → array
//!
//! items({a: 1, b: 2})     → [{key: "a", value: 1}, {key: "b", value: 2}]
//! items({})               → []
//! ```
//!
//! ## from_items
//!
//! Converts an array of `{key, value}` objects back to an object.
//!
//! ```text
//! from_items(array) → object
//!
//! from_items([{key: 'a', value: 1}, {key: 'b', value: 2}])   → {a: 1, b: 2}
//! from_items([])                                              → {}
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
}

// =============================================================================
// entries(object) -> array of {key, value} objects
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

        let entries: Vec<Rcvar> = obj
            .iter()
            .map(|(k, v)| {
                let mut entry = BTreeMap::new();
                entry.insert("key".to_string(), Rc::new(Variable::String(k.clone())));
                entry.insert("value".to_string(), v.clone());
                Rc::new(Variable::Object(entry)) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(entries)))
    }
}

// =============================================================================
// from_entries(array) -> object from array of {key, value}
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
            if let Some(obj) = item.as_object() {
                if let (Some(key), Some(value)) = (obj.get("key"), obj.get("value")) {
                    if let Some(key_str) = key.as_string() {
                        result.insert(key_str.to_string(), value.clone());
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
        let mut obj = BTreeMap::new();
        obj.insert(
            "a".to_string(),
            Rc::new(Variable::Number(serde_json::Number::from(1))),
        );
        let data = Variable::Object(obj);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
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
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_deep_equals_objects_different() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": {"b": 1}, "c": {"b": 2}}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_deep_equals_arrays() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, [2, 3]], "b": [1, [2, 3]]}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_deep_equals_arrays_order_matters() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2], "b": [2, 1]}"#).unwrap();
        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_deep_equals_primitives() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": "hello", "b": "hello", "c": "world"}"#).unwrap();

        let expr = runtime.compile("deep_equals(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let expr = runtime.compile("deep_equals(a, c)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
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
}
