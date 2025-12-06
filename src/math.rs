//! Math and statistics functions.
//!
//! This module provides mathematical operations and statistical functions.
//!
//! # Function Reference
//!
//! | Function | Signature | Description |
//! |----------|-----------|-------------|
//! | [`round`](#round) | `round(n, precision?) → number` | Round to precision |
//! | [`floor_fn`](#floor_fn) | `floor_fn(n) → number` | Round down |
//! | [`ceil_fn`](#ceil_fn) | `ceil_fn(n) → number` | Round up |
//! | [`abs_fn`](#abs_fn) | `abs_fn(n) → number` | Absolute value |
//! | [`mod_fn`](#mod_fn) | `mod_fn(a, b) → number` | Modulo operation |
//! | [`pow`](#pow) | `pow(base, exp) → number` | Exponentiation |
//! | [`sqrt`](#sqrt) | `sqrt(n) → number` | Square root |
//! | [`log`](#log) | `log(n, base?) → number` | Logarithm |
//! | [`clamp`](#clamp) | `clamp(n, min, max) → number` | Clamp to range |
//! | [`median`](#median) | `median(array) → number` | Median value |
//! | [`percentile`](#percentile) | `percentile(array, p) → number` | Percentile |
//! | [`variance`](#variance) | `variance(array) → number` | Statistical variance |
//! | [`stddev`](#stddev) | `stddev(array) → number` | Standard deviation |
//! | [`sin`](#sin) | `sin(n) → number` | Sine (radians) |
//! | [`cos`](#cos) | `cos(n) → number` | Cosine (radians) |
//! | [`tan`](#tan) | `tan(n) → number` | Tangent (radians) |
//! | [`asin`](#asin) | `asin(n) → number` | Arcsine |
//! | [`acos`](#acos) | `acos(n) → number` | Arccosine |
//! | [`atan`](#atan) | `atan(n) → number` | Arctangent |
//! | [`atan2`](#atan2) | `atan2(y, x) → number` | Two-argument arctangent |
//! | [`deg_to_rad`](#deg_to_rad) | `deg_to_rad(deg) → number` | Degrees to radians |
//! | [`rad_to_deg`](#rad_to_deg) | `rad_to_deg(rad) → number` | Radians to degrees |
//! | [`sign`](#sign) | `sign(n) → number` | Sign (-1, 0, 1) |
//!
//! # Examples
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::math;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! math::register(&mut runtime);
//!
//! // Round to 2 decimal places
//! let expr = runtime.compile("round(@, `2`)").unwrap();
//! let data = Variable::from_json("3.14159").unwrap();
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_number().unwrap(), 3.14);
//! ```
//!
//! # Function Details
//!
//! ## round
//!
//! Rounds a number to a specified precision (decimal places).
//!
//! ```text
//! round(number, precision?) → number
//!
//! round(3.14159)        → 3
//! round(3.14159, 2)     → 3.14
//! round(3.14159, 4)     → 3.1416
//! round(1234.5, -2)     → 1200       // Negative precision rounds to tens, hundreds, etc.
//! ```
//!
//! ## floor_fn
//!
//! Rounds a number down to the nearest integer.
//!
//! ```text
//! floor_fn(number) → number
//!
//! floor_fn(3.7)     → 3
//! floor_fn(3.2)     → 3
//! floor_fn(-3.2)    → -4
//! ```
//!
//! ## ceil_fn
//!
//! Rounds a number up to the nearest integer.
//!
//! ```text
//! ceil_fn(number) → number
//!
//! ceil_fn(3.2)      → 4
//! ceil_fn(3.7)      → 4
//! ceil_fn(-3.7)     → -3
//! ```
//!
//! ## abs_fn
//!
//! Returns the absolute value of a number.
//!
//! ```text
//! abs_fn(number) → number
//!
//! abs_fn(-5)        → 5
//! abs_fn(5)         → 5
//! abs_fn(-3.14)     → 3.14
//! ```
//!
//! ## mod_fn
//!
//! Returns the remainder of division (modulo operation).
//!
//! ```text
//! mod_fn(a, b) → number
//!
//! mod_fn(10, 3)     → 1
//! mod_fn(10, 5)     → 0
//! mod_fn(-10, 3)    → -1
//! ```
//!
//! ## pow
//!
//! Raises a number to a power.
//!
//! ```text
//! pow(base, exponent) → number
//!
//! pow(2, 8)         → 256
//! pow(10, 3)        → 1000
//! pow(4, 0.5)       → 2          // Square root
//! pow(2, -1)        → 0.5
//! ```
//!
//! ## sqrt
//!
//! Returns the square root of a number.
//!
//! ```text
//! sqrt(number) → number
//!
//! sqrt(16)          → 4
//! sqrt(2)           → 1.4142...
//! sqrt(0)           → 0
//! ```
//!
//! ## log
//!
//! Returns the logarithm of a number. Default base is e (natural log).
//!
//! ```text
//! log(number, base?) → number
//!
//! log(10)           → 2.302...    // Natural log (ln)
//! log(100, 10)      → 2           // Log base 10
//! log(8, 2)         → 3           // Log base 2
//! ```
//!
//! ## clamp
//!
//! Restricts a number to a range.
//!
//! ```text
//! clamp(number, min, max) → number
//!
//! clamp(15, 0, 10)      → 10
//! clamp(-5, 0, 10)      → 0
//! clamp(5, 0, 10)       → 5
//! ```
//!
//! ## median
//!
//! Returns the median (middle value) of an array of numbers.
//!
//! ```text
//! median(array) → number
//!
//! median([1, 2, 3, 4, 5])       → 3
//! median([1, 2, 3, 4])          → 2.5    // Average of middle two
//! median([5, 1, 3])             → 3
//! ```
//!
//! ## percentile
//!
//! Returns the value at a given percentile in an array.
//!
//! ```text
//! percentile(array, p) → number
//!
//! percentile([1, 2, 3, 4, 5], 50)    → 3      // 50th percentile = median
//! percentile([1, 2, 3, 4, 5], 0)     → 1      // Minimum
//! percentile([1, 2, 3, 4, 5], 100)   → 5      // Maximum
//! ```
//!
//! ## variance
//!
//! Returns the statistical variance of an array.
//!
//! ```text
//! variance(array) → number
//!
//! variance([1, 2, 3, 4, 5])     → 2.5
//! variance([10, 10, 10])        → 0
//! ```
//!
//! ## stddev
//!
//! Returns the standard deviation of an array.
//!
//! ```text
//! stddev(array) → number
//!
//! stddev([1, 2, 3, 4, 5])       → 1.5811...
//! stddev([10, 10, 10])          → 0
//! ```
//!
//! ## Trigonometric Functions
//!
//! All trig functions work with radians.
//!
//! ```text
//! sin(radians) → number         // Sine
//! cos(radians) → number         // Cosine
//! tan(radians) → number         // Tangent
//! asin(value) → radians         // Arcsine
//! acos(value) → radians         // Arccosine
//! atan(value) → radians         // Arctangent
//! atan2(y, x) → radians         // Two-argument arctangent
//!
//! sin(0)            → 0
//! cos(0)            → 1
//! sin(3.14159/2)    → 1         // sin(π/2) ≈ 1
//! ```
//!
//! ## deg_to_rad
//!
//! Converts degrees to radians.
//!
//! ```text
//! deg_to_rad(degrees) → radians
//!
//! deg_to_rad(180)       → 3.14159...   // π
//! deg_to_rad(90)        → 1.5707...    // π/2
//! deg_to_rad(360)       → 6.2831...    // 2π
//! ```
//!
//! ## rad_to_deg
//!
//! Converts radians to degrees.
//!
//! ```text
//! rad_to_deg(radians) → degrees
//!
//! rad_to_deg(3.14159)   → 180
//! rad_to_deg(1.5707)    → 90
//! ```
//!
//! ## sign
//!
//! Returns the sign of a number (-1, 0, or 1).
//!
//! ```text
//! sign(number) → number
//!
//! sign(42)          → 1
//! sign(-42)         → -1
//! sign(0)           → 0
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all math functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("round", Box::new(RoundFn::new()));
    runtime.register_function("floor_fn", Box::new(FloorFn::new()));
    runtime.register_function("ceil_fn", Box::new(CeilFn::new()));
    runtime.register_function("abs_fn", Box::new(AbsFn::new()));
    runtime.register_function("mod_fn", Box::new(ModFn::new()));
    runtime.register_function("pow", Box::new(PowFn::new()));
    runtime.register_function("sqrt", Box::new(SqrtFn::new()));
    runtime.register_function("log", Box::new(LogFn::new()));
    runtime.register_function("clamp", Box::new(ClampFn::new()));
    runtime.register_function("median", Box::new(MedianFn::new()));
    runtime.register_function("percentile", Box::new(PercentileFn::new()));
    runtime.register_function("variance", Box::new(VarianceFn::new()));
    runtime.register_function("stddev", Box::new(StddevFn::new()));
    runtime.register_function("sin", Box::new(SinFn::new()));
    runtime.register_function("cos", Box::new(CosFn::new()));
    runtime.register_function("tan", Box::new(TanFn::new()));
    runtime.register_function("asin", Box::new(AsinFn::new()));
    runtime.register_function("acos", Box::new(AcosFn::new()));
    runtime.register_function("atan", Box::new(AtanFn::new()));
    runtime.register_function("atan2", Box::new(Atan2Fn::new()));
    runtime.register_function("deg_to_rad", Box::new(DegToRadFn::new()));
    runtime.register_function("rad_to_deg", Box::new(RadToDegFn::new()));
    runtime.register_function("sign", Box::new(SignFn::new()));
}

