//! Expression-based higher-order functions.
//!
//! This module provides expression functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category expression`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::expression;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! expression::register(&mut runtime);
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

    // Lodash-style aliases
    runtime.register_function("some", Box::new(AnyExprFn::new()));
    runtime.register_function("every", Box::new(AllExprFn::new()));
    runtime.register_function("reject", Box::new(RejectFn::new()));
    runtime.register_function("map_keys", Box::new(MapKeysFn::new()));
    runtime.register_function("map_values", Box::new(MapValuesFn::new()));
    runtime.register_function("order_by", Box::new(OrderByFn::new()));
    runtime.register_function("reduce_expr", Box::new(ReduceExprFn::new()));
    runtime.register_function("scan_expr", Box::new(ScanExprFn::new()));
    // Alias for reduce_expr (lodash-style)
    runtime.register_function("fold", Box::new(ReduceExprFn::new()));
    runtime.register_function("count_by", Box::new(CountByFn::new()));

    // Partial application functions
    runtime.register_function("partial", Box::new(PartialFn::new()));
    runtime.register_function("apply", Box::new(ApplyFn::new()));

    // Functional array operations
    runtime.register_function("take_while", Box::new(TakeWhileFn::new()));
    runtime.register_function("drop_while", Box::new(DropWhileFn::new()));
    runtime.register_function("zip_with", Box::new(ZipWithFn::new()));

    // Recursive transformation
    runtime.register_function("walk", Box::new(WalkFn::new()));
}

// =============================================================================
// map_expr(expr, array) -> array
// =============================================================================

/// Apply a JMESPath expression to each element of an array.
///
/// # Arguments
/// * `expr` - A JMESPath expression string to evaluate against each element
/// * `array` - The array to map over
///
/// # Returns
/// A new array containing the result of applying the expression to each element.
///
/// # Example
/// ```text
/// map_expr('name', [{"name": "Alice"}, {"name": "Bob"}]) -> ["Alice", "Bob"]
/// map_expr('@ * `2`', [1, 2, 3]) -> [2, 4, 6]
/// ```
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

/// Filter an array, keeping elements where the expression evaluates to a truthy value.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to filter
///
/// # Returns
/// A new array containing only elements where the expression was truthy.
///
/// # Example
/// ```text
/// filter_expr('age >= `18`', [{"age": 25}, {"age": 17}]) -> [{"age": 25}]
/// filter_expr('@ > `2`', [1, 2, 3, 4]) -> [3, 4]
/// ```
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

/// Check if any element in the array matches the expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to check
///
/// # Returns
/// `true` if at least one element produces a truthy result, `false` otherwise.
///
/// # Example
/// ```text
/// any_expr('@ > `5`', [1, 2, 10]) -> true
/// any_expr('@ > `5`', [1, 2, 3]) -> false
/// ```
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

/// Check if all elements in the array match the expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to check
///
/// # Returns
/// `true` if every element produces a truthy result, `false` otherwise.
/// Returns `true` for empty arrays.
///
/// # Example
/// ```text
/// all_expr('@ > `0`', [1, 2, 3]) -> true
/// all_expr('@ > `0`', [1, -1, 3]) -> false
/// ```
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

/// Find the first element in the array that matches the expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to search
///
/// # Returns
/// The first element where the expression is truthy, or `null` if none match.
///
/// # Example
/// ```text
/// find_expr('@ > `5`', [1, 3, 7, 9]) -> 7
/// find_expr('@ > `10`', [1, 3, 7, 9]) -> null
/// ```
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

/// Find the index of the first element that matches the expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to search
///
/// # Returns
/// The zero-based index of the first matching element, or `-1` if none match.
///
/// # Example
/// ```text
/// find_index_expr('@ > `5`', [1, 3, 7, 9]) -> 2
/// find_index_expr('@ > `10`', [1, 3, 7, 9]) -> -1
/// ```
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

/// Count elements in the array where the expression is truthy.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to count matches in
///
/// # Returns
/// The number of elements where the expression evaluated to a truthy value.
///
/// # Example
/// ```text
/// count_expr('@ > `2`', [1, 2, 3, 4, 5]) -> 3
/// count_expr('active', [{"active": true}, {"active": false}]) -> 1
/// ```
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

/// Sort an array by the result of applying an expression to each element.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that extracts a sort key from each element
/// * `array` - The array to sort
///
/// # Returns
/// A new array sorted by the expression result in ascending order.
///
/// # Example
/// ```text
/// sort_by_expr('age', [{"age": 30}, {"age": 20}]) -> [{"age": 20}, {"age": 30}]
/// sort_by_expr('name', [{"name": "Bob"}, {"name": "Alice"}]) -> [{"name": "Alice"}, {"name": "Bob"}]
/// ```
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

/// Group array elements by the result of applying an expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that extracts a grouping key from each element
/// * `array` - The array to group
///
/// # Returns
/// An object where keys are the stringified expression results and values are arrays of matching elements.
///
/// # Example
/// ```text
/// group_by_expr('type', [{"type": "a", "v": 1}, {"type": "b", "v": 2}, {"type": "a", "v": 3}])
///   -> {"a": [{"type": "a", "v": 1}, {"type": "a", "v": 3}], "b": [{"type": "b", "v": 2}]}
/// ```
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
// count_by(expr, array) -> object (count occurrences by expression result)
// =============================================================================

/// Count occurrences of elements grouped by an expression result.
///
/// Similar to `frequencies` but allows extracting a key via expression.
/// Similar to `group_by_expr` but returns counts instead of grouped elements.
///
/// # Arguments
/// * `expr` - A JMESPath expression string to extract the grouping key
/// * `array` - The array to count
///
/// # Returns
/// An object mapping each unique key to its count.
///
/// # Example
/// ```text
/// count_by('type', [{"type": "a"}, {"type": "b"}, {"type": "a"}])
///   -> {"a": 2, "b": 1}
/// count_by('@', ['a', 'b', 'a', 'c', 'a']) -> {"a": 3, "b": 1, "c": 1}
/// ```
pub struct CountByFn {
    signature: Signature,
}

impl Default for CountByFn {
    fn default() -> Self {
        Self::new()
    }
}

impl CountByFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for CountByFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in count_by: {}", e)),
            )
        })?;

        let mut counts: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();

        for item in arr {
            let key_val = compiled.search(item.clone())?;
            let key = value_to_string(&key_val);
            *counts.entry(key).or_insert(0) += 1;
        }

        let result: serde_json::Map<String, serde_json::Value> = counts
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(serde_json::Number::from(v))))
            .collect();

        Ok(Rc::new(
            Variable::from_json(&serde_json::to_string(&result).unwrap()).unwrap(),
        ))
    }
}

// =============================================================================
// partition_expr(expr, array) -> [matches, non_matches]
// =============================================================================

/// Partition an array into two arrays based on an expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to partition
///
/// # Returns
/// A two-element array: `[matches, non_matches]` where `matches` contains elements
/// where the expression was truthy, and `non_matches` contains the rest.
///
/// # Example
/// ```text
/// partition_expr('@ > `2`', [1, 2, 3, 4]) -> [[3, 4], [1, 2]]
/// partition_expr('active', [{active: true}, {active: false}]) -> [[{active: true}], [{active: false}]]
/// ```
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

