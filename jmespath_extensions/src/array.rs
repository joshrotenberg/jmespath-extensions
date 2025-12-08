//! Array manipulation functions.
//!
//! This module provides extended array operations beyond the standard JMESPath built-ins.
//!
//! # Function Reference
//!
//! | Function | Signature | Description |
//! |----------|-----------|-------------|
//! | [`first`](#first) | `first(array) → any` | Get first element |
//! | [`last`](#last) | `last(array) → any` | Get last element |
//! | [`unique`](#unique) | `unique(array) → array` | Remove duplicates |
//! | [`take`](#take) | `take(array, n) → array` | Take first N elements |
//! | [`drop`](#drop) | `drop(array, n) → array` | Drop first N elements |
//! | [`chunk`](#chunk) | `chunk(array, size) → array` | Split into chunks |
//! | [`zip`](#zip) | `zip(array1, array2) → array` | Zip two arrays |
//! | [`flatten_deep`](#flatten_deep) | `flatten_deep(array) → array` | Recursively flatten |
//! | [`compact`](#compact) | `compact(array) → array` | Remove null/false values |
//! | [`range`](#range) | `range(start, end, step?) → array` | Generate number range |
//! | [`index_at`](#index_at) | `index_at(array, index) → any` | Get element at index |
//! | [`includes`](#includes) | `includes(array, value) → boolean` | Check if array contains value |
//! | [`find_index`](#find_index) | `find_index(array, value) → number` | Find index of value |
//! | [`group_by`](#group_by) | `group_by(array, field) → object` | Group by field |
//! | [`nth`](#nth) | `nth(array, n) → array` | Get every nth element |
//! | [`interleave`](#interleave) | `interleave(array1, array2) → array` | Interleave two arrays |
//! | [`rotate`](#rotate) | `rotate(array, n) → array` | Rotate by N positions |
//! | [`partition`](#partition) | `partition(array, n) → array` | Split into N parts |
//! | [`difference`](#difference) | `difference(array1, array2) → array` | Set difference |
//! | [`intersection`](#intersection) | `intersection(array1, array2) → array` | Set intersection |
//! | [`union`](#union) | `union(array1, array2) → array` | Set union |
//! | [`frequencies`](#frequencies) | `frequencies(array) → object` | Count occurrences |
//! | [`mode`](#mode) | `mode(array) → any` | Most frequent element |
//! | [`cartesian`](#cartesian) | `cartesian(array1, array2) → array` | Cartesian product |
//!
//! # Examples
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::array;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! array::register(&mut runtime);
//!
//! // Get first element
//! let expr = runtime.compile("first(@)").unwrap();
//! let data = Variable::from_json("[1, 2, 3]").unwrap();
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_number().unwrap() as i64, 1);
//!
//! // Remove duplicates
//! let expr = runtime.compile("unique(@)").unwrap();
//! let data = Variable::from_json("[1, 2, 2, 3, 1]").unwrap();
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_array().unwrap().len(), 3);
//! ```
//!
//! # Function Details
//!
//! ## first
//!
//! Returns the first element of an array, or null if empty.
//!
//! ```text
//! first(array) → any
//!
//! first([1, 2, 3])        → 1
//! first(['a', 'b'])       → "a"
//! first([])               → null
//! ```
//!
//! ## last
//!
//! Returns the last element of an array, or null if empty.
//!
//! ```text
//! last(array) → any
//!
//! last([1, 2, 3])         → 3
//! last(['a', 'b'])        → "b"
//! last([])                → null
//! ```
//!
//! ## unique
//!
//! Returns an array with duplicate values removed. Order is preserved.
//!
//! ```text
//! unique(array) → array
//!
//! unique([1, 2, 2, 3, 1]) → [1, 2, 3]
//! unique(['a', 'b', 'a']) → ["a", "b"]
//! unique([1, '1'])        → [1, "1"]    // Different types are distinct
//! ```
//!
//! ## take
//!
//! Returns the first N elements of an array.
//!
//! ```text
//! take(array, n) → array
//!
//! take([1, 2, 3, 4, 5], 3)   → [1, 2, 3]
//! take([1, 2], 5)            → [1, 2]      // Returns all if n > length
//! take([1, 2, 3], 0)         → []
//! ```
//!
//! ## drop
//!
//! Returns the array with the first N elements removed.
//!
//! ```text
//! drop(array, n) → array
//!
//! drop([1, 2, 3, 4, 5], 2)   → [3, 4, 5]
//! drop([1, 2], 5)            → []          // Empty if n >= length
//! drop([1, 2, 3], 0)         → [1, 2, 3]
//! ```
//!
//! ## chunk
//!
//! Splits an array into chunks of the specified size.
//!
//! ```text
//! chunk(array, size) → array
//!
//! chunk([1, 2, 3, 4, 5], 2)  → [[1, 2], [3, 4], [5]]
//! chunk([1, 2, 3], 1)        → [[1], [2], [3]]
//! chunk([1, 2, 3], 5)        → [[1, 2, 3]]
//! ```
//!
//! ## zip
//!
//! Combines two arrays into an array of pairs. Stops at shorter array.
//!
//! ```text
//! zip(array1, array2) → array
//!
//! zip([1, 2, 3], ['a', 'b', 'c'])   → [[1, "a"], [2, "b"], [3, "c"]]
//! zip([1, 2], ['a', 'b', 'c'])      → [[1, "a"], [2, "b"]]
//! zip([], [1, 2])                   → []
//! ```
//!
//! ## flatten_deep
//!
//! Recursively flattens nested arrays into a single-level array.
//!
//! ```text
//! flatten_deep(array) → array
//!
//! flatten_deep([[1, 2], [3, 4]])          → [1, 2, 3, 4]
//! flatten_deep([1, [2, [3, [4]]]])        → [1, 2, 3, 4]
//! flatten_deep([1, 2, 3])                 → [1, 2, 3]
//! ```
//!
//! ## compact
//!
//! Removes null and false values from an array.
//!
//! ```text
//! compact(array) → array
//!
//! compact([1, null, 2, false, 3])   → [1, 2, 3]
//! compact([0, '', null, true])      → [0, "", true]  // 0 and '' are kept
//! compact([null, false])            → []
//! ```
//!
//! ## range
//!
//! Generates an array of numbers from start (inclusive) to end (exclusive).
//!
//! ```text
//! range(start, end, step?) → array
//!
//! range(0, 5)           → [0, 1, 2, 3, 4]
//! range(1, 10, 2)       → [1, 3, 5, 7, 9]
//! range(5, 0, -1)       → [5, 4, 3, 2, 1]
//! range(0, 0)           → []
//! ```
//!
//! ## index_at
//!
//! Gets the element at the specified index. Supports negative indices.
//!
//! ```text
//! index_at(array, index) → any
//!
//! index_at([1, 2, 3], 0)    → 1
//! index_at([1, 2, 3], 2)    → 3
//! index_at([1, 2, 3], -1)   → 3     // Last element
//! index_at([1, 2, 3], -2)   → 2     // Second to last
//! index_at([1, 2, 3], 10)   → null  // Out of bounds
//! ```
//!
//! ## includes
//!
//! Checks if an array contains a value.
//!
//! ```text
//! includes(array, value) → boolean
//!
//! includes([1, 2, 3], 2)        → true
//! includes([1, 2, 3], 4)        → false
//! includes(['a', 'b'], 'a')     → true
//! includes([{a: 1}], {a: 1})    → true   // Deep equality
//! ```
//!
//! ## find_index
//!
//! Finds the index of a value in an array. Returns -1 if not found.
//!
//! ```text
//! find_index(array, value) → number
//!
//! find_index([1, 2, 3], 2)      → 1
//! find_index([1, 2, 3], 4)      → -1
//! find_index(['a', 'b'], 'b')   → 1
//! ```
//!
//! ## group_by
//!
//! Groups array elements by a field value.
//!
//! ```text
//! group_by(array, field) → object
//!
//! group_by([{type: 'a', v: 1}, {type: 'b', v: 2}, {type: 'a', v: 3}], 'type')
//!     → {a: [{type: 'a', v: 1}, {type: 'a', v: 3}], b: [{type: 'b', v: 2}]}
//! ```
//!
//! ## nth
//!
//! Returns every nth element of an array.
//!
//! ```text
//! nth(array, n) → array
//!
//! nth([1, 2, 3, 4, 5, 6], 2)   → [1, 3, 5]
//! nth([1, 2, 3, 4, 5], 3)      → [1, 4]
//! nth([1, 2, 3], 1)            → [1, 2, 3]
//! ```
//!
//! ## interleave
//!
//! Interleaves elements from two arrays.
//!
//! ```text
//! interleave(array1, array2) → array
//!
//! interleave([1, 2, 3], ['a', 'b', 'c'])   → [1, "a", 2, "b", 3, "c"]
//! interleave([1, 2], ['a', 'b', 'c'])      → [1, "a", 2, "b", "c"]
//! interleave([1], [])                      → [1]
//! ```
//!
//! ## rotate
//!
//! Rotates array elements by N positions. Positive N rotates left.
//!
//! ```text
//! rotate(array, n) → array
//!
//! rotate([1, 2, 3, 4, 5], 2)    → [3, 4, 5, 1, 2]
//! rotate([1, 2, 3, 4, 5], -1)   → [5, 1, 2, 3, 4]
//! rotate([1, 2, 3], 0)          → [1, 2, 3]
//! ```
//!
//! ## partition
//!
//! Splits an array into N roughly equal parts.
//!
//! ```text
//! partition(array, n) → array
//!
//! partition([1, 2, 3, 4, 5], 2)   → [[1, 2, 3], [4, 5]]
//! partition([1, 2, 3, 4], 3)      → [[1, 2], [3], [4]]
//! partition([1, 2], 4)            → [[1], [2], [], []]
//! ```
//!
//! ## difference
//!
//! Returns elements in the first array that are not in the second.
//!
//! ```text
//! difference(array1, array2) → array
//!
//! difference([1, 2, 3, 4], [2, 4])      → [1, 3]
//! difference([1, 2], [3, 4])            → [1, 2]
//! difference([1, 2], [1, 2])            → []
//! ```
//!
//! ## intersection
//!
//! Returns elements that exist in both arrays (unique).
//!
//! ```text
//! intersection(array1, array2) → array
//!
//! intersection([1, 2, 3], [2, 3, 4])    → [2, 3]
//! intersection([1, 2], [3, 4])          → []
//! intersection([1, 1, 2], [1, 2, 2])    → [1, 2]
//! ```
//!
//! ## union
//!
//! Returns unique elements from both arrays combined.
//!
//! ```text
//! union(array1, array2) → array
//!
//! union([1, 2], [2, 3])         → [1, 2, 3]
//! union([1, 1], [2, 2])         → [1, 2]
//! union([], [1, 2])             → [1, 2]
//! ```
//!
//! ## frequencies
//!
//! Counts the occurrences of each value in an array.
//!
//! ```text
//! frequencies(array) → object
//!
//! frequencies(['a', 'b', 'a', 'c', 'a'])   → {a: 3, b: 1, c: 1}
//! frequencies([1, 2, 1, 1])                → {"1": 3, "2": 1}
//! frequencies([])                          → {}
//! ```
//!
//! ## mode
//!
//! Returns the most frequently occurring value in an array.
//!
//! ```text
//! mode(array) → any
//!
//! mode([1, 2, 2, 3, 2])     → 2
//! mode(['a', 'b', 'a'])     → "a"
//! mode([1, 2, 3])           → 1      // First in case of tie
//! mode([])                  → null
//! ```
//!
//! ## cartesian
//!
//! Returns the Cartesian product of two arrays.
//!
//! ```text
//! cartesian(array1, array2) → array
//!
//! cartesian([1, 2], ['a', 'b'])   → [[1, "a"], [1, "b"], [2, "a"], [2, "b"]]
//! cartesian([1], [2, 3])          → [[1, 2], [1, 3]]
//! cartesian([], [1, 2])           → []
//! ```