// =============================================================================
// round(number, precision?) -> number
// =============================================================================

define_function!(
    RoundFn,
    vec![ArgumentType::Number],
    Some(ArgumentType::Number)
);

impl Function for RoundFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        let precision = if args.len() > 1 {
            args[1].as_number().map(|p| p as i32).unwrap_or(0)
        } else {
            0
        };

        let result = if precision == 0 {
            n.round()
        } else {
            let multiplier = 10_f64.powi(precision);
            (n * multiplier).round() / multiplier
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// floor_fn(number) -> number
// =============================================================================

define_function!(FloorFn, vec![ArgumentType::Number], None);

impl Function for FloorFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Number(serde_json::Number::from(
            n.floor() as i64,
        ))))
    }
}

// =============================================================================
// ceil_fn(number) -> number
// =============================================================================

define_function!(CeilFn, vec![ArgumentType::Number], None);

impl Function for CeilFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Number(serde_json::Number::from(
            n.ceil() as i64,
        ))))
    }
}

// =============================================================================
// abs_fn(number) -> number
// =============================================================================

define_function!(AbsFn, vec![ArgumentType::Number], None);

impl Function for AbsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.abs()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// mod_fn(number, divisor) -> number
// =============================================================================

define_function!(
    ModFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for ModFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        let divisor = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected divisor argument".to_owned()),
            )
        })?;

        if divisor == 0.0 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Division by zero".to_owned()),
            ));
        }

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n % divisor)
                .unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// pow(base, exponent) -> number
