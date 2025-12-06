//! Random and UUID generation functions.
//!
//! These functions provide random number generation and UUID creation.
//! Requires the `rand` and/or `uuid` features.

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
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
// =============================================================================

#[cfg(feature = "rand")]
define_function!(RandomFn, vec![], None);

#[cfg(feature = "rand")]
impl Function for RandomFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        use rand::Rng;
        let value: f64 = rand::thread_rng().gen();

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// shuffle(array) -> array (randomly shuffled)
// =============================================================================

#[cfg(feature = "rand")]
define_function!(ShuffleFn, vec![ArgumentType::Array], None);

#[cfg(feature = "rand")]
impl Function for ShuffleFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        use rand::seq::SliceRandom;
        let mut result: Vec<Rcvar> = arr.clone();
        result.shuffle(&mut rand::thread_rng());

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// sample(array, n) -> array (random sample of n elements)
// =============================================================================

#[cfg(feature = "rand")]
define_function!(
    SampleFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

#[cfg(feature = "rand")]
impl Function for SampleFn {
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

        use rand::seq::SliceRandom;
        let sample: Vec<Rcvar> = arr
            .choose_multiple(&mut rand::thread_rng(), n.min(arr.len()))
            .cloned()
            .collect();

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
        assert!(value >= 0.0 && value < 1.0);
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
