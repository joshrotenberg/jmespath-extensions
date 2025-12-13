//! Duration parsing and formatting functions.
//!
//! This module provides duration functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category duration`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::duration;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! duration::register(&mut runtime);
//! ```

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Variable, rcvar,
};
use crate::define_function;

define_function!(ParseDurationFn, vec![ArgumentType::String], None);

impl Function for ParseDurationFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        match parse_duration_str(s) {
            Some(secs) => Ok(rcvar(Variable::Number(
                serde_json::Number::from_f64(secs as f64).unwrap(),
            ))),
            None => Ok(rcvar(Variable::Null)),
        }
    }
}

define_function!(FormatDurationFn, vec![ArgumentType::Number], None);

impl Function for FormatDurationFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let total_secs = num as u64;
        let formatted = format_duration_secs(total_secs);

        Ok(rcvar(Variable::String(formatted)))
    }
}

define_function!(DurationHoursFn, vec![ArgumentType::Number], None);

impl Function for DurationHoursFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let total_secs = num as u64;
        let hours = (total_secs / 3600) % 24;

        Ok(rcvar(Variable::Number(serde_json::Number::from(hours))))
    }
}

define_function!(DurationMinutesFn, vec![ArgumentType::Number], None);

impl Function for DurationMinutesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let total_secs = num as u64;
        let minutes = (total_secs / 60) % 60;

        Ok(rcvar(Variable::Number(serde_json::Number::from(minutes))))
    }
}

define_function!(DurationSecondsFn, vec![ArgumentType::Number], None);

impl Function for DurationSecondsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let total_secs = num as u64;
        let seconds = total_secs % 60;

        Ok(rcvar(Variable::Number(serde_json::Number::from(seconds))))
    }
}

/// Parse a duration string into total seconds.
fn parse_duration_str(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let mut total_secs: u64 = 0;
    let mut current_num = String::new();

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_ascii_digit() {
            current_num.push(c);
            i += 1;
        } else if c.is_ascii_alphabetic() {
            let num: u64 = if current_num.is_empty() {
                return None;
            } else {
                current_num.parse().ok()?
            };
            current_num.clear();

            let mut unit = String::new();
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                unit.push(chars[i]);
                i += 1;
            }

            let multiplier = match unit.as_str() {
                "w" | "week" | "weeks" => 7 * 24 * 3600,
                "d" | "day" | "days" => 24 * 3600,
                "h" | "hr" | "hrs" | "hour" | "hours" => 3600,
                "m" | "min" | "mins" | "minute" | "minutes" => 60,
                "s" | "sec" | "secs" | "second" | "seconds" => 1,
                _ => return None,
            };

            total_secs += num * multiplier;
        } else if c.is_whitespace() {
            i += 1;
        } else {
            return None;
        }
    }

    if !current_num.is_empty() {
        let num: u64 = current_num.parse().ok()?;
        total_secs += num;
    }

    Some(total_secs)
}

/// Format seconds as a human-readable duration string.
fn format_duration_secs(total_secs: u64) -> String {
    if total_secs == 0 {
        return "0s".to_string();
    }

    let weeks = total_secs / (7 * 24 * 3600);
    let days = (total_secs / (24 * 3600)) % 7;
    let hours = (total_secs / 3600) % 24;
    let minutes = (total_secs / 60) % 60;
    let seconds = total_secs % 60;

    let mut result = String::new();

    if weeks > 0 {
        result.push_str(&format!("{}w", weeks));
    }
    if days > 0 {
        result.push_str(&format!("{}d", days));
    }
    if hours > 0 {
        result.push_str(&format!("{}h", hours));
    }
    if minutes > 0 {
        result.push_str(&format!("{}m", minutes));
    }
    if seconds > 0 {
        result.push_str(&format!("{}s", seconds));
    }

    result
}

/// Register all duration functions with the runtime.
pub fn register(runtime: &mut crate::Runtime) {
    runtime.register_function("parse_duration", Box::new(ParseDurationFn::new()));
    runtime.register_function("format_duration", Box::new(FormatDurationFn::new()));
    runtime.register_function("duration_hours", Box::new(DurationHoursFn::new()));
    runtime.register_function("duration_minutes", Box::new(DurationMinutesFn::new()));
    runtime.register_function("duration_seconds", Box::new(DurationSecondsFn::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration_str("1h"), Some(3600));
        assert_eq!(parse_duration_str("30m"), Some(1800));
        assert_eq!(parse_duration_str("45s"), Some(45));
        assert_eq!(parse_duration_str("1h30m"), Some(5400));
        assert_eq!(parse_duration_str("2h30m45s"), Some(9045));
        assert_eq!(parse_duration_str("1d"), Some(86400));
        assert_eq!(parse_duration_str("1w"), Some(604800));
        assert_eq!(parse_duration_str("1w2d3h4m5s"), Some(788645));
        assert_eq!(parse_duration_str("1 hour 30 minutes"), Some(5400));
        assert_eq!(parse_duration_str(""), None);
        assert_eq!(parse_duration_str("invalid"), None);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration_secs(0), "0s");
        assert_eq!(format_duration_secs(45), "45s");
        assert_eq!(format_duration_secs(60), "1m");
        assert_eq!(format_duration_secs(3600), "1h");
        assert_eq!(format_duration_secs(5400), "1h30m");
        assert_eq!(format_duration_secs(86400), "1d");
        assert_eq!(format_duration_secs(90061), "1d1h1m1s");
        assert_eq!(format_duration_secs(788645), "1w2d3h4m5s");
    }

    #[test]
    fn test_roundtrip() {
        let values = [0, 45, 60, 3600, 5400, 86400, 90061, 788645];
        for &v in &values {
            let formatted = format_duration_secs(v);
            let parsed = parse_duration_str(&formatted).unwrap_or(0);
            assert_eq!(
                parsed, v,
                "Roundtrip failed for {}: {} -> {}",
                v, formatted, parsed
            );
        }
    }
}