/// Find the element with the minimum value when applying an expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that extracts a comparable value from each element
/// * `array` - The array to search
///
/// # Returns
/// The element with the smallest expression result, or `null` for empty arrays.
///
/// # Example
/// ```text
/// min_by_expr('age', [{"age": 30}, {"age": 20}, {"age": 25}]) -> {"age": 20}
/// min_by_expr('@', [5, 2, 8, 1]) -> 1
/// ```
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

/// Find the element with the maximum value when applying an expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that extracts a comparable value from each element
/// * `array` - The array to search
///
/// # Returns
/// The element with the largest expression result, or `null` for empty arrays.
///
/// # Example
/// ```text
/// max_by_expr('age', [{"age": 30}, {"age": 20}, {"age": 25}]) -> {"age": 30}
/// max_by_expr('@', [5, 2, 8, 1]) -> 8
/// ```
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

/// Remove duplicate elements based on the result of an expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that extracts a uniqueness key from each element
/// * `array` - The array to deduplicate
///
/// # Returns
/// A new array with duplicates removed, keeping the first occurrence of each unique key.
///
/// # Example
/// ```text
/// unique_by_expr('id', [{"id": 1, "v": "a"}, {"id": 2, "v": "b"}, {"id": 1, "v": "c"}])
///   -> [{"id": 1, "v": "a"}, {"id": 2, "v": "b"}]
/// ```
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

/// Apply an expression to each element and flatten the results.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns an array for each element
/// * `array` - The array to flat-map over
///
/// # Returns
/// A single array containing all elements from the results concatenated together.
///
/// # Example
/// ```text
/// flat_map_expr('tags', [{"tags": ["a", "b"]}, {"tags": ["c"]}]) -> ["a", "b", "c"]
/// flat_map_expr('@', [[1, 2], [3, 4]]) -> [1, 2, 3, 4]
/// ```
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

/// Convert a Variable to a serde_json::Value for JSON serialization.
///
/// Handles all Variable types including nested arrays and objects.
/// Expression references are converted to null.
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

/// Check if a value is truthy according to JMESPath semantics.
///
/// JMESPath truthiness rules:
/// - `null` is falsy
/// - `false` is falsy
/// - Empty string `""` is falsy
/// - Empty array `[]` is falsy
/// - Empty object `{}` is falsy
/// - All other values (numbers, non-empty strings/arrays/objects, true) are truthy
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

/// Compare two values for sorting purposes.
///
/// Comparison rules:
/// - Numbers are compared numerically
/// - Strings are compared lexicographically
/// - `null` sorts before all other values
/// - Mixed types compare as equal (stable sort preserves original order)
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

// =============================================================================
// reject(expr, array) -> array (inverse of filter_expr)
// =============================================================================

/// Filter an array, keeping elements where the expression is falsy (inverse of filter_expr).
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to filter
///
/// # Returns
/// A new array containing only elements where the expression was falsy.
///
/// # Example
/// ```text
/// reject('@ > `2`', [1, 2, 3, 4]) -> [1, 2]
/// reject('active', [{"active": true}, {"active": false}]) -> [{"active": false}]
/// ```
pub struct RejectFn {
    signature: Signature,
}

impl Default for RejectFn {
    fn default() -> Self {
        Self::new()
    }
}

impl RejectFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for RejectFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e.to_string()))
        })?;

        let mut result = Vec::new();
        for item in arr {
            let matched = compiled.search(item).map_err(|e| {
                JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e.to_string()))
            })?;
            // Keep items where expression is falsy (inverse of filter)
            if !is_truthy(&matched) {
                result.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// map_keys(expr, object) -> object
// =============================================================================

use std::collections::BTreeMap;

/// Transform the keys of an object by applying an expression to each key.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that transforms each key (key is passed as `@`)
/// * `object` - The object whose keys to transform
///
/// # Returns
/// A new object with transformed keys and original values.
///
/// # Example
/// ```text
/// map_keys('upper(@)', {"a": 1, "b": 2}) -> {"A": 1, "B": 2}
/// map_keys('@ & "_suffix"', {"foo": 1}) -> {"foo_suffix": 1}
/// ```
pub struct MapKeysFn {
    signature: Signature,
}

impl Default for MapKeysFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MapKeysFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Object], None),
        }
    }
}

impl Function for MapKeysFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let obj = args[1].as_object().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e.to_string()))
        })?;

        let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();
        for (key, value) in obj.iter() {
            // Apply expression to the key
            let key_var = Rc::new(Variable::String(key.clone()));
            let new_key = compiled.search(&key_var).map_err(|e| {
                JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e.to_string()))
            })?;

            let new_key_str = match &*new_key {
                Variable::String(s) => s.clone(),
                Variable::Number(n) => n.to_string(),
                _ => key.clone(), // Keep original if result isn't a string/number
            };

            result.insert(new_key_str, value.clone());
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// map_values(expr, object) -> object
// =============================================================================

/// Transform the values of an object by applying an expression to each value.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that transforms each value (value is passed as `@`)
/// * `object` - The object whose values to transform
///
/// # Returns
/// A new object with original keys and transformed values.
///
/// # Example
/// ```text
/// map_values('@ * `2`', {"a": 1, "b": 2}) -> {"a": 2, "b": 4}
/// map_values('upper(@)', {"x": "hello", "y": "world"}) -> {"x": "HELLO", "y": "WORLD"}
/// ```
pub struct MapValuesFn {
    signature: Signature,
}

impl Default for MapValuesFn {
    fn default() -> Self {
        Self::new()
    }
}

impl MapValuesFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Object], None),
        }
    }
}

impl Function for MapValuesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let obj = args[1].as_object().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e.to_string()))
        })?;

        let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();
        for (key, value) in obj.iter() {
            // Apply expression to the value
            let new_value = compiled.search(value).map_err(|e| {
                JmespathError::new(ctx.expression, 0, ErrorReason::Parse(e.to_string()))
            })?;

            result.insert(key.clone(), new_value);
        }

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// order_by(array, criteria) -> array
// =============================================================================

/// Sort an array by multiple criteria with direction control.
///
/// # Arguments
/// * `array` - The array to sort
/// * `criteria` - Array of [field, direction] pairs where direction is "asc" or "desc"
///   Use JMESPath literal syntax with backticks: `` `[["field", "asc"]]` ``
///
/// # Returns
/// A new sorted array.
///
/// # Example
/// ```text
/// order_by(@, `[["name", "asc"]]`)  // Sort by name ascending
/// order_by(@, `[["age", "desc"], ["name", "asc"]]`)  // Sort by age desc, then name asc
/// ```
pub struct OrderByFn {
    signature: Signature,
}

impl Default for OrderByFn {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderByFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::Array, ArgumentType::Array], None),
        }
    }
}

