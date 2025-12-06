//! Expression-based functions for JMESPath.
//!
//! This module provides higher-order functions that accept JMESPath expressions
//! as arguments, enabling powerful data transformations.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `map_expr(expr, array)` | Apply expression to each element |
//! | `filter_expr(expr, array)` | Keep elements where expression is truthy |
//! | `any_expr(expr, array)` | True if any element matches expression |
//! | `all_expr(expr, array)` | True if all elements match expression |
//! | `find_expr(expr, array)` | First element matching expression |
//! | `find_index_expr(expr, array)` | Index of first match or -1 |
//! | `count_expr(expr, array)` | Count elements where expression is truthy |
//! | `sort_by_expr(expr, array)` | Sort array by expression result |
//! | `group_by_expr(expr, array)` | Group elements by expression result |
//! | `partition_expr(expr, array)` | Split into [matches, non_matches] |
//! | `min_by_expr(expr, array)` | Element with minimum expression value |
//! | `max_by_expr(expr, array)` | Element with maximum expression value |
//! | `unique_by_expr(expr, array)` | Dedupe by expression result |
//! | `flat_map_expr(expr, array)` | Map and flatten results |
//!
//! # Examples
//!
//! ```
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::expression;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! expression::register(&mut runtime);
//!
//! // Map: extract field from each object
//! let data = Variable::from_json(r#"[{"name": "Alice"}, {"name": "Bob"}]"#).unwrap();
//! let expr = runtime.compile("map_expr('name', @)").unwrap();
//! let result = expr.search(&data).unwrap();
//! // Result: ["Alice", "Bob"]
//!
//! // Filter: keep objects matching condition
//! let data = Variable::from_json(r#"[{"age": 25}, {"age": 17}, {"age": 30}]"#).unwrap();
//! let expr = runtime.compile("filter_expr('age >= `18`', @)").unwrap();
//! let result = expr.search(&data).unwrap();
//! // Result: [{"age": 25}, {"age": 30}]
//! ```

use std::rc::Rc;

use crate::common::Function;
use crate::{
    ArgumentType, Context, ErrorReason, JmespathError, Rcvar, Runtime, Signature, Variable,
};

/// Register all expression functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("map_expr", Box::new(MapExprFn::new()));
    runtime.register_function("filter_expr", Box::new(FilterExprFn::new()));
    runtime.register_function("any_expr", Box::new(AnyExprFn::new()));
    runtime.register_function("all_expr", Box::new(AllExprFn::new()));
    runtime.register_function("find_expr", Box::new(FindExprFn::new()));
    runtime.register_function("find_index_expr", Box::new(FindIndexExprFn::new()));
    runtime.register_function("count_expr", Box::new(CountExprFn::new()));
    runtime.register_function("sort_by_expr", Box::new(SortByExprFn::new()));
    runtime.register_function("group_by_expr", Box::new(GroupByExprFn::new()));
    runtime.register_function("partition_expr", Box::new(PartitionExprFn::new()));
    runtime.register_function("min_by_expr", Box::new(MinByExprFn::new()));
    runtime.register_function("max_by_expr", Box::new(MaxByExprFn::new()));
    runtime.register_function("unique_by_expr", Box::new(UniqueByExprFn::new()));
    runtime.register_function("flat_map_expr", Box::new(FlatMapExprFn::new()));
}

// =============================================================================
// map_expr(expr, array) -> array
// =============================================================================

pub struct MapExprFn {
    signature: Signature,
}

impl Default for MapExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MapExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for MapExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in map_expr: {}", e)),
            )
        })?;

        let results: Result<Vec<Rcvar>, _> = arr
            .iter()
            .map(|item| compiled.search(item.clone()))
            .collect();

        Ok(Rc::new(Variable::Array(results?)))
    }
}

// =============================================================================
// filter_expr(expr, array) -> array
// =============================================================================

pub struct FilterExprFn {
    signature: Signature,
}

impl Default for FilterExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for FilterExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in filter_expr: {}", e)),
            )
        })?;

        let mut results = Vec::new();
        for item in arr {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                results.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// any_expr(expr, array) -> bool
// =============================================================================

pub struct AnyExprFn {
    signature: Signature,
}

impl Default for AnyExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl AnyExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for AnyExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in any_expr: {}", e)),
            )
        })?;

        for item in arr {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                return Ok(Rc::new(Variable::Bool(true)));
            }
        }

        Ok(Rc::new(Variable::Bool(false)))
    }
}