use std::collections::HashSet;
use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all array functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("unique", Box::new(UniqueFn::new()));
    runtime.register_function("zip", Box::new(ZipFn::new()));
    runtime.register_function("chunk", Box::new(ChunkFn::new()));
    runtime.register_function("take", Box::new(TakeFn::new()));
    runtime.register_function("drop", Box::new(DropFn::new()));
    runtime.register_function("flatten_deep", Box::new(FlattenDeepFn::new()));
    runtime.register_function("compact", Box::new(CompactFn::new()));
    runtime.register_function("range", Box::new(RangeFn::new()));
    runtime.register_function("index_at", Box::new(IndexAtFn::new()));
    runtime.register_function("includes", Box::new(IncludesFn::new()));
    runtime.register_function("find_index", Box::new(FindIndexFn::new()));
    runtime.register_function("first", Box::new(FirstFn::new()));
    runtime.register_function("last", Box::new(LastFn::new()));
    runtime.register_function("group_by", Box::new(GroupByFn::new()));
    runtime.register_function("nth", Box::new(NthFn::new()));
    runtime.register_function("interleave", Box::new(InterleaveFn::new()));
    runtime.register_function("rotate", Box::new(RotateFn::new()));
    runtime.register_function("partition", Box::new(PartitionFn::new()));
    runtime.register_function("difference", Box::new(DifferenceFn::new()));
    runtime.register_function("intersection", Box::new(IntersectionFn::new()));
    runtime.register_function("union", Box::new(UnionFn::new()));
    runtime.register_function("frequencies", Box::new(FrequenciesFn::new()));
    runtime.register_function("mode", Box::new(ModeFn::new()));
    runtime.register_function("cartesian", Box::new(CartesianFn::new()));
}

