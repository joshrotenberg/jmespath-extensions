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
//! | [`flatten`](#flatten) | `flatten(array) → array` | Single-level flatten |
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
//! | [`initial`](#initial) | `initial(array) → array` | All but last element |
//! | [`tail`](#tail) | `tail(array) → array` | All but first element |
//! | [`without`](#without) | `without(array, ...values) → array` | Remove specified values |
//! | [`xor`](#xor) | `xor(array1, array2) → array` | Symmetric difference |
//! | [`fill`](#fill) | `fill(array, value, start?, end?) → array` | Fill with value |
//! | [`pull_at`](#pull_at) | `pull_at(array, indices) → array` | Get elements at indices |
//! | [`window`](#window) | `window(array, size, step?) → array` | Sliding window |
//! | [`combinations`](#combinations) | `combinations(array, k) → array` | K-combinations |
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
//! ## flatten
//!
//! Flattens an array one level deep. Nested arrays are merged into the parent,
//! but deeper nesting is preserved.
//!
//! ```text
//! flatten(array) → array
//!
//! flatten([[1, 2], [3, 4]])          → [1, 2, 3, 4]
//! flatten([1, [2, [3, 4]]])          → [1, 2, [3, 4]]
//! flatten([1, 2, 3])                 → [1, 2, 3]
//! flatten([[1], [[2]], [[[3]]]])     → [1, [2], [[3]]]
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
    runtime.register_function("flatten", Box::new(FlattenFn::new()));
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
    runtime.register_function("initial", Box::new(InitialFn::new()));
    runtime.register_function("tail", Box::new(TailFn::new()));
    runtime.register_function("without", Box::new(WithoutFn::new()));
    runtime.register_function("xor", Box::new(XorFn::new()));
    runtime.register_function("fill", Box::new(FillFn::new()));
    runtime.register_function("pull_at", Box::new(PullAtFn::new()));
    runtime.register_function("window", Box::new(WindowFn::new()));
    runtime.register_function("combinations", Box::new(CombinationsFn::new()));
    runtime.register_function("transpose", Box::new(TransposeFn::new()));
    runtime.register_function("pairwise", Box::new(PairwiseFn::new()));
    // Alias for window (sliding_window is a common name)
    runtime.register_function("sliding_window", Box::new(WindowFn::new()));
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
// flatten(array) -> array (single-level flatten)
// =============================================================================

define_function!(FlattenFn, vec![ArgumentType::Array], None);

impl Function for FlattenFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut result = Vec::new();
        for item in arr {
            if let Some(inner) = item.as_array() {
                result.extend(inner.iter().cloned());
            } else {
                result.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(result)))
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

// =============================================================================
// initial(array) -> array (all elements except the last)
// =============================================================================

define_function!(InitialFn, vec![ArgumentType::Array], None);

impl Function for InitialFn {
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

        let result: Vec<Rcvar> = arr[..arr.len() - 1].to_vec();
        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// tail(array) -> array (all elements except the first)
// =============================================================================

define_function!(TailFn, vec![ArgumentType::Array], None);

impl Function for TailFn {
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

        let result: Vec<Rcvar> = arr[1..].to_vec();
        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// without(array, values_array) -> array (remove specified values)
// =============================================================================

define_function!(
    WithoutFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for WithoutFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let exclude = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument for values to exclude".to_owned()),
            )
        })?;

        // Create a set of serialized values to exclude for efficient lookup
        let exclude_set: HashSet<String> = exclude
            .iter()
            .map(|v| serde_json::to_string(&**v).unwrap_or_default())
            .collect();

        let result: Vec<Rcvar> = arr
            .iter()
            .filter(|item| {
                let key = serde_json::to_string(&***item).unwrap_or_default();
                !exclude_set.contains(&key)
            })
            .cloned()
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// xor(array1, array2) -> array (symmetric difference)
// =============================================================================

define_function!(XorFn, vec![ArgumentType::Array, ArgumentType::Array], None);

impl Function for XorFn {
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

        // Create sets of serialized values
        let set1: HashSet<String> = arr1
            .iter()
            .map(|v| serde_json::to_string(&**v).unwrap_or_default())
            .collect();

        let set2: HashSet<String> = arr2
            .iter()
            .map(|v| serde_json::to_string(&**v).unwrap_or_default())
            .collect();

        let mut result = Vec::new();