impl Function for OrderByFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();
        let criteria = args[1].as_array().unwrap();

        if arr.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        // Parse criteria: each element should be [field, direction]
        let mut sort_specs: Vec<(String, bool)> = Vec::new(); // (field, ascending)
        for criterion in criteria {
            let crit_arr = criterion.as_array().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse("Each criterion must be an array [field, direction]".into()),
                )
            })?;

            if crit_arr.len() < 2 {
                return Err(JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse("Each criterion must have [field, direction]".into()),
                ));
            }

            let field = crit_arr[0].as_string().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse("Field name must be a string".into()),
                )
            })?;

            let direction = crit_arr[1].as_string().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse("Direction must be 'asc' or 'desc'".into()),
                )
            })?;

            let ascending = match direction.to_lowercase().as_str() {
                "asc" | "ascending" => true,
                "desc" | "descending" => false,
                _ => {
                    return Err(JmespathError::new(
                        ctx.expression,
                        ctx.offset,
                        ErrorReason::Parse("Direction must be 'asc' or 'desc'".into()),
                    ));
                }
            };

            sort_specs.push((field.to_string(), ascending));
        }

        // Clone and sort the array
        let mut result: Vec<Rcvar> = arr.clone();
        result.sort_by(|a, b| {
            for (field, ascending) in &sort_specs {
                let a_val = a
                    .as_object()
                    .and_then(|o| o.get(field))
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Variable::Null));
                let b_val = b
                    .as_object()
                    .and_then(|o| o.get(field))
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Variable::Null));

                let cmp = compare_values(&a_val, &b_val);
                if cmp != std::cmp::Ordering::Equal {
                    return if *ascending { cmp } else { cmp.reverse() };
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// reduce_expr(expr, array, initial) -> any
// =============================================================================

/// Reduce an array to a single value using an expression.
///
/// The expression is evaluated with a special context where:
/// - `accumulator` is the current accumulated value
/// - `current` is the current element being processed
/// - `index` is the current index (0-based)
///
/// # Arguments
/// * `expr` - A JMESPath expression string. Use `accumulator` and `current` in the expression.
/// * `array` - The array to reduce
/// * `initial` - The initial value for the accumulator
///
/// # Returns
/// The final accumulated value.
///
/// # Example
/// ```text
/// reduce_expr('accumulator + current', [1, 2, 3], `0`)  // Sum: 6
/// reduce_expr('max([accumulator, current])', [3, 1, 4], `0`)  // Max: 4
/// ```
pub struct ReduceExprFn {
    signature: Signature,
}

impl Default for ReduceExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ReduceExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![ArgumentType::String, ArgumentType::Array, ArgumentType::Any],
                None,
            ),
        }
    }
}

impl Function for ReduceExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();
        let initial = args[2].clone();

        if arr.is_empty() {
            return Ok(initial);
        }

        // Compile the expression
        let runtime = ctx.runtime;
        let compiled = runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid reduce expression: {}", e)),
            )
        })?;

        let mut accumulator = initial;

        for (idx, item) in arr.iter().enumerate() {
            // Create context object with accumulator, current, and index
            let mut context_map: std::collections::BTreeMap<String, Rcvar> =
                std::collections::BTreeMap::new();
            context_map.insert("accumulator".to_string(), accumulator.clone());
            context_map.insert("current".to_string(), item.clone());
            context_map.insert(
                "index".to_string(),
                Rc::new(Variable::Number(serde_json::Number::from(idx as i64))),
            );
            let context_var = Rc::new(Variable::Object(context_map));

            accumulator = compiled.search(&context_var).map_err(|e| {
                JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse(format!("Reduce expression evaluation error: {}", e)),
                )
            })?;
        }

        Ok(accumulator)
    }
}

// =============================================================================
// scan_expr(expr, array, initial) -> array
// =============================================================================

/// Scan (cumulative reduce) an array, returning all intermediate accumulated values.
///
/// Similar to reduce_expr, but returns an array of all intermediate results.
///
/// # Arguments
/// * `expr` - A JMESPath expression string. Use `accumulator` and `current` in the expression.
/// * `array` - The array to scan
/// * `initial` - The initial value for the accumulator
///
/// # Returns
/// An array of all accumulated values (including each intermediate step).
///
/// # Example
/// ```text
/// scan_expr('accumulator + current', [1, 2, 3], `0`)  // Running sum: [1, 3, 6]
/// ```
pub struct ScanExprFn {
    signature: Signature,
}

impl Default for ScanExprFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanExprFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![ArgumentType::String, ArgumentType::Array, ArgumentType::Any],
                None,
            ),
        }
    }
}

impl Function for ScanExprFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();
        let initial = args[2].clone();

        if arr.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        // Compile the expression
        let runtime = ctx.runtime;
        let compiled = runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid scan expression: {}", e)),
            )
        })?;

        let mut accumulator = initial;
        let mut results: Vec<Rcvar> = Vec::with_capacity(arr.len());

        for (idx, item) in arr.iter().enumerate() {
            // Create context object with accumulator, current, and index
            let mut context_map: std::collections::BTreeMap<String, Rcvar> =
                std::collections::BTreeMap::new();
            context_map.insert("accumulator".to_string(), accumulator.clone());
            context_map.insert("current".to_string(), item.clone());
            context_map.insert(
                "index".to_string(),
                Rc::new(Variable::Number(serde_json::Number::from(idx as i64))),
            );
            let context_var = Rc::new(Variable::Object(context_map));

            accumulator = compiled.search(&context_var).map_err(|e| {
                JmespathError::new(
                    ctx.expression,
                    ctx.offset,
                    ErrorReason::Parse(format!("Scan expression evaluation error: {}", e)),
                )
            })?;

            results.push(accumulator.clone());
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// partial(fn_name, ...args) -> partial object
// =============================================================================

/// Create a partial function with some arguments pre-filled.
///
/// Returns an object that can be used with `apply()` to invoke the function
/// with the remaining arguments. This enables currying and reusable function
/// configurations.
///
/// # Arguments
/// * `fn_name` - The name of the function to partially apply
/// * `...args` - Zero or more arguments to pre-fill
///
/// # Returns
/// A partial object: `{"__partial__": true, "fn": "fn_name", "args": [...]}`
///
/// # Examples
///
/// ## Basic Usage
/// ```text
/// partial('join', `"-"`)  // Create a dash-joiner
/// // -> {"__partial__": true, "fn": "join", "args": ["-"]}
/// ```
///
/// ## Reusable String Operations
/// ```text
/// // Create a comma-joiner for CSV-like output
/// csv_joiner = partial('join', `","`)
/// apply(csv_joiner, `["name", "age", "city"]`)  // -> "name,age,city"
/// ```
///
/// ## Pre-configured Search
/// ```text
/// // Create a contains checker with pre-filled haystack
/// has_hello = partial('contains', `"hello world"`)
/// apply(has_hello, `"world"`)  // -> true
/// apply(has_hello, `"xyz"`)    // -> false
/// ```
///
/// ## Date Formatting
/// ```text
/// // Create a reusable ISO date formatter
/// iso_formatter = partial('format_date', `"%Y-%m-%d"`)
/// apply(iso_formatter, `"2024-01-15T10:30:00Z"`)  // -> "2024-01-15"
/// ```
pub struct PartialFn {
    #[allow(dead_code)]
    signature: Signature,
}

impl Default for PartialFn {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialFn {
    pub fn new() -> Self {
        Self {
            // At least function name required, then variadic args
            signature: Signature::new(vec![ArgumentType::String], Some(ArgumentType::Any)),
        }
    }
}

impl Function for PartialFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        if args.is_empty() {
            return Err(JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse("partial() requires at least a function name".into()),
            ));
        }

        let fn_name = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(
                    "partial() first argument must be a function name string".into(),
                ),
            )
        })?;

        // Collect the pre-filled arguments
        let prefilled_args: Vec<serde_json::Value> =
            args[1..].iter().map(variable_to_json).collect();

        // Create the partial object
        let mut partial_obj = serde_json::Map::new();
        partial_obj.insert("__partial__".to_string(), serde_json::Value::Bool(true));
        partial_obj.insert(
            "fn".to_string(),
            serde_json::Value::String(fn_name.to_string()),
        );
        partial_obj.insert("args".to_string(), serde_json::Value::Array(prefilled_args));

        Ok(Rc::new(
            Variable::from_json(&serde_json::to_string(&partial_obj).unwrap()).unwrap(),
        ))
    }
}