// =============================================================================
// unique(array) -> array
// =============================================================================

define_function!(UniqueFn, vec![ArgumentType::Array], None);

impl Function for UniqueFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in arr {
            let key = serde_json::to_string(&**item).unwrap_or_default();
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// zip(array1, array2) -> array of pairs
// =============================================================================

define_function!(ZipFn, vec![ArgumentType::Array, ArgumentType::Array], None);

impl Function for ZipFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let arr2 = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let result: Vec<Rcvar> = arr1
            .iter()
            .zip(arr2.iter())
            .map(|(a, b)| Rc::new(Variable::Array(vec![a.clone(), b.clone()])) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// chunk(array, size) -> array of arrays
// =============================================================================

define_function!(
    ChunkFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for ChunkFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let size = args[1].as_number().map(|n| n as usize).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected positive number for size".to_owned()),
            )
        })?;

        if size == 0 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let chunks: Vec<Rcvar> = arr
            .chunks(size)
            .map(|chunk| Rc::new(Variable::Array(chunk.to_vec())) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(chunks)))
    }
}

// =============================================================================
// take(array, n) -> array (first n elements)
// =============================================================================

define_function!(
    TakeFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for TakeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let n = args[1].as_number().map(|n| n as usize).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected positive number".to_owned()),
            )
        })?;

        let result: Vec<Rcvar> = arr.iter().take(n).cloned().collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// drop(array, n) -> array (skip first n elements)