        // Add elements from arr1 that are not in arr2
        for item in arr1 {
            let key = serde_json::to_string(&**item).unwrap_or_default();
            if !set2.contains(&key) {
                result.push(item.clone());
            }
        }

        // Add elements from arr2 that are not in arr1
        for item in arr2 {
            let key = serde_json::to_string(&**item).unwrap_or_default();
            if !set1.contains(&key) {
                result.push(item.clone());
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// window(array, size, step?) -> array (sliding window)
// =============================================================================

define_function!(
    WindowFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    Some(ArgumentType::Number)
);

impl Function for WindowFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let size = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for window size".to_owned()),
            )
        })? as usize;

        if size == 0 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        // Default step is 1
        let step = if args.len() > 2 {
            args[2].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for step".to_owned()),
                )
            })? as usize
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

        let len = arr.len();
        if len < size {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i + size <= len {
            let window: Vec<Rcvar> = arr[i..i + size].to_vec();
            result.push(Rc::new(Variable::Array(window)) as Rcvar);
            i += step;
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// combinations(array, k) -> array (k-combinations of array)
// =============================================================================

define_function!(
    CombinationsFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

fn generate_combinations(arr: &[Rcvar], k: usize) -> Vec<Vec<Rcvar>> {
    if k == 0 {
        return vec![vec![]];
    }
    if arr.len() < k {
        return vec![];
    }

    let mut result = Vec::new();

    // Include first element in combination
    let first = arr[0].clone();
    let rest = &arr[1..];
    for mut combo in generate_combinations(rest, k - 1) {
        let mut new_combo = vec![first.clone()];
        new_combo.append(&mut combo);
        result.push(new_combo);
    }

    // Exclude first element
    result.extend(generate_combinations(rest, k));

    result
}

impl Function for CombinationsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let k = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for k".to_owned()),
            )
        })? as usize;

        // Limit to prevent excessive computation
        const MAX_COMBINATIONS: usize = 10000;

        // Quick check: if C(n, k) would be too large, return error
        let n = arr.len();
        if n > 20 && k > 3 && k < n - 3 {
            // For large arrays with mid-range k, combinations could be huge
            // Rough estimate: C(n, k) grows quickly
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Combination size too large".to_owned()),
            ));
        }

        let combinations = generate_combinations(arr, k);

        if combinations.len() > MAX_COMBINATIONS {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Too many combinations generated".to_owned()),
            ));
        }

        let result: Vec<Rcvar> = combinations
            .into_iter()
            .map(|combo| Rc::new(Variable::Array(combo)) as Rcvar)
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// fill(array, value, start?, end?) -> array (fill range with value)
// =============================================================================

define_function!(
    FillFn,
    vec![ArgumentType::Array, ArgumentType::Any],
    Some(ArgumentType::Number)
);

