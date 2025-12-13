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
//! | [`add`](#add) | `add(a, b) → number` | Addition |
//! | [`subtract`](#subtract) | `subtract(a, b) → number` | Subtraction |
//! | [`multiply`](#multiply) | `multiply(a, b) → number` | Multiplication |
//! | [`divide`](#divide) | `divide(a, b) → number` | Division |
//! | [`mode`](#mode) | `mode(array) → any` | Most common value |
//! | [`to_fixed`](#to_fixed) | `to_fixed(n, precision) → string` | Fixed decimal places |
//! | [`format_number`](#format_number) | `format_number(n, precision?, suffix?) → string` | Format with separators |
//! | [`histogram`](#histogram) | `histogram(array, bins) → array` | Bucket values into bins |
//! | [`normalize`](#normalize) | `normalize(array) → array` | Normalize to 0-1 range |
//! | [`z_score`](#z_score) | `z_score(array) → array` | Calculate z-scores |
//! | [`correlation`](#correlation) | `correlation(arr1, arr2) → number` | Pearson correlation |
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
//!
//! ## add
//!
//! Adds two numbers.
//!
//! ```text
//! add(a, b) → number
//!
//! add(1, 2)         → 3
//! add(10.5, 2.5)    → 13
//! add(-5, 10)       → 5
//! ```
//!
//! ## subtract
//!
//! Subtracts second number from first.
//!
//! ```text
//! subtract(a, b) → number
//!
//! subtract(5, 3)    → 2
//! subtract(10, 20)  → -10
//! subtract(3.5, 1.2)→ 2.3
//! ```
//!
//! ## multiply
//!
//! Multiplies two numbers.
//!
//! ```text
//! multiply(a, b) → number
//!
//! multiply(3, 4)    → 12
//! multiply(2.5, 4)  → 10
//! multiply(-3, 5)   → -15
//! ```
//!
//! ## divide
//!
//! Divides first number by second.
//!
//! ```text
//! divide(a, b) → number
//!
//! divide(10, 2)     → 5
//! divide(7, 2)      → 3.5
//! divide(10, 0)     → error (division by zero)
//! ```
//!
//! ## mode
//!
//! Returns the most common value in an array. If there's a tie, returns
//! the first value that reached the highest count.
//!
//! ```text
//! mode(array) → any
//!
//! mode([1, 2, 2, 3])           → 2
//! mode(['a', 'b', 'a', 'c'])   → "a"
//! mode([1, 1, 2, 2, 3])        → 1    // tie: returns first
//! mode([])                     → null
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
    runtime.register_function("add", Box::new(AddFn::new()));
    runtime.register_function("subtract", Box::new(SubtractFn::new()));
    runtime.register_function("multiply", Box::new(MultiplyFn::new()));
    runtime.register_function("divide", Box::new(DivideFn::new()));
    runtime.register_function("mode", Box::new(ModeFn::new()));
    runtime.register_function("to_fixed", Box::new(ToFixedFn::new()));
    runtime.register_function("format_number", Box::new(FormatNumberFn::new()));
    runtime.register_function("histogram", Box::new(HistogramFn::new()));
    runtime.register_function("normalize", Box::new(NormalizeFn::new()));
    runtime.register_function("z_score", Box::new(ZScoreFn::new()));
    runtime.register_function("correlation", Box::new(CorrelationFn::new()));
    runtime.register_function("quantile", Box::new(QuantileFn::new()));
    runtime.register_function("moving_avg", Box::new(MovingAvgFn::new()));
    runtime.register_function("ewma", Box::new(EwmaFn::new()));
    runtime.register_function("covariance", Box::new(CovarianceFn::new()));
    runtime.register_function("standardize", Box::new(StandardizeFn::new()));
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
        let median = if len % 2 == 0 {
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
        let result = n.asin();
        // Return null for out-of-domain values (|n| > 1 produces NaN)
        if result.is_nan() {
            Ok(Rc::new(Variable::Null))
        } else {
            Ok(Rc::new(Variable::Number(
                serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
            )))
        }
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
        let result = n.acos();
        // Return null for out-of-domain values (|n| > 1 produces NaN)
        if result.is_nan() {
            Ok(Rc::new(Variable::Null))
        } else {
            Ok(Rc::new(Variable::Number(
                serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
            )))
        }
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

// =============================================================================
// add(a, b) -> number
// =============================================================================

define_function!(
    AddFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for AddFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(a + b).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// subtract(a, b) -> number
// =============================================================================

define_function!(
    SubtractFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for SubtractFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(a - b).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// multiply(a, b) -> number
// =============================================================================

define_function!(
    MultiplyFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for MultiplyFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(a * b).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// divide(a, b) -> number
// =============================================================================

define_function!(
    DivideFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for DivideFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;
        if b == 0.0 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Division by zero".to_owned()),
            ));
        }
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(a / b).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// mode(array) -> any (most common value)
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

        // Count occurrences - use JSON string representation as key
        let mut counts: std::collections::HashMap<String, (usize, Rcvar)> =
            std::collections::HashMap::new();

        for item in arr.iter() {
            let key = serde_json::to_string(&**item).unwrap_or_default();
            counts
                .entry(key)
                .and_modify(|(count, _)| *count += 1)
                .or_insert((1, item.clone()));
        }

        // Find the value with highest count
        let (_, (_, mode_value)) = counts
            .into_iter()
            .max_by_key(|(_, (count, _))| *count)
            .unwrap();

        Ok(mode_value)
    }
}