// =============================================================================
// apply(partial_or_fn, ...args) -> result
// =============================================================================

/// Apply a partial function or regular function with arguments.
///
/// If the first argument is a partial object (from `partial()`), combines
/// the pre-filled arguments with the provided arguments and invokes the function.
/// If it's a string, treats it as a function name and invokes directly.
///
/// This function is the complement to `partial()` - use `partial()` to create
/// reusable function configurations, then `apply()` to execute them.
///
/// # Arguments
/// * `partial_or_fn` - Either a partial object or a function name string
/// * `...args` - Additional arguments to pass to the function
///
/// # Returns
/// The result of invoking the function with all arguments.
///
/// # Examples
///
/// ## Apply a Partial
/// ```text
/// // Create and apply a dash-joiner
/// apply(partial('join', `"-"`), `["a", "b", "c"]`)  // -> "a-b-c"
/// ```
///
/// ## Direct Function Call by Name
/// ```text
/// // Call any function by its string name
/// apply('length', `"hello"`)  // -> 5
/// apply('upper', `"hello"`)   // -> "HELLO"
/// ```
///
/// ## Dynamic Function Dispatch
/// ```text
/// // Useful when the function name comes from data or configuration
/// fn_name = 'sum'
/// apply(fn_name, `[1, 2, 3, 4]`)  // -> 10
/// ```
///
/// ## Combining with Partials
/// ```text
/// // Pre-configure a contains check, then apply multiple times
/// checker = partial('contains', `"The quick brown fox"`)
/// apply(checker, `"quick"`)  // -> true
/// apply(checker, `"slow"`)   // -> false
/// ```
///
/// ## Building Pipelines
/// ```text
/// // Create specialized validators
/// email_pattern = partial('regex_match', `"^[a-z]+@[a-z]+\\.[a-z]+$"`)
/// apply(email_pattern, `"test@example.com"`)  // -> true
/// ```
pub struct ApplyFn {
    #[allow(dead_code)]
    signature: Signature,
}

impl Default for ApplyFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplyFn {
    pub fn new() -> Self {
        Self {
            // First arg is partial or fn name, then variadic args
            signature: Signature::new(vec![ArgumentType::Any], Some(ArgumentType::Any)),
        }
    }
}

impl Function for ApplyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        if args.is_empty() {
            return Err(JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse("apply() requires at least one argument".into()),
            ));
        }

        let first_arg = &args[0];
        let additional_args = &args[1..];

        // Check if it's a partial object
        if let Some(obj) = first_arg.as_object() {
            if obj.get("__partial__").map(|v| v.as_boolean()) == Some(Some(true)) {
                // It's a partial - extract fn name and pre-filled args
                let fn_name = obj.get("fn").and_then(|v| v.as_string()).ok_or_else(|| {
                    JmespathError::new(
                        ctx.expression,
                        ctx.offset,
                        ErrorReason::Parse("Invalid partial object: missing 'fn' field".into()),
                    )
                })?;

                let prefilled = obj.get("args").and_then(|v| v.as_array()).ok_or_else(|| {
                    JmespathError::new(
                        ctx.expression,
                        ctx.offset,
                        ErrorReason::Parse("Invalid partial object: missing 'args' field".into()),
                    )
                })?;

                // Build the full expression: fn_name(prefilled_args..., additional_args...)
                return invoke_function(fn_name, prefilled, additional_args, ctx);
            }
        }

        // If it's a string, treat as function name
        if let Some(fn_name) = first_arg.as_string() {
            return invoke_function(fn_name, &[], additional_args, ctx);
        }

        Err(JmespathError::new(
            ctx.expression,
            ctx.offset,
            ErrorReason::Parse(
                "apply() first argument must be a partial object or function name string".into(),
            ),
        ))
    }
}

/// Helper to invoke a function by name with pre-filled and additional arguments
fn invoke_function(
    fn_name: &str,
    prefilled: &[Rcvar],
    additional: &[Rcvar],
    ctx: &mut Context<'_>,
) -> Result<Rcvar, JmespathError> {
    // Build the argument list for the expression
    let mut all_args_json: Vec<String> = Vec::new();

    // Add pre-filled args as literals
    for arg in prefilled {
        let json = variable_to_json(arg);
        all_args_json.push(format!("`{}`", serde_json::to_string(&json).unwrap()));
    }

    // Add additional args as literals
    for arg in additional {
        let json = variable_to_json(arg);
        all_args_json.push(format!("`{}`", serde_json::to_string(&json).unwrap()));
    }

    // Build and execute the expression
    let expr_str = format!("{}({})", fn_name, all_args_json.join(", "));

    let compiled = ctx.runtime.compile(&expr_str).map_err(|e| {
        JmespathError::new(
            ctx.expression,
            ctx.offset,
            ErrorReason::Parse(format!(
                "Failed to compile function call '{}': {}",
                expr_str, e
            )),
        )
    })?;

    // Execute with null input since all args are literals
    compiled.search(Rc::new(Variable::Null)).map_err(|e| {
        JmespathError::new(
            ctx.expression,
            ctx.offset,
            ErrorReason::Parse(format!("Failed to execute '{}': {}", fn_name, e)),
        )
    })
}

// =============================================================================
// take_while(expr, array) -> array
// =============================================================================

/// Take elements from the beginning of an array while the expression is truthy.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to process
///
/// # Returns
/// A new array containing elements from the start until the predicate returns false.
///
/// # Example
/// ```text
/// take_while('@ < `4`', [1, 2, 3, 5, 1, 2]) -> [1, 2, 3]
/// take_while('@ > `0`', [3, 2, 1, 0, -1]) -> [3, 2, 1]
/// ```
pub struct TakeWhileFn {
    signature: Signature,
}

impl Default for TakeWhileFn {
    fn default() -> Self {
        Self::new()
    }
}