impl Function for FillFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let fill_value = args[1].clone();

        let len = arr.len();
        if len == 0 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        // Default start is 0, default end is array length
        let start = if args.len() > 2 {
            let s = args[2].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for start index".to_owned()),
                )
            })? as i64;
            // Handle negative indices
            if s < 0 {
                (len as i64 + s).max(0) as usize
            } else {
                (s as usize).min(len)
            }
        } else {
            0
        };

        let end = if args.len() > 3 {
            let e = args[3].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for end index".to_owned()),
                )
            })? as i64;
            // Handle negative indices
            if e < 0 {
                (len as i64 + e).max(0) as usize
            } else {
                (e as usize).min(len)
            }
        } else {
            len
        };

        let mut result: Vec<Rcvar> = arr.clone();

        for item in result.iter_mut().take(end.min(len)).skip(start) {
            *item = fill_value.clone();
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// pull_at(array, indices_array) -> array (get elements at specified indices)
// =============================================================================

define_function!(
    PullAtFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for PullAtFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let indices = args[1].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array of indices".to_owned()),
            )
        })?;

        let len = arr.len();
        let mut result = Vec::new();

        for idx_var in indices {
            let idx = idx_var.as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number in indices array".to_owned()),
                )
            })? as i64;

            // Handle negative indices
            let actual_idx = if idx < 0 {
                (len as i64 + idx).max(0) as usize
            } else {
                idx as usize
            };

            if actual_idx < len {
                result.push(arr[actual_idx].clone());
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// transpose(array) -> array
// =============================================================================

// Transpose a 2D array (swap rows and columns).
//
// # Arguments
// * `array` - A 2D array (array of arrays)
//
// # Returns
// A new 2D array with rows and columns swapped.
// The result has as many rows as the shortest inner array.
//
// # Example
// transpose([[1, 2, 3], [4, 5, 6]]) -> [[1, 4], [2, 5], [3, 6]]
// transpose([[1, 2], [3, 4], [5, 6]]) -> [[1, 3, 5], [2, 4, 6]]
define_function!(TransposeFn, vec![ArgumentType::Array], None);

impl Function for TransposeFn {
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

        // Get all inner arrays and find the minimum length
        let mut inner_arrays: Vec<&Vec<Rcvar>> = Vec::new();
        let mut min_len = usize::MAX;

        for item in arr {
            if let Some(inner) = item.as_array() {
                min_len = min_len.min(inner.len());
                inner_arrays.push(inner);
            } else {
                // If any element is not an array, return empty
                return Ok(Rc::new(Variable::Array(vec![])));
            }
        }

        if inner_arrays.is_empty() || min_len == 0 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        // Transpose: create new arrays where each contains the i-th element from each inner array
        let mut result = Vec::with_capacity(min_len);
        for i in 0..min_len {
            let mut row = Vec::with_capacity(inner_arrays.len());
            for inner in &inner_arrays {
                row.push(inner[i].clone());
            }
            result.push(Rc::new(Variable::Array(row)));
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// pairwise(array) -> array
// =============================================================================

// Return adjacent pairs from an array.
//
// This is equivalent to `window(array, 2)` but provided as a convenience.
//
// # Arguments
// * `array` - The input array
//
// # Returns
// An array of 2-element arrays, each containing adjacent elements.
//
// # Example
// pairwise([1, 2, 3, 4]) -> [[1, 2], [2, 3], [3, 4]]
// pairwise([10, 15, 13, 20]) -> [[10, 15], [15, 13], [13, 20]]
define_function!(PairwiseFn, vec![ArgumentType::Array], None);

impl Function for PairwiseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        if arr.len() < 2 {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let mut result = Vec::with_capacity(arr.len() - 1);
        for i in 0..arr.len() - 1 {
            let pair = vec![arr[i].clone(), arr[i + 1].clone()];
            result.push(Rc::new(Variable::Array(pair)));
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

    #[test]
    fn test_initial() {
        let runtime = setup_runtime();
        let expr = runtime.compile("initial(@)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[1].as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_initial_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("initial(@)").unwrap();
        let data = Variable::Array(vec![]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_tail() {
        let runtime = setup_runtime();
        let expr = runtime.compile("tail(@)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number().unwrap() as i64, 2);
        assert_eq!(arr[1].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_tail_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("tail(@)").unwrap();
        let data = Variable::Array(vec![]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_without() {
        let runtime = setup_runtime();
        let expr = runtime.compile("without(@, `[2, 4]`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
            Rc::new(Variable::Number(serde_json::Number::from(4))),
            Rc::new(Variable::Number(serde_json::Number::from(5))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[1].as_number().unwrap() as i64, 3);
        assert_eq!(arr[2].as_number().unwrap() as i64, 5);
    }

    #[test]
    fn test_xor() {
        let runtime = setup_runtime();
        let expr = runtime.compile("xor(`[1, 2, 3]`, `[2, 3, 4]`)").unwrap();
        let data = Variable::Null;
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[1].as_number().unwrap() as i64, 4);
    }

    #[test]
    fn test_fill() {
        let runtime = setup_runtime();
        let expr = runtime.compile("fill(@, `0`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap() as i64, 0);
        assert_eq!(arr[1].as_number().unwrap() as i64, 0);
        assert_eq!(arr[2].as_number().unwrap() as i64, 0);
    }

    #[test]
    fn test_fill_with_range() {
        let runtime = setup_runtime();
        let expr = runtime.compile("fill(@, `0`, `1`, `3`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
            Rc::new(Variable::Number(serde_json::Number::from(4))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[1].as_number().unwrap() as i64, 0);
        assert_eq!(arr[2].as_number().unwrap() as i64, 0);
        assert_eq!(arr[3].as_number().unwrap() as i64, 4);
    }

    #[test]
    fn test_pull_at() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pull_at(@, `[0, 2]`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::String("a".to_string())),
            Rc::new(Variable::String("b".to_string())),
            Rc::new(Variable::String("c".to_string())),
            Rc::new(Variable::String("d".to_string())),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "a");
        assert_eq!(arr[1].as_string().unwrap(), "c");
    }

    #[test]
    fn test_pull_at_negative_index() {
        let runtime = setup_runtime();
        let expr = runtime.compile("pull_at(@, `[-1, -2]`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::String("a".to_string())),
            Rc::new(Variable::String("b".to_string())),
            Rc::new(Variable::String("c".to_string())),
            Rc::new(Variable::String("d".to_string())),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_string().unwrap(), "d");
        assert_eq!(arr[1].as_string().unwrap(), "c");
    }

    #[test]
    fn test_window() {
        let runtime = setup_runtime();
        let expr = runtime.compile("window(@, `3`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
            Rc::new(Variable::Number(serde_json::Number::from(4))),
            Rc::new(Variable::Number(serde_json::Number::from(5))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // [1,2,3], [2,3,4], [3,4,5]
        assert_eq!(arr.len(), 3);
        let first = arr[0].as_array().unwrap();
        assert_eq!(first.len(), 3);
        assert_eq!(first[0].as_number().unwrap() as i64, 1);
        assert_eq!(first[1].as_number().unwrap() as i64, 2);
        assert_eq!(first[2].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_window_with_step() {
        let runtime = setup_runtime();
        let expr = runtime.compile("window(@, `2`, `2`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
            Rc::new(Variable::Number(serde_json::Number::from(4))),
            Rc::new(Variable::Number(serde_json::Number::from(5))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // [1,2], [3,4]
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_window_empty_result() {
        let runtime = setup_runtime();
        let expr = runtime.compile("window(@, `5`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_combinations() {
        let runtime = setup_runtime();
        let expr = runtime.compile("combinations(@, `2`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // C(3,2) = 3: [1,2], [1,3], [2,3]
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_combinations_k_zero() {
        let runtime = setup_runtime();
        let expr = runtime.compile("combinations(@, `0`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // C(n,0) = 1 (the empty set)
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_combinations_k_equals_n() {
        let runtime = setup_runtime();
        let expr = runtime.compile("combinations(@, `3`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // C(3,3) = 1: [1,2,3]
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_combinations_k_greater_than_n() {
        let runtime = setup_runtime();
        let expr = runtime.compile("combinations(@, `5`)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // C(2,5) = 0
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // zip tests
    // =========================================================================

    #[test]
    fn test_zip_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2, 3], "b": ["x", "y", "z"]}"#).unwrap();
        let expr = runtime.compile("zip(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_array().unwrap()[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[0].as_array().unwrap()[1].as_string().unwrap(), "x");
    }

    #[test]
    fn test_zip_unequal_lengths() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2], "b": ["x", "y", "z"]}"#).unwrap();
        let expr = runtime.compile("zip(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Stops at shorter array
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_zip_empty_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [], "b": [1, 2, 3]}"#).unwrap();
        let expr = runtime.compile("zip(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_zip_with_objects() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"{"names": ["Alice", "Bob"], "scores": [95, 87]}"#).unwrap();
        let expr = runtime.compile("zip(names, scores)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_array().unwrap()[0].as_string().unwrap(), "Alice");
        assert_eq!(
            arr[0].as_array().unwrap()[1].as_number().unwrap() as i64,
            95
        );
    }

    // =========================================================================
    // chunk tests
    // =========================================================================

    #[test]
    fn test_chunk_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("chunk(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // [1,2], [3,4], [5]
        assert_eq!(arr[0].as_array().unwrap().len(), 2);
        assert_eq!(arr[2].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_chunk_exact_fit() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6]"#).unwrap();
        let expr = runtime.compile("chunk(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_array().unwrap().len(), 3);
        assert_eq!(arr[1].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_chunk_size_larger_than_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("chunk(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_chunk_size_one() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("chunk(@, `1`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_chunk_and_process_pipeline() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#).unwrap();
        let expr = runtime.compile("chunk(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // [1,2,3], [4,5,6], [7,8,9], [10]
        assert_eq!(arr.len(), 4);
    }

    // =========================================================================
    // take tests
    // =========================================================================

    #[test]
    fn test_take_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("take(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[2].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_take_more_than_length() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2]"#).unwrap();
        let expr = runtime.compile("take(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_take_zero() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("take(@, `0`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // drop tests
    // =========================================================================

    #[test]
    fn test_drop_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("drop(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_drop_more_than_length() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2]"#).unwrap();
        let expr = runtime.compile("drop(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_drop_zero() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("drop(@, `0`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    // =========================================================================
    // flatten_deep tests
    // =========================================================================

    #[test]
    fn test_flatten_deep_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, 2], [3, 4]]"#).unwrap();
        let expr = runtime.compile("flatten_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4);
    }

    #[test]
    fn test_flatten_deep_nested() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, [2, [3, [4, [5]]]]]"#).unwrap();
        let expr = runtime.compile("flatten_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[4].as_number().unwrap() as i64, 5);
    }

    #[test]
    fn test_flatten_deep_already_flat() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("flatten_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_flatten_deep_mixed() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, [2, 3], [[4]], [[[5, 6]]]]"#).unwrap();
        let expr = runtime.compile("flatten_deep(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 6);
    }

    // =========================================================================
    // flatten tests (single-level)
    // =========================================================================

    #[test]
    fn test_flatten_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, 2], [3, 4]]"#).unwrap();
        let expr = runtime.compile("flatten(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4);
    }

    #[test]
    fn test_flatten_single_level_only() {
        let runtime = setup_runtime();
        // flatten should only go one level deep
        let data = Variable::from_json(r#"[1, [2, [3, 4]]]"#).unwrap();
        let expr = runtime.compile("flatten(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should be [1, 2, [3, 4]] - 3 elements, not 4
        assert_eq!(arr.len(), 3);
        // The third element should still be an array
        assert!(arr[2].as_array().is_some());
    }

    #[test]
    fn test_flatten_already_flat() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("flatten(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_flatten_mixed_nesting() {
        let runtime = setup_runtime();
        // [[1], [[2]], [[[3]]]] should become [1, [2], [[3]]]
        let data = Variable::from_json(r#"[[1], [[2]], [[[3]]]]"#).unwrap();
        let expr = runtime.compile("flatten(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // First element is a number
        assert!(arr[0].as_number().is_some());
        // Second element is [2]
        assert!(arr[1].as_array().is_some());
        // Third element is [[3]]
        assert!(arr[2].as_array().is_some());
    }

    #[test]
    fn test_flatten_empty() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("flatten(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // compact tests
    // =========================================================================

    #[test]
    fn test_compact_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, null, 2, false, 3]"#).unwrap();
        let expr = runtime.compile("compact(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_compact_keeps_zero_and_empty_string() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[0, "", null, true]"#).unwrap();
        let expr = runtime.compile("compact(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 0, "", true
    }

    #[test]
    fn test_compact_all_falsy() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[null, false, null]"#).unwrap();
        let expr = runtime.compile("compact(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // index_at tests
    // =========================================================================

    #[test]
    fn test_index_at_positive() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "c", "d"]"#).unwrap();
        let expr = runtime.compile("index_at(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "c");
    }

    #[test]
    fn test_index_at_negative() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "c", "d"]"#).unwrap();
        let expr = runtime.compile("index_at(@, `-1`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "d");
    }

    #[test]
    fn test_index_at_negative_second() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "c", "d"]"#).unwrap();
        let expr = runtime.compile("index_at(@, `-2`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "c");
    }

    #[test]
    fn test_index_at_out_of_bounds() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "c"]"#).unwrap();
        let expr = runtime.compile("index_at(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    // =========================================================================
    // includes tests
    // =========================================================================

    #[test]
    fn test_includes_number() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("includes(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_includes_not_found() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3]"#).unwrap();
        let expr = runtime.compile("includes(@, `10`)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_includes_string() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["apple", "banana", "cherry"]"#).unwrap();
        let expr = runtime.compile(r#"includes(@, `"banana"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_includes_object() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
        let expr = runtime.compile(r#"includes(@, `{"a": 1}`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    // =========================================================================
    // find_index tests
    // =========================================================================

    #[test]
    fn test_find_index_found() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "c", "d"]"#).unwrap();
        let expr = runtime.compile(r#"find_index(@, `"c"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_find_index_not_found() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "c"]"#).unwrap();
        let expr = runtime.compile(r#"find_index(@, `"z"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, -1);
    }

    // =========================================================================
    // group_by tests
    // =========================================================================

    #[test]
    fn test_group_by_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(
            r#"[{"type": "a", "v": 1}, {"type": "b", "v": 2}, {"type": "a", "v": 3}]"#,
        )
        .unwrap();
        let expr = runtime.compile(r#"group_by(@, `"type"`)"#).unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(obj.get("b").unwrap().as_array().unwrap().len(), 1);
    }

    // =========================================================================
    // nth tests
    // =========================================================================

    #[test]
    fn test_nth_every_second() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6]"#).unwrap();
        let expr = runtime.compile("nth(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 1, 3, 5
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[1].as_number().unwrap() as i64, 3);
        assert_eq!(arr[2].as_number().unwrap() as i64, 5);
    }

    #[test]
    fn test_nth_every_third() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9]"#).unwrap();
        let expr = runtime.compile("nth(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 1, 4, 7
    }

    // =========================================================================
    // interleave tests
    // =========================================================================

    #[test]
    fn test_interleave_equal() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2, 3], "b": ["a", "b", "c"]}"#).unwrap();
        let expr = runtime.compile("interleave(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 6);
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[1].as_string().unwrap(), "a");
        assert_eq!(arr[2].as_number().unwrap() as i64, 2);
        assert_eq!(arr[3].as_string().unwrap(), "b");
    }

    #[test]
    fn test_interleave_unequal() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2], "b": ["a", "b", "c"]}"#).unwrap();
        let expr = runtime.compile("interleave(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5); // 1, a, 2, b, c
    }

    // =========================================================================
    // rotate tests
    // =========================================================================

    #[test]
    fn test_rotate_left() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("rotate(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_number().unwrap() as i64, 3);
        assert_eq!(arr[4].as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_rotate_right() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("rotate(@, `-1`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr[0].as_number().unwrap() as i64, 5);
        assert_eq!(arr[1].as_number().unwrap() as i64, 1);
    }

    // =========================================================================
    // partition tests
    // =========================================================================

    #[test]
    fn test_partition_even() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6]"#).unwrap();
        let expr = runtime.compile("partition(@, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_array().unwrap().len(), 3);
        assert_eq!(arr[1].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_partition_uneven() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("partition(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    // =========================================================================
    // set operations tests
    // =========================================================================

    #[test]
    fn test_difference() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2, 3, 4], "b": [2, 4]}"#).unwrap();
        let expr = runtime.compile("difference(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // 1, 3
    }

    #[test]
    fn test_intersection() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2, 3], "b": [2, 3, 4]}"#).unwrap();
        let expr = runtime.compile("intersection(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // 2, 3
    }

    #[test]
    fn test_union() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2], "b": [2, 3]}"#).unwrap();
        let expr = runtime.compile("union(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3); // 1, 2, 3
    }

    // =========================================================================
    // frequencies tests
    // =========================================================================

    #[test]
    fn test_frequencies_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["a", "b", "a", "c", "a", "b"]"#).unwrap();
        let expr = runtime.compile("frequencies(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_number().unwrap() as i64, 3);
        assert_eq!(obj.get("b").unwrap().as_number().unwrap() as i64, 2);
        assert_eq!(obj.get("c").unwrap().as_number().unwrap() as i64, 1);
    }

    #[test]
    fn test_frequencies_numbers() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 1, 1, 2, 3]"#).unwrap();
        let expr = runtime.compile("frequencies(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("1").unwrap().as_number().unwrap() as i64, 3);
        assert_eq!(obj.get("2").unwrap().as_number().unwrap() as i64, 2);
    }

    // =========================================================================
    // mode tests
    // =========================================================================

    #[test]
    fn test_mode_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 2, 3, 2, 4]"#).unwrap();
        let expr = runtime.compile("mode(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_mode_empty() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("mode(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    // =========================================================================
    // cartesian tests
    // =========================================================================

    #[test]
    fn test_cartesian_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [1, 2], "b": ["x", "y"]}"#).unwrap();
        let expr = runtime.compile("cartesian(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4); // [1,x], [1,y], [2,x], [2,y]
    }

    #[test]
    fn test_cartesian_empty() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"{"a": [], "b": [1, 2]}"#).unwrap();
        let expr = runtime.compile("cartesian(a, b)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_first_empty_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("first(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_last_empty_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("last(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_unique_preserves_order() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"["c", "a", "b", "a", "c"]"#).unwrap();
        let expr = runtime.compile("unique(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "c");
        assert_eq!(arr[1].as_string().unwrap(), "a");
        assert_eq!(arr[2].as_string().unwrap(), "b");
    }

    #[test]
    fn test_unique_different_types() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, "1", 1, "1"]"#).unwrap();
        let expr = runtime.compile("unique(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2); // 1 and "1" are different
    }

    #[test]
    fn test_range_with_step() {
        let runtime = setup_runtime();
        let data = Variable::Null;
        let expr = runtime.compile("range(`1`, `10`, `2`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5); // 1, 3, 5, 7, 9
        assert_eq!(arr[0].as_number().unwrap() as i64, 1);
        assert_eq!(arr[4].as_number().unwrap() as i64, 9);
    }

    #[test]
    fn test_range_descending() {
        let runtime = setup_runtime();
        let data = Variable::Null;
        let expr = runtime.compile("range(`5`, `0`, `-1`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5); // 5, 4, 3, 2, 1
        assert_eq!(arr[0].as_number().unwrap() as i64, 5);
        assert_eq!(arr[4].as_number().unwrap() as i64, 1);
    }

    // =========================================================================
    // Pipeline patterns with arrays
    // =========================================================================

    #[test]
    fn test_pipeline_unique_sort() {
        let runtime = setup_runtime();
        let data =
            Variable::from_json(r#"["redis", "database", "redis", "nosql", "database"]"#).unwrap();
        let expr = runtime.compile("unique(@) | sort(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_string().unwrap(), "database");
        assert_eq!(arr[1].as_string().unwrap(), "nosql");
        assert_eq!(arr[2].as_string().unwrap(), "redis");
    }

    #[test]
    fn test_pipeline_filter_take() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#).unwrap();
        let expr = runtime.compile("[?@ > `3`] | take(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_number().unwrap() as i64, 4);
        assert_eq!(arr[1].as_number().unwrap() as i64, 5);
        assert_eq!(arr[2].as_number().unwrap() as i64, 6);
    }

    #[test]
    fn test_pipeline_flatten_unique() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, 2], [2, 3], [3, 4]]"#).unwrap();
        let expr = runtime.compile("flatten_deep(@) | unique(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4); // 1, 2, 3, 4
    }

    #[test]
    fn test_large_array_processing() {
        let runtime = setup_runtime();
        // Create array with 1000 elements
        let items: Vec<i32> = (1..=1000).collect();
        let json = serde_json::to_string(&items).unwrap();
        let data = Variable::from_json(&json).unwrap();

        let expr = runtime.compile("length(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 1000);
    }

    #[test]
    fn test_transpose_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, 2, 3], [4, 5, 6]]"#).unwrap();
        let expr = runtime.compile("transpose(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // First column: [1, 4]
        let col0 = arr[0].as_array().unwrap();
        assert_eq!(col0[0].as_number().unwrap() as i64, 1);
        assert_eq!(col0[1].as_number().unwrap() as i64, 4);
        // Second column: [2, 5]
        let col1 = arr[1].as_array().unwrap();
        assert_eq!(col1[0].as_number().unwrap() as i64, 2);
        assert_eq!(col1[1].as_number().unwrap() as i64, 5);
    }

    #[test]
    fn test_transpose_empty() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let expr = runtime.compile("transpose(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_transpose_unequal_rows() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[[1, 2], [3, 4, 5], [6, 7]]"#).unwrap();
        let expr = runtime.compile("transpose(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        // Should use minimum length (2)
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_pairwise_basic() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4]"#).unwrap();
        let expr = runtime.compile("pairwise(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // First pair: [1, 2]
        let pair0 = arr[0].as_array().unwrap();
        assert_eq!(pair0[0].as_number().unwrap() as i64, 1);
        assert_eq!(pair0[1].as_number().unwrap() as i64, 2);
        // Second pair: [2, 3]
        let pair1 = arr[1].as_array().unwrap();
        assert_eq!(pair1[0].as_number().unwrap() as i64, 2);
        assert_eq!(pair1[1].as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_pairwise_short_array() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1]"#).unwrap();
        let expr = runtime.compile("pairwise(@)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_sliding_window_alias() {
        let runtime = setup_runtime();
        let data = Variable::from_json(r#"[1, 2, 3, 4, 5]"#).unwrap();
        let expr = runtime.compile("sliding_window(@, `3`)").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // First window: [1, 2, 3]
        let win0 = arr[0].as_array().unwrap();
        assert_eq!(win0.len(), 3);
        assert_eq!(win0[0].as_number().unwrap() as i64, 1);
    }
}