// =============================================================================
// all_expr(expr, array) -> bool
// =============================================================================

pub struct AllExprFn {
    signature: Signature,
}

impl Default for AllExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl AllExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for AllExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        // Empty array returns true (vacuous truth)
        if arr.is_empty() {
            return Ok(Rc::new(Variable::Bool(true)));
        }

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in all_expr: {}", e)),
            )
        })?;

        for item in arr {
            let result = compiled.search(item.clone())?;
            if !is_truthy(&result) {
                return Ok(Rc::new(Variable::Bool(false)));
            }
        }

        Ok(Rc::new(Variable::Bool(true)))
    }
}

// =============================================================================
// find_expr(expr, array) -> element | null
// =============================================================================

pub struct FindExprFn {
    signature: Signature,
}

impl Default for FindExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl FindExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for FindExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in find_expr: {}", e)),
            )
        })?;

        for item in arr {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                return Ok(item.clone());
            }
        }

        Ok(Rc::new(Variable::Null))
    }
}

// =============================================================================
// find_index_expr(expr, array) -> number | null
// =============================================================================

pub struct FindIndexExprFn {
    signature: Signature,
}

impl Default for FindIndexExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl FindIndexExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for FindIndexExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in find_index_expr: {}", e)),
            )
        })?;

        for (i, item) in arr.iter().enumerate() {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                return Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(i as f64).unwrap(),
                )));
            }
        }

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(-1.0).unwrap(),
        )))
    }
}

// =============================================================================
// count_expr(expr, array) -> number
// =============================================================================

pub struct CountExprFn {
    signature: Signature,
}

impl Default for CountExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CountExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for CountExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in count_expr: {}", e)),
            )
        })?;

        let mut count = 0;
        for item in arr {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                count += 1;
            }
        }

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(count as f64).unwrap(),
        )))
    }
}

// =============================================================================
// sort_by_expr(expr, array) -> array
// =============================================================================

pub struct SortByExprFn {
    signature: Signature,
}

impl Default for SortByExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl SortByExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for SortByExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in sort_by_expr: {}", e)),
            )
        })?;

        // Compute sort keys for each element
        let mut keyed: Vec<(Rcvar, Rcvar)> = Vec::with_capacity(arr.len());
        for item in arr {
            let key = compiled.search(item.clone())?;
            keyed.push((item.clone(), key));
        }

        // Sort by key
        keyed.sort_by(|a, b| compare_values(&a.1, &b.1));

        let results: Vec<Rcvar> = keyed.into_iter().map(|(item, _)| item).collect();
        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// group_by_expr(expr, array) -> object
// =============================================================================

pub struct GroupByExprFn {
    signature: Signature,
}

impl Default for GroupByExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupByExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for GroupByExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in group_by_expr: {}", e)),
            )
        })?;

        let mut groups: std::collections::BTreeMap<String, Vec<Rcvar>> =
            std::collections::BTreeMap::new();

        for item in arr {
            let key_val = compiled.search(item.clone())?;
            let key = value_to_string(&key_val);
            groups.entry(key).or_default().push(item.clone());
        }

        let result: serde_json::Map<String, serde_json::Value> = groups
            .into_iter()
            .map(|(k, v)| {
                let arr: Vec<serde_json::Value> =
                    v.into_iter().map(|item| variable_to_json(&item)).collect();
                (k, serde_json::Value::Array(arr))
            })
            .collect();

        Ok(Rc::new(
            Variable::from_json(&serde_json::to_string(&result).unwrap()).unwrap(),
        ))
    }
}

// =============================================================================
// partition_expr(expr, array) -> [matches, non_matches]
// =============================================================================

pub struct PartitionExprFn {
    signature: Signature,
}

impl Default for PartitionExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl PartitionExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for PartitionExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in partition_expr: {}", e)),
            )
        })?;

        let mut matches = Vec::new();
        let mut non_matches = Vec::new();

        for item in arr {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                matches.push(item.clone());
            } else {
                non_matches.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(vec![
            Rc::new(Variable::Array(matches)),
            Rc::new(Variable::Array(non_matches)),
        ])))
    }
}

// =============================================================================
// min_by_expr(expr, array) -> element | null
// =============================================================================

pub struct MinByExprFn {
    signature: Signature,
}