impl TakeWhileFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for TakeWhileFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in take_while: {}", e)),
            )
        })?;

        let mut results = Vec::new();
        for item in arr {
            let result = compiled.search(item.clone())?;
            if is_truthy(&result) {
                results.push(item.clone());
            } else {
                break;
            }
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// drop_while(expr, array) -> array
// =============================================================================

/// Drop elements from the beginning of an array while the expression is truthy.
///
/// # Arguments
/// * `expr` - A JMESPath expression string that returns a truthy/falsy value
/// * `array` - The array to process
///
/// # Returns
/// A new array with leading elements removed until the predicate returns false.
///
/// # Example
/// ```text
/// drop_while('@ < `4`', [1, 2, 3, 5, 1, 2]) -> [5, 1, 2]
/// drop_while('@ > `0`', [3, 2, 1, 0, -1]) -> [0, -1]
/// ```
pub struct DropWhileFn {
    signature: Signature,
}

impl Default for DropWhileFn {
    fn default() -> Self {
        Self::new()
    }
}

impl DropWhileFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Array], None),
        }
    }
}

impl Function for DropWhileFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr = args[1].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in drop_while: {}", e)),
            )
        })?;

        let mut dropping = true;
        let mut results = Vec::new();
        for item in arr {
            if dropping {
                let result = compiled.search(item.clone())?;
                if !is_truthy(&result) {
                    dropping = false;
                    results.push(item.clone());
                }
            } else {
                results.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// zip_with(expr, array1, array2) -> array
// =============================================================================

/// Zip two arrays together using a custom combiner expression.
///
/// # Arguments
/// * `expr` - A JMESPath expression that receives `[element1, element2]` as input
/// * `array1` - The first array
/// * `array2` - The second array
///
/// # Returns
/// A new array with elements combined using the expression.
/// The result length is the minimum of the two input array lengths.
///
/// # Example
/// ```text
/// zip_with('add([0], [1])', [1, 2, 3], [10, 20, 30]) -> [11, 22, 33]
/// zip_with('[0] * [1]', [2, 3, 4], [5, 6, 7]) -> [10, 18, 28]
/// ```
pub struct ZipWithFn {
    signature: Signature,
}

impl Default for ZipWithFn {
    fn default() -> Self {
        Self::new()
    }
}

impl ZipWithFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(
                vec![
                    ArgumentType::String,
                    ArgumentType::Array,
                    ArgumentType::Array,
                ],
                None,
            ),
        }
    }
}

impl Function for ZipWithFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();
        let arr1 = args[1].as_array().unwrap();
        let arr2 = args[2].as_array().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in zip_with: {}", e)),
            )
        })?;

        let min_len = arr1.len().min(arr2.len());
        let mut results = Vec::with_capacity(min_len);

        for i in 0..min_len {
            // Create a pair array [element1, element2] as input to the expression
            let pair = Rc::new(Variable::Array(vec![arr1[i].clone(), arr2[i].clone()]));
            let result = compiled.search(pair)?;
            results.push(result);
        }

        Ok(Rc::new(Variable::Array(results)))
    }
}

// =============================================================================
// walk(expr, value) -> value (recursive transformation)
// =============================================================================

/// Recursively apply a transformation to every component of a data structure.
///
/// The transformation is applied bottom-up: for arrays and objects, children
/// are transformed first, then the expression is applied to the result.
///
/// # Arguments
/// * `expr` - A JMESPath expression string to apply at each node
/// * `value` - The value to walk
///
/// # Returns
/// The transformed value.
///
/// # Example
/// ```text
/// walk('if(is_array(@), sort(@), @)', {a: [3, 1, 2]}) -> {a: [1, 2, 3]}
/// walk('if(is_object(@), merge(@, {visited: `true`}), @)', data) -> all objects get visited: true
/// ```
pub struct WalkFn {
    signature: Signature,
}

impl Default for WalkFn {
    fn default() -> Self {
        Self::new()
    }
}

impl WalkFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String, ArgumentType::Any], None),
        }
    }
}

/// Recursively walk a value, applying the expression bottom-up
fn walk_value(value: &Rcvar, compiled: &jmespath::Expression<'_>) -> Result<Rcvar, JmespathError> {
    match &**value {
        Variable::Array(arr) => {
            // First, recursively walk all elements
            let walked_elements: Result<Vec<Rcvar>, _> =
                arr.iter().map(|elem| walk_value(elem, compiled)).collect();
            let new_array = Rc::new(Variable::Array(walked_elements?));
            // Then apply the expression to the array itself
            compiled.search(new_array)
        }
        Variable::Object(obj) => {
            // First, recursively walk all values
            let walked_entries: Result<std::collections::BTreeMap<String, Rcvar>, _> = obj
                .iter()
                .map(|(k, v)| walk_value(v, compiled).map(|walked| (k.clone(), walked)))
                .collect();
            let new_object = Rc::new(Variable::Object(walked_entries?));
            // Then apply the expression to the object itself
            compiled.search(new_object)
        }
        // For scalars (string, number, bool, null), just apply the expression
        _ => compiled.search(value.clone()),
    }
}