// =============================================================================

define_function!(
    DropFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for DropFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let n = args[1].as_number().map(|n| n as usize).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected positive number".to_owned()),
            )
        })?;

        let result: Vec<Rcvar> = arr.iter().skip(n).cloned().collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// flatten_deep(array) -> array (recursively flatten)
// =============================================================================

define_function!(FlattenDeepFn, vec![ArgumentType::Array], None);

fn flatten_recursive(arr: &[Rcvar]) -> Vec<Rcvar> {
    let mut result = Vec::new();
    for item in arr {
        if let Some(inner) = item.as_array() {
            result.extend(flatten_recursive(inner));
        } else {
            result.push(item.clone());
        }
    }
    result
}

impl Function for FlattenDeepFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Array(flatten_recursive(arr))))
    }
}

// =============================================================================
// compact(array) -> array (remove null/false values)
// =============================================================================

define_function!(CompactFn, vec![ArgumentType::Array], None);

impl Function for CompactFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let result: Vec<Rcvar> = arr
            .iter()
            .filter(|v| !v.is_null() && !matches!(&***v, Variable::Bool(false)))
            .cloned()
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// range(start, end, step?) -> array
// =============================================================================

define_function!(
    RangeFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    Some(ArgumentType::Number)
);

impl Function for RangeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let start = args[0].as_number().map(|n| n as i64).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected start number".to_owned()),
            )
        })?;

        let end = args[1].as_number().map(|n| n as i64).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected end number".to_owned()),
            )
        })?;

        let step = if args.len() > 2 {
            args[2].as_number().map(|n| n as i64).ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected step number".to_owned()),
                )
            })?
        } else {
            1
        };

        if step == 0 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Step cannot be zero".to_owned()),
            ));
        }

        let mut result = Vec::new();
        let mut current = start;

        const MAX_RANGE: usize = 10000;

        if step > 0 {
            while current < end && result.len() < MAX_RANGE {
                result.push(Rc::new(Variable::Number(serde_json::Number::from(current))) as Rcvar);
                current += step;
            }
        } else {
            while current > end && result.len() < MAX_RANGE {
                result.push(Rc::new(Variable::Number(serde_json::Number::from(current))) as Rcvar);
                current += step;
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// index_at(array, index) -> element (supports negative index)
// =============================================================================

define_function!(
    IndexAtFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for IndexAtFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let index = args[1].as_number().map(|n| n as i64).ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for index".to_owned()),
            )
        })?;

        let len = arr.len() as i64;
        let actual_index = if index < 0 {
            (len + index) as usize
        } else {
            index as usize
        };

        if actual_index < arr.len() {
            Ok(arr[actual_index].clone())
        } else {
            Ok(Rc::new(Variable::Null))
        }
    }
}