impl Default for MinByExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MinByExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for MinByExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in min_by_expr: {}", e)),
            )
        })?;

        let mut min_item = arr[0].clone();
        let mut min_key = compiled.search(arr[0].clone())?;

        for item in arr.iter().skip(1) {
            let key = compiled.search(item.clone())?;
            if compare_values(&key, &min_key) == std::cmp::Ordering::Less {
                min_item = item.clone();
                min_key = key;
            }
        }

        Ok(min_item)
    }
}

// =============================================================================
// max_by_expr(expr, array) -> element | null
// =============================================================================

pub struct MaxByExprFn {
    signature: Signature,
}

impl Default for MaxByExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MaxByExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for MaxByExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in max_by_expr: {}", e)),
            )
        })?;

        let mut max_item = arr[0].clone();
        let mut max_key = compiled.search(arr[0].clone())?;

        for item in arr.iter().skip(1) {
            let key = compiled.search(item.clone())?;
            if compare_values(&key, &max_key) == std::cmp::Ordering::Greater {
                max_item = item.clone();
                max_key = key;
            }
        }

        Ok(max_item)
    }
}

// =============================================================================
// unique_by_expr(expr, array) -> array
// =============================================================================

pub struct UniqueByExprFn {
    signature: Signature,
}

impl Default for UniqueByExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl UniqueByExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for UniqueByExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in unique_by_expr: {}", e)),
            )
        })?;

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut results = Vec::new();

        for item in arr {
            let key_val = compiled.search(item.clone())?;
            let key = value_to_string(&key_val);
            if seen.insert(key) {
                results.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// flat_map_expr(expr, array) -> array
// =============================================================================

pub struct FlatMapExprFn {
    signature: Signature,
}

impl Default for FlatMapExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl FlatMapExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for FlatMapExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in flat_map_expr: {}", e)),
            )
        })?;

        let mut results = Vec::new();
        for item in arr {
            let result = compiled.search(item.clone())?;
            match result.as_ref() {
                Variable::Array(inner) => {
                    results.extend(inner.iter().cloned());
                }
                Variable::Null => {
                    // Skip nulls
                }
                _ => {
                    results.push(result);
                }
            }
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Convert a Variable to a string key for grouping/deduplication
fn value_to_string(value: &Rcvar) -> String {
    match value.as_ref() {
        Variable::String(s) => s.clone(),
        Variable::Number(n) => n.to_string(),
        Variable::Bool(b) => b.to_string(),
        Variable::Null => "null".to_string(),
        _ => serde_json::to_string(&variable_to_json(value)).unwrap_or_default(),
    }
}

/// Convert a Variable to a serde_json::Value
fn variable_to_json(value: &Rcvar) -> serde_json::Value {
    match value.as_ref() {
        Variable::String(s) => serde_json::Value::String(s.clone()),
        Variable::Number(n) => serde_json::Value::Number(n.clone()),
        Variable::Bool(b) => serde_json::Value::Bool(*b),
        Variable::Null => serde_json::Value::Null,
        Variable::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(variable_to_json).collect())
        }
        Variable::Object(obj) => {
            let map: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), variable_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Variable::Expref(_) => serde_json::Value::Null,
    }
}

/// Check if a value is truthy (JMESPath semantics)
fn is_truthy(value: &Rcvar) -> bool {
    match value.as_ref() {
        Variable::Null => false,
        Variable::Bool(b) => *b,
        Variable::String(s) => !s.is_empty(),
        Variable::Array(a) => !a.is_empty(),
        Variable::Object(o) => !o.is_empty(),
        Variable::Number(_) => true,
        Variable::Expref(_) => true,
    }
}