// =============================================================================
// to_fixed(number, precision) -> string
// Like JavaScript's Number.toFixed() - returns string with exact decimal places
// =============================================================================

define_function!(
    ToFixedFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for ToFixedFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        let precision = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected precision argument".to_owned()),
            )
        })? as usize;

        let result = format!("{:.prec$}", num, prec = precision);
        Ok(Rc::new(Variable::String(result)))
    }
}

// =============================================================================
// format_number(number, precision?, suffix?) -> string
// Format a number with thousand separators and optional suffix (k, M, B, etc.)
// =============================================================================

define_function!(
    FormatNumberFn,
    vec![ArgumentType::Number],
    Some(ArgumentType::Any)
);

impl Function for FormatNumberFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number argument".to_owned()),
            )
        })?;

        let precision = args
            .get(1)
            .and_then(|v| v.as_number())
            .map(|n| n as usize)
            .unwrap_or(0);

        let suffix = args
            .get(2)
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        // Handle suffix scaling (k, M, B, T)
        let (scaled_num, auto_suffix) = if let Some(ref s) = suffix {
            match s.as_str() {
                "k" | "K" => (num / 1_000.0, "k"),
                "M" => (num / 1_000_000.0, "M"),
                "B" => (num / 1_000_000_000.0, "B"),
                "T" => (num / 1_000_000_000_000.0, "T"),
                "auto" => {
                    // Auto-detect best suffix
                    let abs_num = num.abs();
                    if abs_num >= 1_000_000_000_000.0 {
                        (num / 1_000_000_000_000.0, "T")
                    } else if abs_num >= 1_000_000_000.0 {
                        (num / 1_000_000_000.0, "B")
                    } else if abs_num >= 1_000_000.0 {
                        (num / 1_000_000.0, "M")
                    } else if abs_num >= 1_000.0 {
                        (num / 1_000.0, "k")
                    } else {
                        (num, "")
                    }
                }
                _ => (num, s.as_str()),
            }
        } else {
            (num, "")
        };

        // Format with precision
        let formatted = format!("{:.prec$}", scaled_num, prec = precision);

        // Add thousand separators to the integer part
        let result = if suffix.is_none() || suffix.as_deref() == Some("") {
            add_thousand_separators(&formatted)
        } else {
            format!("{}{}", formatted, auto_suffix)
        };

        Ok(Rc::new(Variable::String(result)))
    }
}