// =============================================================================
// includes(array, value) -> boolean
// =============================================================================

define_function!(
    IncludesFn,
    vec![ArgumentType::Array, ArgumentType::Any],
    None
);

impl Function for IncludesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let search_key = serde_json::to_string(&*args[1]).unwrap_or_default();

        let found = arr.iter().any(|item| {
            let item_key = serde_json::to_string(&**item).unwrap_or_default();
            item_key == search_key
        });

        Ok(Rc::new(Variable::Bool(found)))
    }
}

// =============================================================================
// find_index(array, value) -> number (-1 if not found)
// =============================================================================

define_function!(
    FindIndexFn,
    vec![ArgumentType::Array, ArgumentType::Any],
    None
);

impl Function for FindIndexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let search_key = serde_json::to_string(&*args[1]).unwrap_or_default();

        let index = arr
            .iter()
            .position(|item| {
                let item_key = serde_json::to_string(&**item).unwrap_or_default();
                item_key == search_key
            })
            .map(|i| i as i64)
            .unwrap_or(-1);

        Ok(Rc::new(Variable::Number(serde_json::Number::from(index))))
    }
}

// =============================================================================
// first(array) -> any (first element or null)
// =============================================================================

define_function!(FirstFn, vec![ArgumentType::Array], None);

impl Function for FirstFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        Ok(arr
            .first()
            .cloned()
            .unwrap_or_else(|| Rc::new(Variable::Null)))
    }
}

