//! Random value generation functions.
//!
//! This module provides random functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category random`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::random;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! random::register(&mut runtime);
//! ```

use std::rc::Rc;

#[cfg(feature = "rand")]
use crate::common::ErrorReason;
use crate::common::{Context, Function, JmespathError, Rcvar, Runtime, Variable};

#[cfg(feature = "uuid")]
use crate::define_function;

/// Register all random functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    #[cfg(feature = "rand")]
    {
        runtime.register_function("random", Box::new(RandomFn::new()));
        runtime.register_function("shuffle", Box::new(ShuffleFn::new()));
        runtime.register_function("sample", Box::new(SampleFn::new()));
    }
    #[cfg(feature = "uuid")]
    {
        runtime.register_function("uuid", Box::new(UuidFn::new()));
    }
}

// =============================================================================
// random() -> number (0.0 to 1.0)
// random(min, max) -> number in range [min, max)
// =============================================================================

#[cfg(feature = "rand")]
pub struct RandomFn;

#[cfg(feature = "rand")]
impl Default for RandomFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rand")]
impl RandomFn {
    pub fn new() -> RandomFn {
        RandomFn
    }
}

#[cfg(feature = "rand")]
impl Function for RandomFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        use rand::Rng;

        // Manual validation: accept 0 or 2 arguments
        if !args.is_empty() && args.len() != 2 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("random() takes 0 or 2 arguments".to_owned()),
            ));
        }

        let mut rng = rand::thread_rng();

        let value: f64 = if args.is_empty() {
            // random() - return 0.0 to 1.0
            rng.gen_range(0.0..1.0)
        } else {
            // random(min, max) - return min to max
            let min = args[0].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for min".to_owned()),
                )
            })?;
            let max = args[1].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for max".to_owned()),
                )
            })?;
            rng.gen_range(min..max)
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// shuffle(array) -> array (randomly shuffled)
// shuffle(array, seed) -> array (deterministically shuffled)
// =============================================================================

#[cfg(feature = "rand")]
pub struct ShuffleFn;

#[cfg(feature = "rand")]
impl Default for ShuffleFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rand")]
impl ShuffleFn {
    pub fn new() -> ShuffleFn {
        ShuffleFn
    }
}

#[cfg(feature = "rand")]
impl Function for ShuffleFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        // Manual validation: 1 or 2 arguments
        if args.is_empty() || args.len() > 2 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("shuffle() takes 1 or 2 arguments".to_owned()),
            ));
        }

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        let mut result: Vec<Rcvar> = arr.clone();

        if args.len() == 2 {
            // Deterministic shuffle with seed
            let seed = args[1].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for seed".to_owned()),
                )
            })? as u64;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            result.shuffle(&mut rng);
        } else {
            // Random shuffle
            result.shuffle(&mut rand::thread_rng());
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// sample(array, n) -> array (random sample of n elements)
// sample(array, n, seed) -> array (deterministic sample)
// =============================================================================

#[cfg(feature = "rand")]
pub struct SampleFn;

#[cfg(feature = "rand")]
impl Default for SampleFn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rand")]
impl SampleFn {
    pub fn new() -> SampleFn {
        SampleFn
    }
}

#[cfg(feature = "rand")]
impl Function for SampleFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        // Manual validation: 2 or 3 arguments
        if args.len() < 2 || args.len() > 3 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("sample() takes 2 or 3 arguments".to_owned()),
            ));
        }

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

        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        let sample: Vec<Rcvar> = if args.len() == 3 {
            // Deterministic sample with seed
            let seed = args[2].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected number for seed".to_owned()),
                )
            })? as u64;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            arr.choose_multiple(&mut rng, n.min(arr.len()))
                .cloned()
                .collect()
        } else {
            // Random sample
            arr.choose_multiple(&mut rand::thread_rng(), n.min(arr.len()))
                .cloned()
                .collect()
        };

        Ok(Rc::new(Variable::Array(sample)))
    }
}

// =============================================================================
// uuid() -> string (UUID v4)
// =============================================================================

#[cfg(feature = "uuid")]
define_function!(UuidFn, vec![], None);

#[cfg(feature = "uuid")]
impl Function for UuidFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let id = uuid::Uuid::new_v4();
        Ok(Rc::new(Variable::String(id.to_string())))
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

    #[cfg(feature = "rand")]
    #[test]
    fn test_random() {
        let runtime = setup_runtime();
        let expr = runtime.compile("random()").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let value = result.as_number().unwrap();
        assert!((0.0..1.0).contains(&value));
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_shuffle() {
        let runtime = setup_runtime();
        let expr = runtime.compile("shuffle(@)").unwrap();
        let data = Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
            Rc::new(Variable::Number(serde_json::Number::from(3))),
        ]);
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("uuid()").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let uuid_str = result.as_string().unwrap();
        assert_eq!(uuid_str.len(), 36); // UUID format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    }
}