// =============================================================================

define_function!(
    PowFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for PowFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let base = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected base number".to_owned()),
            )
        })?;

        let exp = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected exponent number".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(base.powf(exp))
                .unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// sqrt(number) -> number
// =============================================================================

define_function!(SqrtFn, vec![ArgumentType::Number], None);

impl Function for SqrtFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        if n < 0.0 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Cannot take square root of negative number".to_owned()),
            ));
        }

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.sqrt()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// log(number, base?) -> number (default base e)
// =============================================================================

define_function!(
    LogFn,
    vec![ArgumentType::Number],
    Some(ArgumentType::Number)
);

impl Function for LogFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        if n <= 0.0 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Logarithm requires positive number".to_owned()),
            ));
        }

        let result = if args.len() > 1 {
            let base = args[1].as_number().ok_or_else(|| {
                JmespathError::new(
                    ctx.expression,
                    0,
                    ErrorReason::Parse("Expected base number".to_owned()),
                )
            })?;
            n.log(base)
        } else {
            n.ln()
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// clamp(number, min, max) -> number
// =============================================================================

define_function!(
    ClampFn,
    vec![
        ArgumentType::Number,
        ArgumentType::Number,
        ArgumentType::Number
    ],
    None
);

impl Function for ClampFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        let min = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected min number".to_owned()),
            )
        })?;

        let max = args[2].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected max number".to_owned()),
            )
        })?;

        let result = n.max(min).min(max);

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// median(array) -> number
// =============================================================================