impl Function for WalkFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let expr_str = args[0].as_string().unwrap();

        let compiled = ctx.runtime.compile(expr_str).map_err(|e| {
            JmespathError::new(
                ctx.expression,
                ctx.offset,
                ErrorReason::Parse(format!("Invalid expression in walk: {}", e)),
            )
        })?;

        walk_value(&args[1], &compiled)
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
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_any_expr_false() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": false}, {"active": false}]"#).unwrap();
        let expr = runtime.compile("any_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_all_expr_true() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": true}, {"active": true}]"#).unwrap();
        let expr = runtime.compile("all_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_all_expr_false() {
        let runtime = setup();
        let data = Variable::from_json(r#"[{"active": true}, {"active": false}]"#).unwrap();
        let expr = runtime.compile("all_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_all_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("all_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap()); // vacuous truth
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

    #[test]
    fn test_some_alias() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("some('@ > `3`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_every_alias() {
        let runtime = setup();
        let data = Variable::from_json(r#"[2, 4, 6]"#).unwrap();
        let expr = runtime.compile("every('@ > `0`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_reject() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("reject('@ > `2`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // 1, 2
        assert_eq!(arr[0].as_number().unwrap(), 1.0);
        assert_eq!(arr[1].as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_reject_objects() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"active": true}, {"active": false}, {"active": true}]"#)
                .unwrap();
        let expr = runtime.compile("reject('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1); // Only the inactive one
    }

    #[test]
    fn test_map_keys() {
        let runtime = setup();
        // Use length to transform key to its length (as string)
        let data = Variable::from_json(r#"{"abc": 1, "de": 2}"#).unwrap();
        let expr = runtime.compile("map_keys('length(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        // "abc" -> 3, "de" -> 2 (converted to string keys)
        assert!(obj.contains_key("3") || obj.contains_key("2"));
    }

    #[test]
    fn test_map_values_add() {
        let runtime = setup();
        // Use sum to double values - sum of array with value twice
        let data = Variable::from_json(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
        let expr = runtime.compile("map_values('sum(`[1]`)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        // Each value becomes 1 (sum of [1])
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_map_values_length() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"name": "alice", "city": "boston"}"#).unwrap();
        let expr = runtime.compile("map_values('length(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_number().unwrap(), 5.0); // "alice" = 5 chars
        assert_eq!(obj.get("city").unwrap().as_number().unwrap(), 6.0); // "boston" = 6 chars
    }

    #[test]
    #[cfg(feature = "string")]
    fn test_map_values_with_string_fns() {
        // Full integration test with string functions
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        crate::string::register(&mut runtime);

        let data = Variable::from_json(r#"{"name": "alice", "city": "boston"}"#).unwrap();
        let expr = runtime.compile("map_values('upper(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_string().unwrap(), "ALICE");
        assert_eq!(obj.get("city").unwrap().as_string().unwrap(), "BOSTON");
    }

    #[test]
    #[cfg(feature = "string")]
    fn test_map_keys_with_string_fns() {
        // Full integration test with string functions
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        crate::string::register(&mut runtime);

        let data = Variable::from_json(r#"{"hello": 1, "world": 2}"#).unwrap();
        let expr = runtime.compile("map_keys('upper(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("HELLO"));
        assert!(obj.contains_key("WORLD"));
    }

    #[test]
    fn test_order_by_single_field_asc() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"name": "Charlie", "age": 30}, {"name": "Alice", "age": 25}, {"name": "Bob", "age": 35}]"#,
        )
        .unwrap();
        let expr = runtime
            .compile(r#"order_by(@, `[["name", "asc"]]`)"#)
            .unwrap();
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
    fn test_order_by_single_field_desc() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"name": "Alice", "age": 25}, {"name": "Bob", "age": 35}, {"name": "Charlie", "age": 30}]"#,
        )
        .unwrap();
        let expr = runtime
            .compile(r#"order_by(@, `[["age", "desc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_number()
                .unwrap(),
            35.0
        );
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_number()
                .unwrap(),
            30.0
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("age")
                .unwrap()
                .as_number()
                .unwrap(),
            25.0
        );
    }

    #[test]
    fn test_order_by_multiple_fields() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"dept": "sales", "name": "Bob"}, {"dept": "eng", "name": "Alice"}, {"dept": "sales", "name": "Alice"}]"#,
        )
        .unwrap();
        let expr = runtime
            .compile(r#"order_by(@, `[["dept", "asc"], ["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // eng comes first, then sales (sorted by dept)
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("dept")
                .unwrap()
                .as_string()
                .unwrap(),
            "eng"
        );
        // Within sales, Alice comes before Bob
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Alice"
        );
        assert_eq!(
            arr[2]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Bob"
        );
    }

    #[test]
    fn test_reduce_expr_sum() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime
            .compile("reduce_expr('sum([accumulator, current])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 15.0);
    }

    #[test]
    fn test_reduce_expr_max() {
        let runtime = setup();
        let data = Variable::from_json(r#"[3, 1, 4, 1, 5, 9, 2, 6]"#).unwrap();
        let expr = runtime
            .compile("reduce_expr('max([accumulator, current])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 9.0);
    }

    #[test]
    fn test_reduce_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime
            .compile("reduce_expr('sum([accumulator, current])', @, `42`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0); // Returns initial value
    }

    #[test]
    fn test_fold_alias() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime
            .compile("fold('sum([accumulator, current])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 6.0);
    }

    #[test]
    fn test_scan_expr_running_sum() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 4]"#).unwrap();
        let expr = runtime
            .compile("scan_expr('sum([accumulator, current])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Running sum: [1, 3, 6, 10]
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_number().unwrap(), 1.0);
        assert_eq!(arr[1].as_number().unwrap(), 3.0);
        assert_eq!(arr[2].as_number().unwrap(), 6.0);
        assert_eq!(arr[3].as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_scan_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime
            .compile("scan_expr('sum([accumulator, current])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_reduce_expr_with_index() {
        let runtime = setup();
        // Access the index in the reduce expression
        let data = Variable::from_json(r#"[10, 20, 30]"#).unwrap();
        let expr = runtime
            .compile("reduce_expr('sum([accumulator, index])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // 0 + 1 + 2 = 3
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_count_by_objects() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"type": "a"}, {"type": "b"}, {"type": "a"}, {"type": "a"}]"#)
                .unwrap();
        let expr = runtime.compile("count_by('type', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 3.0);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_count_by_strings() {
        let runtime = setup();
        let data = Variable::from_json(r#"["a", "b", "a", "c", "a"]"#).unwrap();
        let expr = runtime.compile("count_by('@', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap(), 3.0);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap(), 1.0);
        assert_eq!(obj.get("c").unwrap().as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_count_by_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("count_by('type', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_count_by_numbers() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 1, 3, 1, 2]"#).unwrap();
        let expr = runtime.compile("count_by('@', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("1").unwrap().as_number().unwrap(), 3.0);
        assert_eq!(obj.get("2").unwrap().as_number().unwrap(), 2.0);
        assert_eq!(obj.get("3").unwrap().as_number().unwrap(), 1.0);
    }

    // =============================================================================
    // Partial application tests
    // =============================================================================

    #[test]
    fn test_partial_creates_object() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("partial('length')").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("__partial__").unwrap().as_boolean().unwrap());
        assert_eq!(obj.get("fn").unwrap().as_string().unwrap(), "length");
        assert!(obj.get("args").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn test_partial_with_args() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime
            .compile("partial('contains', `\"hello world\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("__partial__").unwrap().as_boolean().unwrap());
        assert_eq!(obj.get("fn").unwrap().as_string().unwrap(), "contains");
        let args = obj.get("args").unwrap().as_array().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_apply_with_fn_name() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("apply('length', `\"hello\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_apply_with_partial() {
        let runtime = setup();
        let data = Variable::Null;
        // Create partial with first arg, then apply with second arg
        let expr = runtime
            .compile("apply(partial('contains', `\"hello world\"`), `\"world\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_apply_partial_not_found() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime
            .compile("apply(partial('contains', `\"hello world\"`), `\"xyz\"`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_partial_with_multiple_prefilled_args() {
        let runtime = setup();
        let data = Variable::Null;
        // partial with 2 args pre-filled
        let expr = runtime.compile("partial('join', `\"-\"`)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        let args = obj.get("args").unwrap().as_array().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].as_string().unwrap(), "-");
    }

    #[test]
    fn test_apply_partial_join() {
        let runtime = setup();
        let data = Variable::Null;
        // Create a join with "-" separator, then apply to array
        let expr = runtime
            .compile("apply(partial('join', `\"-\"`), `[\"a\", \"b\", \"c\"]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "a-b-c");
    }

    // =========================================================================
    // Pipeline pattern tests
    // =========================================================================

    #[test]
    fn test_pipeline_filter_sort_products() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"{
                "products": [
                    {"name": "A", "price": 30, "in_stock": true},
                    {"name": "B", "price": 10, "in_stock": true},
                    {"name": "C", "price": 20, "in_stock": false},
                    {"name": "D", "price": 5, "in_stock": true}
                ]
            }"#,
        )
        .unwrap();
        let expr = runtime
            .compile("products | filter_expr('in_stock', @) | sort_by_expr('price', @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "D"
        ); // $5
        assert_eq!(
            arr[1]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "B"
        ); // $10
    }

    #[test]
    fn test_pipeline_funnel_errors() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"{
                "events": [
                    {"level": "error", "timestamp": 1704067300, "message": "Disk full"},
                    {"level": "info", "timestamp": 1704067200, "message": "Started"},
                    {"level": "error", "timestamp": 1704067400, "message": "Connection lost"},
                    {"level": "warn", "timestamp": 1704067350, "message": "High memory"}
                ]
            }"#,
        )
        .unwrap();
        let expr = runtime
            .compile(
                r#"events | filter_expr('level == `"error"`', @) | sort_by_expr('timestamp', @)"#,
            )
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Sorted by timestamp ascending
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("message")
                .unwrap()
                .as_string()
                .unwrap(),
            "Disk full"
        );
    }

    #[test]
    fn test_pipeline_transactions_completed() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"{
                "transactions": [
                    {"amount": 100, "status": "completed"},
                    {"amount": 50, "status": "completed"},
                    {"amount": 75, "status": "pending"},
                    {"amount": 200, "status": "completed"}
                ]
            }"#,
        )
        .unwrap();
        let expr = runtime
            .compile(r#"transactions | filter_expr('status == `"completed"`', @) | map_expr('amount', @)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap(), 100.0);
        assert_eq!(arr[1].as_number().unwrap(), 50.0);
        assert_eq!(arr[2].as_number().unwrap(), 200.0);
    }

    #[test]
    fn test_pipeline_fork_join() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"{
                "items": [
                    {"name": "A", "price": 150},
                    {"name": "B", "price": 50},
                    {"name": "C", "price": 200},
                    {"name": "D", "price": 25}
                ]
            }"#,
        )
        .unwrap();
        let expr = runtime
            .compile(
                r#"@.{
                    expensive: items | filter_expr('price > `100`', @),
                    cheap: items | filter_expr('price <= `100`', @)
                }"#,
            )
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("expensive").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("cheap").unwrap().as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_pipeline_nested_users() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"{
                "users": [
                    {"name": "Alice", "orders": [{"total": 100}, {"total": 50}]},
                    {"name": "Bob", "orders": [{"total": 200}]},
                    {"name": "Carol", "orders": []}
                ]
            }"#,
        )
        .unwrap();
        // Filter users with orders, then map to get names
        let expr = runtime
            .compile("users | filter_expr('length(orders) > `0`', @) | map_expr('name', @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "Alice");
        assert_eq!(arr[1].as_string().unwrap(), "Bob");
    }

    #[test]
    fn test_pipeline_rag_chunks() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"{
                "chunks": [
                    {"content": "Redis is fast", "score": 0.9},
                    {"content": "Redis is in-memory", "score": 0.85},
                    {"content": "Unrelated content", "score": 0.5},
                    {"content": "Redis supports modules", "score": 0.75}
                ]
            }"#,
        )
        .unwrap();
        let expr = runtime
            .compile("chunks | filter_expr('score > `0.7`', @) | sort_by_expr('score', @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // Sorted ascending by score
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("score")
                .unwrap()
                .as_number()
                .unwrap(),
            0.75
        );
    }

    // =========================================================================
    // Additional reduce_expr/scan_expr tests
    // =========================================================================

    #[test]
    fn test_reduce_expr_product() {
        let runtime = setup();
        // Test reduce with min (similar to existing max test but finds minimum)
        let data = Variable::from_json(r#"[5, 3, 8, 1, 9]"#).unwrap();
        let expr = runtime
            .compile("reduce_expr('min([accumulator, current])', @, `100`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_scan_expr_running_balance() {
        let runtime = setup();
        // Test scan with running max - shows progressive maximum
        let data = Variable::from_json(r#"[3, 1, 4, 1, 5, 9]"#).unwrap();
        let expr = runtime
            .compile("scan_expr('max([accumulator, current])', @, `0`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Running max: 3, 3, 4, 4, 5, 9
        assert_eq!(arr[0].as_number().unwrap(), 3.0);
        assert_eq!(arr[1].as_number().unwrap(), 3.0);
        assert_eq!(arr[2].as_number().unwrap(), 4.0);
        assert_eq!(arr[3].as_number().unwrap(), 4.0);
        assert_eq!(arr[4].as_number().unwrap(), 5.0);
        assert_eq!(arr[5].as_number().unwrap(), 9.0);
    }

    // =========================================================================
    // Additional order_by tests
    // =========================================================================

    #[test]
    fn test_order_by_three_fields() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[
                {"dept": "Engineering", "level": "senior", "name": "Charlie"},
                {"dept": "Engineering", "level": "junior", "name": "Alice"},
                {"dept": "Engineering", "level": "senior", "name": "Bob"},
                {"dept": "Sales", "level": "senior", "name": "David"}
            ]"#,
        )
        .unwrap();
        let expr = runtime
            .compile(r#"order_by(@, `[["dept", "asc"], ["level", "desc"], ["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Engineering seniors first (alphabetical), then Engineering juniors, then Sales
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("name")
                .unwrap()
                .as_string()
                .unwrap(),
            "Bob"
        );
        assert_eq!(
            arr[1]
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
    fn test_order_by_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime
            .compile(r#"order_by(@, `[["name", "asc"]]`)"#)
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert!(arr.is_empty());
    }

    // =========================================================================
    // Additional partition_expr tests
    // =========================================================================

    #[test]
    fn test_partition_expr_scores() {
        let runtime = setup();
        let data = Variable::from_json(r#"[85, 42, 91, 67, 55, 78, 33, 99]"#).unwrap();
        let expr = runtime.compile("partition_expr('@ >= `60`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        let passing = arr[0].as_array().unwrap();
        let failing = arr[1].as_array().unwrap();
        assert_eq!(passing.len(), 5); // 85, 91, 67, 78, 99
        assert_eq!(failing.len(), 3); // 42, 55, 33
    }

    #[test]
    fn test_partition_expr_active() {
        let runtime = setup();
        let data =
            Variable::from_json(r#"[{"active": true}, {"active": false}, {"active": true}]"#)
                .unwrap();
        let expr = runtime.compile("partition_expr('active', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_array().unwrap().len(), 2);
        assert_eq!(arr[1].as_array().unwrap().len(), 1);
    }

    // =========================================================================
    // Additional map_values/map_keys tests
    // =========================================================================

    #[test]
    fn test_map_values_discount() {
        let runtime = setup();
        // Test with string transformation since nested expressions don't have extension math functions
        let data = Variable::from_json(r#"{"apple": "FRUIT", "banana": "ITEM"}"#).unwrap();
        let expr = runtime.compile("map_values('length(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("apple").unwrap().as_number().unwrap(), 5.0);
        assert_eq!(obj.get("banana").unwrap().as_number().unwrap(), 4.0);
    }

    // =========================================================================
    // Additional group_by_expr tests
    // =========================================================================

    #[test]
    fn test_group_by_expr_type() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"type": "fruit", "name": "apple"}, {"type": "vegetable", "name": "carrot"}, {"type": "fruit", "name": "banana"}]"#,
        )
        .unwrap();
        let expr = runtime.compile("group_by_expr('type', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("fruit").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("vegetable").unwrap().as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_group_by_expr_computed() {
        let runtime = setup();
        // Group strings by their length using built-in length function
        let data = Variable::from_json(r#"["a", "bb", "ccc", "dd", "eee", "f"]"#).unwrap();
        let expr = runtime
            .compile("group_by_expr('to_string(length(@))', @)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("1")); // "a", "f"
        assert!(obj.contains_key("2")); // "bb", "dd"
        assert!(obj.contains_key("3")); // "ccc", "eee"
        assert_eq!(obj.get("1").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("2").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("3").unwrap().as_array().unwrap().len(), 2);
    }

    // =========================================================================
    // Additional unique_by_expr tests
    // =========================================================================

    #[test]
    fn test_unique_by_expr_id() {
        let runtime = setup();
        let data = Variable::from_json(
            r#"[{"id": 1, "v": "a"}, {"id": 2, "v": "b"}, {"id": 1, "v": "c"}]"#,
        )
        .unwrap();
        let expr = runtime.compile("unique_by_expr('id', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        // Keeps first occurrence
        assert_eq!(
            arr[0]
                .as_object()
                .unwrap()
                .get("v")
                .unwrap()
                .as_string()
                .unwrap(),
            "a"
        );
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_any_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("any_expr('@ > `0`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_max_by_expr_empty() {
        let runtime = setup();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("max_by_expr('age', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_flat_map_expr_duplicate() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        // Duplicate each element
        let expr = runtime.compile("flat_map_expr('[@, @]', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 6);
    }

    #[test]
    fn test_reject_greater_than() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6]"#).unwrap();
        let expr = runtime.compile("reject('@ > `3`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 1, 2, 3
    }

    #[test]
    fn test_every_false_case() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, -1, 3]"#).unwrap();
        let expr = runtime.compile("every('@ > `0`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_count_expr_all_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[5, 10, 15, 20]"#).unwrap();
        let expr = runtime.compile("count_expr('@ > `0`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);
    }

    #[test]
    fn test_find_expr_first_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 5, 10, 15]"#).unwrap();
        let expr = runtime.compile("find_expr('@ > `3`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);
    }

    #[test]
    fn test_find_index_expr_first_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 5, 10, 15]"#).unwrap();
        let expr = runtime.compile("find_index_expr('@ > `3`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_take_while_basic() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 5, 1, 2]"#).unwrap();
        let expr = runtime.compile("take_while('@ < `4`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap(), 1.0);
        assert_eq!(arr[1].as_number().unwrap(), 2.0);
        assert_eq!(arr[2].as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_take_while_all_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("take_while('@ < `10`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_take_while_none_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[5, 6, 7]"#).unwrap();
        let expr = runtime.compile("take_while('@ < `4`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_drop_while_basic() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3, 5, 1, 2]"#).unwrap();
        let expr = runtime.compile("drop_while('@ < `4`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap(), 5.0);
        assert_eq!(arr[1].as_number().unwrap(), 1.0);
        assert_eq!(arr[2].as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_drop_while_all_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("drop_while('@ < `10`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_drop_while_none_match() {
        let runtime = setup();
        let data = Variable::from_json(r#"[5, 6, 7]"#).unwrap();
        let expr = runtime.compile("drop_while('@ < `4`', @)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_zip_with_add() {
        let mut runtime = setup();
        crate::math::register(&mut runtime);
        let data = Variable::Null;
        let expr = runtime
            .compile("zip_with('add([0], [1])', `[1, 2, 3]`, `[10, 20, 30]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap(), 11.0);
        assert_eq!(arr[1].as_number().unwrap(), 22.0);
        assert_eq!(arr[2].as_number().unwrap(), 33.0);
    }

    #[test]
    fn test_zip_with_unequal_lengths() {
        let mut runtime = setup();
        crate::math::register(&mut runtime);
        let data = Variable::Null;
        let expr = runtime
            .compile("zip_with('add([0], [1])', `[1, 2, 3, 4, 5]`, `[10, 20]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number().unwrap(), 11.0);
        assert_eq!(arr[1].as_number().unwrap(), 22.0);
    }

    #[test]
    fn test_zip_with_multiply() {
        let mut runtime = setup();
        crate::math::register(&mut runtime);
        let data = Variable::Null;
        let expr = runtime
            .compile("zip_with('multiply([0], [1])', `[2, 3, 4]`, `[5, 6, 7]`)")
            .unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap(), 10.0);
        assert_eq!(arr[1].as_number().unwrap(), 18.0);
        assert_eq!(arr[2].as_number().unwrap(), 28.0);
    }

    // =========================================================================
    // walk tests
    // =========================================================================

    #[test]
    fn test_walk_identity() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"a": [1, 2, 3], "b": {"c": 4}}"#).unwrap();
        let expr = runtime.compile("walk('@', @)").unwrap();
        let result = expr.search(&data).unwrap();
        // Identity should return the same structure
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("b"));
    }

    #[test]
    fn test_walk_type_of_all() {
        let mut runtime = setup();
        crate::type_conv::register(&mut runtime);
        let data = Variable::from_json(r#"{"a": 5, "b": [1, 2]}"#).unwrap();
        // type() works on everything - shows bottom-up processing
        let expr = runtime.compile("walk('type(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        // After walking, everything becomes its type string, and the final result
        // is type of the top-level result
        assert_eq!(result.as_string().unwrap(), "object");
    }

    #[test]
    fn test_walk_nested_arrays() {
        let runtime = setup();
        // Use only arrays (no scalars inside) so length works at every level
        let data = Variable::from_json(r#"[[[], []], [[]]]"#).unwrap();
        // length works on arrays - get lengths at each level
        let expr = runtime.compile("walk('length(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        // Inner [] -> 0, outer arrays get lengths, top level has 2 elements
        assert_eq!(result.as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_walk_scalar() {
        let mut runtime = setup();
        crate::math::register(&mut runtime);
        let data = Variable::Number(serde_json::Number::from(5));
        // Double the number
        let expr = runtime.compile("walk('multiply(@, `2`)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_walk_length_all() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"items": ["a", "bb", "ccc"]}"#).unwrap();
        // Get length of everything (works for strings, arrays, objects)
        let expr = runtime.compile("walk('length(@)', @)").unwrap();
        let result = expr.search(&data).unwrap();
        // Top level object has 1 key
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_walk_preserves_structure() {
        let runtime = setup();
        let data = Variable::from_json(r#"{"a": [1, {"b": 2}], "c": "hello"}"#).unwrap();
        // Identity transform - should preserve structure
        let expr = runtime.compile("walk('@', @)").unwrap();
        let result = expr.search(&data).unwrap();

        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("a"));
        assert!(obj.contains_key("c"));
        let arr = obj.get("a").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_walk_empty_structures() {
        let runtime = setup();

        // Empty array
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("walk('@', @)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_array().unwrap().is_empty());

        // Empty object
        let data = Variable::from_json(r#"{}"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.as_object().unwrap().is_empty());
    }
}