/// Add thousand separators (commas) to a number string
fn add_thousand_separators(s: &str) -> String {
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts.get(1);

    // Handle negative sign
    let (sign, digits) = if let Some(stripped) = int_part.strip_prefix('-') {
        ("-", stripped)
    } else {
        ("", int_part)
    };

    // Add commas every 3 digits from the right
    let digit_chars: Vec<char> = digits.chars().collect();
    let len = digit_chars.len();
    let with_commas: String = digit_chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let pos_from_right = len - 1 - i;
            if pos_from_right > 0 && pos_from_right % 3 == 0 {
                format!("{},", c)
            } else {
                c.to_string()
            }
        })
        .collect();

    match dec_part {
        Some(dec) => format!("{}{}.{}", sign, with_commas, dec),
        None => format!("{}{}", sign, with_commas),
    }
}

// =============================================================================
// histogram(array, bins) -> array
// Bucket values into histogram bins
// =============================================================================

define_function!(
    HistogramFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for HistogramFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let num_bins = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number of bins".to_owned()),
            )
        })? as usize;

        if num_bins == 0 {
            return Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Number of bins must be greater than 0".to_owned()),
            ));
        }

        // Extract numeric values
        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Handle case where all values are the same
        let bin_width = if (max_val - min_val).abs() < f64::EPSILON {
            1.0
        } else {
            (max_val - min_val) / num_bins as f64
        };

        // Initialize bins
        let mut bins: Vec<(f64, f64, usize)> = (0..num_bins)
            .map(|i| {
                let bin_min = min_val + (i as f64 * bin_width);
                let bin_max = if i == num_bins - 1 {
                    max_val
                } else {
                    min_val + ((i + 1) as f64 * bin_width)
                };
                (bin_min, bin_max, 0)
            })
            .collect();

        // Count values in each bin
        for val in &values {
            let bin_idx = if (max_val - min_val).abs() < f64::EPSILON {
                0
            } else {
                let idx = ((val - min_val) / bin_width) as usize;
                idx.min(num_bins - 1)
            };
            bins[bin_idx].2 += 1;
        }

        // Convert to array of objects
        let result: Vec<Rcvar> = bins
            .into_iter()
            .map(|(bin_min, bin_max, count)| {
                let mut map = std::collections::BTreeMap::new();
                map.insert(
                    "min".to_string(),
                    Rc::new(Variable::Number(
                        serde_json::Number::from_f64(bin_min)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )) as Rcvar,
                );
                map.insert(
                    "max".to_string(),
                    Rc::new(Variable::Number(
                        serde_json::Number::from_f64(bin_max)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )) as Rcvar,
                );
                map.insert(
                    "count".to_string(),
                    Rc::new(Variable::Number(serde_json::Number::from(count))) as Rcvar,
                );
                Rc::new(Variable::Object(map)) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// normalize(array) -> array
// Normalize values to 0-1 range
// =============================================================================

define_function!(NormalizeFn, vec![ArgumentType::Array], None);

impl Function for NormalizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        // Extract numeric values
        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        let result: Vec<Rcvar> = values
            .iter()
            .map(|v| {
                let normalized = if range.abs() < f64::EPSILON {
                    0.0 // All values are the same
                } else {
                    (v - min_val) / range
                };
                Rc::new(Variable::Number(
                    serde_json::Number::from_f64(normalized)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// z_score(array) -> array
// Calculate z-scores (standard scores) for values
// =============================================================================

define_function!(ZScoreFn, vec![ArgumentType::Array], None);

impl Function for ZScoreFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        // Extract numeric values
        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();

        let result: Vec<Rcvar> = values
            .iter()
            .map(|v| {
                let z = if stddev.abs() < f64::EPSILON {
                    0.0 // All values are the same
                } else {
                    (v - mean) / stddev
                };
                Rc::new(Variable::Number(
                    serde_json::Number::from_f64(z).unwrap_or_else(|| serde_json::Number::from(0)),
                )) as Rcvar
            })
            .collect();

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// correlation(arr1, arr2) -> number
// Pearson correlation coefficient between two arrays
// =============================================================================

define_function!(
    CorrelationFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for CorrelationFn {
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

        // Extract numeric values
        let values1: Vec<f64> = arr1.iter().filter_map(|v| v.as_number()).collect();
        let values2: Vec<f64> = arr2.iter().filter_map(|v| v.as_number()).collect();

        if values1.is_empty() || values2.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        // Use the shorter length
        let n = values1.len().min(values2.len());
        if n == 0 {
            return Ok(Rc::new(Variable::Null));
        }

        let values1 = &values1[..n];
        let values2 = &values2[..n];

        let mean1 = values1.iter().sum::<f64>() / n as f64;
        let mean2 = values2.iter().sum::<f64>() / n as f64;

        let mut cov = 0.0;
        let mut var1 = 0.0;
        let mut var2 = 0.0;

        for i in 0..n {
            let d1 = values1[i] - mean1;
            let d2 = values2[i] - mean2;
            cov += d1 * d2;
            var1 += d1 * d1;
            var2 += d2 * d2;
        }

        let denom = (var1 * var2).sqrt();
        let correlation = if denom.abs() < f64::EPSILON {
            0.0 // No variance in one or both arrays
        } else {
            cov / denom
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(correlation)
                .unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// quantile(array, q) -> number
// Nth quantile (generalized percentile), q in [0, 1]
// =============================================================================

define_function!(
    QuantileFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for QuantileFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let q = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for quantile".to_owned()),
            )
        })?;

        if !(0.0..=1.0).contains(&q) {
            return Ok(Rc::new(Variable::Null));
        }

        let mut values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() {
            return Ok(Rc::new(Variable::Null));
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = values.len();
        let pos = q * (n - 1) as f64;
        let lower = pos.floor() as usize;
        let upper = pos.ceil() as usize;
        let frac = pos - lower as f64;

        let result = if lower == upper {
            values[lower]
        } else {
            values[lower] * (1.0 - frac) + values[upper] * frac
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(result).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// moving_avg(array, window) -> array
// Simple moving average
// =============================================================================

define_function!(
    MovingAvgFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for MovingAvgFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let window = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for window size".to_owned()),
            )
        })? as usize;

        if window == 0 {
            return Ok(Rc::new(Variable::Null));
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() || window > values.len() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let mut result: Vec<Rcvar> = Vec::new();

        // Compute moving averages for each position where we have enough data
        for i in 0..values.len() {
            if i + 1 < window {
                // Not enough data yet, use null
                result.push(Rc::new(Variable::Null));
            } else {
                let start = i + 1 - window;
                let sum: f64 = values[start..=i].iter().sum();
                let avg = sum / window as f64;
                result.push(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(avg)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )));
            }
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// ewma(array, alpha) -> array
// Exponential weighted moving average
// =============================================================================

define_function!(
    EwmaFn,
    vec![ArgumentType::Array, ArgumentType::Number],
    None
);

impl Function for EwmaFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let alpha = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for alpha".to_owned()),
            )
        })?;

        if !(0.0..=1.0).contains(&alpha) {
            return Ok(Rc::new(Variable::Null));
        }

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let mut result: Vec<Rcvar> = Vec::new();
        let mut ewma = values[0];

        for value in &values {
            ewma = alpha * value + (1.0 - alpha) * ewma;
            result.push(Rc::new(Variable::Number(
                serde_json::Number::from_f64(ewma).unwrap_or_else(|| serde_json::Number::from(0)),
            )));
        }

        Ok(Rc::new(Variable::Array(result)))
    }
}

// =============================================================================
// covariance(arr1, arr2) -> number
// Covariance between two arrays
// =============================================================================

define_function!(
    CovarianceFn,
    vec![ArgumentType::Array, ArgumentType::Array],
    None
);

impl Function for CovarianceFn {
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

        let values1: Vec<f64> = arr1.iter().filter_map(|v| v.as_number()).collect();
        let values2: Vec<f64> = arr2.iter().filter_map(|v| v.as_number()).collect();

        if values1.is_empty() || values1.len() != values2.len() {
            return Ok(Rc::new(Variable::Null));
        }

        let n = values1.len() as f64;
        let mean1: f64 = values1.iter().sum::<f64>() / n;
        let mean2: f64 = values2.iter().sum::<f64>() / n;

        let cov: f64 = values1
            .iter()
            .zip(values2.iter())
            .map(|(x, y)| (x - mean1) * (y - mean2))
            .sum::<f64>()
            / n;

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(cov).unwrap_or_else(|| serde_json::Number::from(0)),
        )))
    }
}