define_function!(MedianFn, vec![ArgumentType::Array], None);

impl Function for MedianFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let mut numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if numbers.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = numbers.len();
        let median = if len.is_multiple_of(2) {
            (numbers[len / 2 - 1] + numbers[len / 2]) / 2.0
        } else {
            numbers[len / 2]
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(median).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// percentile(array, p) -> number (pth percentile, p in 0-100)
// =============================================================================

define_function!(
    PercentileFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for PercentileFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let p = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected percentile value".to_owned()),
            )
        })?;

        if !(0.0..=100.0).contains(&p) {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Percentile must be between 0 and 100".to_owned()),
            ));
        }

        let mut numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if numbers.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = numbers.len();
        if len == 1 {
            return Ok(Rc::new(Variable::Number(
                serde_json::Number::from_f64(numbers[0])
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            )));
        }

        let rank = (p / 100.0) * (len - 1) as f64;
        let lower_idx = rank.floor() as usize;
        let upper_idx = rank.ceil() as usize;
        let fraction = rank - lower_idx as f64;

        let result = if lower_idx == upper_idx {
            numbers[lower_idx]
        } else {
            numbers[lower_idx] * (1.0 - fraction) + numbers[upper_idx] * fraction
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// variance(array) -> number (population variance)
// =============================================================================

define_function!(VarianceFn, vec![ArgumentType::Array], None);

impl Function for VarianceFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if numbers.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
        let variance =
            numbers.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / numbers.len() as f64;

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(variance).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// stddev(array) -> number (population standard deviation)
// =============================================================================

define_function!(StddevFn, vec![ArgumentType::Array], None);

impl Function for StddevFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let numbers: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if numbers.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
        let variance =
            numbers.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / numbers.len() as f64;
        let stddev = variance.sqrt();

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(stddev).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// Trigonometric functions
// =============================================================================

define_function!(SinFn, vec![ArgumentType::Number], None);

impl Function for SinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.sin()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(CosFn, vec![ArgumentType::Number], None);

impl Function for CosFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.cos()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(TanFn, vec![ArgumentType::Number], None);

impl Function for TanFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.tan()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(AsinFn, vec![ArgumentType::Number], None);

impl Function for AsinFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.asin()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(AcosFn, vec![ArgumentType::Number], None);

impl Function for AcosFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.acos()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(AtanFn, vec![ArgumentType::Number], None);

impl Function for AtanFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.atan()).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(
    Atan2Fn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for Atan2Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let y = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        let x = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(y.atan2(x)).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(DegToRadFn, vec![ArgumentType::Number], None);

impl Function for DegToRadFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.to_radians())
                .unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(RadToDegFn, vec![ArgumentType::Number], None);

impl Function for RadToDegFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(n.to_degrees())
                .unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

define_function!(SignFn, vec![ArgumentType::Number], None);

impl Function for SignFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let n = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        let sign = if n > 0.0 {
            1
        } else if n < 0.0 {
            -1
        } else {
            0
        };
        Ok(Rc::new(Variable::Number(serde_json::Number::from(sign))))
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
    fn test_round() {
        let runtime = setup_runtime();
        let expr = runtime.compile("round(`3.14159`, `2`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!((result.as_number().unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_sqrt() {
        let runtime = setup_runtime();
        let expr = runtime.compile("sqrt(`16`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 4);
    }

    #[test]
    fn test_clamp() {
        let runtime = setup_runtime();
        let expr = runtime.compile("clamp(`5`, `0`, `3`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 3);
    }
}