// =============================================================================
// last(array) -> any (last element or null)
// =============================================================================

define_function!(LastFn, vec![ArgumentType::Array], None);

impl Function for LastFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        Ok(arr
            .last()
            .cloned()
            .unwrap_or_else(|| Rc::new(Variable::Null)))
    }
}

// =============================================================================
// group_by(array, field_name) -> object
// =============================================================================

define_function!(
    GroupByFn,
    vec![ArgumentType::Array, ArgumentType::String],
    None
);

impl Function for GroupByFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let field_name = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected field name string".to_owned()),
            )
        })?;

        let mut groups: std::collections::BTreeMap<String, Vec<Rcvar>> =
            std::collections::BTreeMap::new();

        for item in arr {
            let key = if let Some(obj) = item.as_object() {
                if let Some(field_value) = obj.get(field_name) {
                    match &**field_value {
                        Variable::String(s) => s.clone(),
                        Variable::Number(n) => n.to_string(),
                        Variable::Bool(b) => b.to_string(),
                        Variable::Null => "null".to_string(),
                        _ => continue,
                    }
                } else {
                    "null".to_string()
                }
            } else {
                continue;
            };
            groups.entry(key).or_default().push(item.clone());
        }

        let result: std::collections::BTreeMap<String, Rcvar> = groups
            .into_iter()
            .map(|(k, v)| (k, Rc::new(Variable::Array(v)) as Rcvar))
            .collect();

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// nth(array, n) -> array (every nth element)
// =============================================================================

define_function!(NthFn, vec![ArgumentType::Array, ArgumentType::Number], None);

impl Function for NthFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let n = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })? as usize;

        if n == 0 {
            return Ok(Rc::new(Variable::Null));
        }

        let result: Vec<Rcvar> = arr.iter().step_by(n).cloned().collect();
        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// interleave(array1, array2) -> array (alternate elements)
// =============================================================================