/// Compare two values for sorting
fn compare_values(a: &Rcvar, b: &Rcvar) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a.as_ref(), b.as_ref()) {
        (Variable::Number(an), Variable::Number(bn)) => {
            let a_f = an.as_f64().unwrap_or(0.0);
            let b_f = bn.as_f64().unwrap_or(0.0);
            a_f.partial_cmp(&b_f).unwrap_or(Ordering::Equal)
        }
        (Variable::String(as_), Variable::String(bs)) => as_.cmp(bs),
        (Variable::Null, Variable::Null) => Ordering::Equal,
        (Variable::Null, _) => Ordering::Less,
        (_, Variable::Null) => Ordering::Greater,
        _ => Ordering::Equal,
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
    fn test_map_expr_field() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"name": "Alice"}, {"name": "Bob"}]"#).unwrap();
        let expr = runtime.compile("map_expr('name', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "Alice");
        assert_eq!(arr[1].as_string().unwrap(), "Bob");
    }

    #[test]
    fn test_map_expr_transform() {
        let runtime = setup();
        let data = Variable::from_json(r#"["hello", "world"]"#).unwrap();
        let expr = runtime.compile("map_expr('length(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_number().unwrap(), 5.0);
        assert_eq!(arr[1].as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_filter_expr() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"age": 25}, {"age": 17}, {"age": 30}]"#).unwrap();
        let expr = runtime.compile("filter_expr('age >= `18`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_filter_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("filter_expr('@ > `10`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_any_expr_true() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": false}, {"active": true}]"#).unwrap();
        let expr = runtime.compile("any_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_any_expr_false() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": false}, {"active": false}]"#).unwrap();
        let expr = runtime.compile("any_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_all_expr_true() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": true}, {"active": true}]"#).unwrap();
        let expr = runtime.compile("all_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_all_expr_false() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": true}, {"active": false}]"#).unwrap();
        let expr = runtime.compile("all_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_all_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("all_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true); // vacuous truth
    }

    #[test]
    fn test_find_expr_found() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#)
            .unwrap();
        let expr = runtime.compile("find_expr('id == `2`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "Bob");
    }

    #[test]
    fn test_find_expr_not_found() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        let expr = runtime.compile("find_expr('id == `99`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_sort_by_expr_numbers() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"val": 3}, {"val": 1}, {"val": 2}]"#).unwrap();
        let expr = runtime.compile("sort_by_expr('val', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_number()
                .unwrap(),
            1.0
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_number()
                .unwrap(),
            2.0
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_number()
                .unwrap(),
            3.0
        );
    }

    #[test]
    fn test_sort_by_expr_strings() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"name": "Charlie"}, {"name": "Alice"}, {"name": "Bob"}]"#)
                .unwrap();
        let expr = runtime.compile("sort_by_expr('name', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Alice"
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Bob"
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Charlie"
        );
    }

    #[test]
    fn test_find_index_expr_found() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"id": 1}, {"id": 2}, {"id": 3}]"#).unwrap();
        let expr = runtime.compile("find_index_expr('id == `2`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_find_index_expr_not_found() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        let expr = runtime.compile("find_index_expr('id == `99`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), -1.0);
    }

    #[test]
    fn test_count_expr() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"active": true}, {"active": false}, {"active": true}]"#)
                .unwrap();
        let expr = runtime.compile("count_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_count_expr_none() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("count_expr('@ > `10`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_group_by_expr() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"type": "a", "val": 1}, {"type": "b", "val": 2}, {"type": "a", "val": 3}]"#,
        )
        .unwrap();
        let expr = runtime.compile("group_by_expr('type', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("b").unwrap().as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_partition_expr() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("partition_expr('@ > `3`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let matches = arr[0].as_array().unwrap();
        let non_matches = arr[1].as_array().unwrap();
        assert_eq!(matches.len(), 2); // 4, 5
        assert_eq!(non_matches.len(), 3); // 1, 2, 3
    }

    #[test]
    fn test_min_by_expr() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]"#)
                .unwrap();
        let expr = runtime.compile("min_by_expr('age', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "Bob");
    }

    #[test]
    fn test_min_by_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("min_by_expr('age', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_max_by_expr() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]"#)
                .unwrap();
        let expr = runtime.compile("max_by_expr('age', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "Alice");
    }

    #[test]
    fn test_unique_by_expr() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"type": "a", "val": 1}, {"type": "b", "val": 2}, {"type": "a", "val": 3}]"#,
        )
        .unwrap();
        let expr = runtime.compile("unique_by_expr('type', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // First "a" and first "b"
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("val")
                .unwrap()
                .as_number()
                .unwrap(),
            1.0
        );
    }

    #[test]
    fn test_flat_map_expr() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"tags": ["a", "b"]}, {"tags": ["c"]}, {"tags": ["d", "e"]}]"#)
                .unwrap();
        let expr = runtime.compile("flat_map_expr('tags', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_string().unwrap(), "a");
        assert_eq!(arr[4].as_string().unwrap(), "e");
    }

    #[test]
    fn test_flat_map_expr_non_array() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"name": "Alice"}, {"name": "Bob"}]"#).unwrap();
        let expr = runtime.compile("flat_map_expr('name', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "Alice");
    }
}