// =============================================================================
// standardize(array) -> array
// Standardize to mean=0, std=1 (z-score normalization)
// =============================================================================

define_function!(StandardizeFn, vec![ArgumentType::Array], None);

impl Function for StandardizeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected array argument".to_owned()),
            )
        })?;

        let values: Vec<f64> = arr.iter().filter_map(|v| v.as_number()).collect();

        if values.is_empty() {
            return Ok(Rc::new(Variable::Array(vec![])));
        }

        let n = values.len() as f64;
        let mean: f64 = values.iter().sum::<f64>() / n;
        let variance: f64 = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let result: Vec<Rcvar> = values
            .iter()
            .map(|x| {
                let standardized = if std_dev.abs() < f64::EPSILON {
                    0.0
                } else {
                    (x - mean) / std_dev
                };
                Rc::new(Variable::Number(
                    serde_json::Number::from_f64(standardized)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ))
            })
            .collect();

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
    #[allow(clippy::approx_constant)]
    fn test_round() {
        let runtime = setup_runtime();
        let expr = runtime.compile("round(`3.14159`, `2`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!((result.as_number().unwrap() - 3.14_f64).abs() < 0.001);
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

    #[test]
    fn test_add() {
        let runtime = setup_runtime();
        let expr = runtime.compile("add(`1`, `2`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 3);
    }

    #[test]
    fn test_subtract() {
        let runtime = setup_runtime();
        let expr = runtime.compile("subtract(`10`, `3`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 7);
    }

    #[test]
    fn test_multiply() {
        let runtime = setup_runtime();
        let expr = runtime.compile("multiply(`4`, `5`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 20);
    }

    #[test]
    fn test_divide() {
        let runtime = setup_runtime();
        let expr = runtime.compile("divide(`10`, `4`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.5);
    }

    #[test]
    fn test_mode_numbers() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mode(`[1, 2, 2, 3]`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap() as i64, 2);
    }

    #[test]
    fn test_mode_strings() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("mode(`[\"a\", \"b\", \"a\", \"c\"]`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "a");
    }

    #[test]
    fn test_mode_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("mode(`[]`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_to_fixed() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_fixed(`3.14159`, `2`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "3.14");
    }

    #[test]
    fn test_to_fixed_padding() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_fixed(`3.1`, `3`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "3.100");
    }

    #[test]
    fn test_format_number_with_separators() {
        let runtime = setup_runtime();
        let expr = runtime.compile("format_number(`1234567.89`, `2`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "1,234,567.89");
    }

    #[test]
    fn test_format_number_with_k_suffix() {
        let runtime = setup_runtime();
        let expr = runtime.compile("format_number(`1500`, `1`, 'k')").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "1.5k");
    }

    #[test]
    fn test_format_number_with_m_suffix() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("format_number(`1500000`, `1`, 'M')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "1.5M");
    }

    #[test]
    fn test_format_number_auto_suffix() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("format_number(`1500000000`, `2`, 'auto')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "1.50B");
    }

    #[test]
    fn test_histogram() {
        let runtime = setup_runtime();
        let expr = runtime.compile("histogram(@, `3`)").unwrap();
        let data: Variable = serde_json::from_str("[1, 2, 3, 4, 5, 6, 7, 8, 9]").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // Each bin should have 3 values
        for bin in arr {
            let obj = bin.as_object().unwrap();
            assert!(obj.contains_key("min"));
            assert!(obj.contains_key("max"));
            assert!(obj.contains_key("count"));
        }
    }

    #[test]
    fn test_normalize() {
        let runtime = setup_runtime();
        let expr = runtime.compile("normalize(@)").unwrap();
        let data: Variable = serde_json::from_str("[0, 50, 100]").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert!((arr[0].as_number().unwrap() - 0.0).abs() < 0.001);
        assert!((arr[1].as_number().unwrap() - 0.5).abs() < 0.001);
        assert!((arr[2].as_number().unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_z_score() {
        let runtime = setup_runtime();
        let expr = runtime.compile("z_score(@)").unwrap();
        let data: Variable = serde_json::from_str("[1, 2, 3, 4, 5]").unwrap();
        let result = expr.search(&data).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        // Middle value (3) should have z-score of 0
        assert!((arr[2].as_number().unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_positive() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("correlation(`[1, 2, 3]`, `[1, 2, 3]`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!((result.as_number().unwrap() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_correlation_negative() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("correlation(`[1, 2, 3]`, `[3, 2, 1]`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!((result.as_number().unwrap() - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_quantile_median() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("quantile(`[1, 2, 3, 4, 5]`, `0.5`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_quantile_quartiles() {
        let runtime = setup_runtime();
        // First quartile
        let expr = runtime
            .compile("quantile(`[1, 2, 3, 4, 5]`, `0.25`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        // Third quartile
        let expr = runtime
            .compile("quantile(`[1, 2, 3, 4, 5]`, `0.75`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);
    }

    #[test]
    fn test_moving_avg() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("moving_avg(`[1, 2, 3, 4, 5, 6]`, `3`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 6);
        assert!(arr[0].is_null());
        assert!(arr[1].is_null());
        assert_eq!(arr[2].as_number().unwrap(), 2.0); // (1+2+3)/3
        assert_eq!(arr[3].as_number().unwrap(), 3.0); // (2+3+4)/3
        assert_eq!(arr[4].as_number().unwrap(), 4.0); // (3+4+5)/3
        assert_eq!(arr[5].as_number().unwrap(), 5.0); // (4+5+6)/3
    }

    #[test]
    fn test_ewma() {
        let runtime = setup_runtime();
        let expr = runtime.compile("ewma(`[1, 2, 3, 4, 5]`, `0.5`)").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        // First value is just the first value
        assert_eq!(arr[0].as_number().unwrap(), 1.0);
        // Subsequent values: alpha * current + (1-alpha) * prev_ewma
        assert_eq!(arr[1].as_number().unwrap(), 1.5); // 0.5*2 + 0.5*1
        assert_eq!(arr[2].as_number().unwrap(), 2.25); // 0.5*3 + 0.5*1.5
    }

    #[test]
    fn test_covariance() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("covariance(`[1, 2, 3]`, `[1, 2, 3]`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        // Variance of [1,2,3] is 2/3
        assert!((result.as_number().unwrap() - 0.666666).abs() < 0.01);
    }

    #[test]
    fn test_covariance_negative() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("covariance(`[1, 2, 3]`, `[3, 2, 1]`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert!((result.as_number().unwrap() - (-0.666666)).abs() < 0.01);
    }

    #[test]
    fn test_standardize() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("standardize(`[10, 20, 30, 40, 50]`)")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        // Mean is 30, std is ~14.14
        // First value: (10-30)/14.14 ≈ -1.41
        assert!((arr[0].as_number().unwrap() - (-1.414)).abs() < 0.01);
        // Middle value should be 0
        assert!(arr[2].as_number().unwrap().abs() < 0.001);
        // Last value: (50-30)/14.14 ≈ 1.41
        assert!((arr[4].as_number().unwrap() - 1.414).abs() < 0.01);
    }
}