define_function!(
    InterleaveFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for InterleaveFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let arr2 = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut result = Vec::with_capacity(arr1.len() + arr2.len());
        let mut iter1 = arr1.iter();
        let mut iter2 = arr2.iter();

        loop {
            match (iter1.next(), iter2.next()) {
                (Some(a), Some(b)) => {
                    result.push(a.clone());
                    result.push(b.clone());
                }
                (Some(a), None) => {
                    result.push(a.clone());
                    result.extend(iter1.cloned());
                    break;
                }
                (None, Some(b)) => {
                    result.push(b.clone());
                    result.extend(iter2.cloned());
                    break;
                }
                (None, None) => break,
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// rotate(array, n) -> array (rotate elements by n positions)
// =============================================================================

define_function!(
    RotateFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for RotateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        if arr.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let n = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })? as i64;

        let len = arr.len() as i64;
        let rotation = ((n % len) + len) % len;
        let rotation = rotation as usize;

        let mut result = Vec::with_capacity(arr.len());
        result.extend(arr[rotation..].iter().cloned());
        result.extend(arr[..rotation].iter().cloned());

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// partition(array, n) -> array (split into n equal parts)
// =============================================================================

define_function!(
    PartitionFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for PartitionFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let n = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })? as usize;

        if n == 0 {
            return Ok(Rc::new(Variable::Null));
        }

        let len = arr.len();
        let base_size = len / n;
        let remainder = len % n;

        let mut result = Vec::with_capacity(n);
        let mut start = 0;

        for i in 0..n {
            let size = base_size + if i < remainder { 1 } else { 0 };
            if size > 0 {
                result.push(Rc::new(Variable::Array(arr[start..start + size].to_vec())) as Rcvar);
            } else {
                result.push(Rc::new(Variable::Array(vec![])) as Rcvar);
            }
            start += size;
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// difference(arr1, arr2) -> array (set difference)
// =============================================================================

define_function!(
    DifferenceFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for DifferenceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let arr2 = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let set2: HashSet<String> = arr2
            .iter()
            .map(|v| serde_json::to_string(&**v).unwrap_or_default())
            .collect();

        let result: Vec<Rcvar> = arr1
            .iter()
            .filter(|v| {
                let key = serde_json::to_string(&***v).unwrap_or_default();
                !set2.contains(&key)
            })
            .cloned()
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// intersection(arr1, arr2) -> array (set intersection)
// =============================================================================

define_function!(
    IntersectionFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for IntersectionFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let arr2 = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let set2: HashSet<String> = arr2
            .iter()
            .map(|v| serde_json::to_string(&**v).unwrap_or_default())
            .collect();

        let mut seen: HashSet<String> = HashSet::new();
        let result: Vec<Rcvar> = arr1
            .iter()
            .filter(|v| {
                let key = serde_json::to_string(&***v).unwrap_or_default();
                set2.contains(&key) && seen.insert(key)
            })
            .cloned()
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// union(arr1, arr2) -> array (set union)
// =============================================================================

define_function!(
    UnionFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for UnionFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let arr2 = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut seen: HashSet<String> = HashSet::new();
        let mut result: Vec<Rcvar> = Vec::new();

        for item in arr1.iter().chain(arr2.iter()) {
            let key = serde_json::to_string(&**item).unwrap_or_default();
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// frequencies(array) -> object (count occurrences)
// =============================================================================

define_function!(FrequenciesFn, vec![ArgumentType::Array], None);

impl Function for FrequenciesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for item in arr {
            let key = match &**item {
                Variable::String(s) => s.clone(),
                Variable::Number(n) => n.to_string(),
                Variable::Bool(b) => b.to_string(),
                Variable::Null => "null".to_string(),
                _ => serde_json::to_string(&**item).unwrap_or_else(|_| "null".to_string()),
            };
            *counts.entry(key).or_insert(0) += 1;
        }

        let result: std::collections::BTreeMap<String, Rcvar> = counts
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    Rc::new(Variable::Number(serde_json::Number::from(v))) as Rcvar,
                )
            })
            .collect();

        Ok(Rc::new(Variable::Object(result)))
    }
}

// =============================================================================
// mode(array) -> any (most frequent value)
// =============================================================================

define_function!(ModeFn, vec![ArgumentType::Array], None);

impl Function for ModeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        if arr.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        let mut counts: std::collections::HashMap<String, (i64, Rcvar)> =
            std::collections::HashMap::new();

        for item in arr {
            let key = serde_json::to_string(&**item).unwrap_or_default();
            counts
                .entry(key)
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, item.clone()));
        }

        let (_, (_, mode_value)) = counts
            .into_iter()
            .max_by_key(|(_, (count, _))| *count)
            .unwrap();

        Ok(mode_value)
    }
}

// =============================================================================
// cartesian(arr1, arr2) -> array (cartesian product)
// =============================================================================

define_function!(
    CartesianFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for CartesianFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr1 = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let arr2 = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut result = Vec::with_capacity(arr1.len() * arr2.len());

        for a in arr1 {
            for b in arr2 {
                result.push(Rc::new(Variable::Array(vec![a.clone(), b.clone()])) as Rcvar);
            }
        }

        Ok(Rc::new(Variable::Array(result)))
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
    fn test_unique() {
        let runtime = setup_runtime();
        let expr = runtime.compile("unique(@)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(1))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_first() {
        let runtime = setup_runtime();
        let expr = runtime.compile("first(@)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        ]);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 1);
    }

    #[test]
    fn test_last() {
        let runtime = setup_runtime();
        let expr = runtime.compile("last(@)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        ]);
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_range() {
        let runtime = setup_runtime();
        let expr = runtime.compile("range(`0`, `5`)").unwrap();
        let data = Variable::Null;
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
    }
}
