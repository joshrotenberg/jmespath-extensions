//! Date/time functions for JMESPath.
//!
//! This module provides functions for working with dates and times.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `now()` | Current Unix timestamp in seconds |
//! | `now_millis()` | Current Unix timestamp in milliseconds |
//! | `parse_date(string, format?)` | Parse date string to timestamp |
//! | `format_date(timestamp, format)` | Format timestamp to string |
//! | `date_add(timestamp, amount, unit)` | Add time to timestamp |
//! | `date_diff(ts1, ts2, unit)` | Difference between timestamps |
//!
//! # Format Specifiers
//!
//! Uses [chrono strftime format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html):
//!
//! | Specifier | Description | Example |
//! |-----------|-------------|---------|
//! | `%Y` | Year with century | 2024 |
//! | `%m` | Month (01-12) | 07 |
//! | `%d` | Day of month (01-31) | 15 |
//! | `%H` | Hour (00-23) | 14 |
//! | `%M` | Minute (00-59) | 30 |
//! | `%S` | Second (00-59) | 45 |
//! | `%Y-%m-%d` | ISO date | 2024-07-15 |
//! | `%Y-%m-%dT%H:%M:%S` | ISO datetime | 2024-07-15T14:30:45 |
//!
//! # Time Units
//!
//! For `date_add` and `date_diff`:
//! - `seconds`, `second`, `s`
//! - `minutes`, `minute`, `m`
//! - `hours`, `hour`, `h`
//! - `days`, `day`, `d`
//! - `weeks`, `week`, `w`
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "datetime")]
//! # fn main() {
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::datetime;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! datetime::register(&mut runtime);
//!
//! // Format a timestamp
//! let expr = runtime.compile("format_date(`1720000000`, '%Y-%m-%d')").unwrap();
//! let result = expr.search(&Variable::Null).unwrap();
//! assert_eq!(result.as_string().unwrap(), "2024-07-03");
//! # }
//! # #[cfg(not(feature = "datetime"))]
//! # fn main() {}
//! ```

use std::rc::Rc;

use chrono::{DateTime, NaiveDateTime, TimeDelta, TimeZone, Utc};

use crate::common::{Function, custom_error};
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Variable, define_function};

/// Register all datetime functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("now", Box::new(NowFn::new()));
    runtime.register_function("now_millis", Box::new(NowMillisFn::new()));
    runtime.register_function("parse_date", Box::new(ParseDateFn::new()));
    runtime.register_function("format_date", Box::new(FormatDateFn::new()));
    runtime.register_function("date_add", Box::new(DateAddFn::new()));
    runtime.register_function("date_diff", Box::new(DateDiffFn::new()));
}

// now() -> number
define_function!(NowFn, vec![], None);

impl Function for NowFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let ts = Utc::now().timestamp();
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(ts as f64).unwrap(),
        )))
    }
}

// now_millis() -> number
define_function!(NowMillisFn, vec![], None);

impl Function for NowMillisFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let ts = Utc::now().timestamp_millis();
        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(ts as f64).unwrap(),
        )))
    }
}

// parse_date(string, format?) -> number | null
define_function!(
    ParseDateFn,
    vec![ArgumentType::String],
    Some(ArgumentType::String)
);

impl Function for ParseDateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().unwrap();

        if args.len() > 1 {
            // Custom format provided
            let format = args[1].as_string().unwrap();
            match NaiveDateTime::parse_from_str(s, format) {
                Ok(dt) => Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(dt.and_utc().timestamp() as f64).unwrap(),
                ))),
                Err(_) => Ok(Rc::new(Variable::Null)),
            }
        } else {
            // Try common formats
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(dt.timestamp() as f64).unwrap(),
                )));
            }
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                return Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(dt.and_utc().timestamp() as f64).unwrap(),
                )));
            }
            if let Ok(dt) =
                NaiveDateTime::parse_from_str(&format!("{}T00:00:00", s), "%Y-%m-%dT%H:%M:%S")
            {
                return Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(dt.and_utc().timestamp() as f64).unwrap(),
                )));
            }
            Ok(Rc::new(Variable::Null))
        }
    }
}

// format_date(timestamp, format) -> string
define_function!(
    FormatDateFn,
    vec![ArgumentType::Number, ArgumentType::String],
    None
);

impl Function for FormatDateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_number().unwrap();
        let format = args[1].as_string().unwrap();

        let dt = Utc.timestamp_opt(ts as i64, 0);
        match dt {
            chrono::LocalResult::Single(dt) => {
                Ok(Rc::new(Variable::String(dt.format(format).to_string())))
            }
            _ => Ok(Rc::new(Variable::Null)),
        }
    }
}

// date_add(timestamp, amount, unit) -> number
define_function!(
    DateAddFn,
    vec![
        ArgumentType::Number,
        ArgumentType::Number,
        ArgumentType::String
    ],
    None
);

impl Function for DateAddFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let ts = args[0].as_number().unwrap();
        let amount = args[1].as_number().unwrap();
        let unit = args[2].as_string().unwrap();

        let duration = match unit.to_lowercase().as_str() {
            "seconds" | "second" | "s" => TimeDelta::seconds(amount as i64),
            "minutes" | "minute" | "m" => TimeDelta::minutes(amount as i64),
            "hours" | "hour" | "h" => TimeDelta::hours(amount as i64),
            "days" | "day" | "d" => TimeDelta::days(amount as i64),
            "weeks" | "week" | "w" => TimeDelta::weeks(amount as i64),
            _ => return Err(custom_error(ctx, &format!("invalid time unit: {}", unit))),
        };

        let dt = Utc.timestamp_opt(ts as i64, 0);
        match dt {
            chrono::LocalResult::Single(dt) => {
                let new_dt = dt + duration;
                Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(new_dt.timestamp() as f64).unwrap(),
                )))
            }
            _ => Ok(Rc::new(Variable::Null)),
        }
    }
}

// date_diff(ts1, ts2, unit) -> number
define_function!(
    DateDiffFn,
    vec![
        ArgumentType::Number,
        ArgumentType::Number,
        ArgumentType::String
    ],
    None
);

impl Function for DateDiffFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let ts1 = args[0].as_number().unwrap();
        let ts2 = args[1].as_number().unwrap();
        let unit = args[2].as_string().unwrap();

        let diff_seconds = (ts1 - ts2) as i64;

        let result = match unit.to_lowercase().as_str() {
            "seconds" | "second" | "s" => diff_seconds as f64,
            "minutes" | "minute" | "m" => diff_seconds as f64 / 60.0,
            "hours" | "hour" | "h" => diff_seconds as f64 / 3600.0,
            "days" | "day" | "d" => diff_seconds as f64 / 86400.0,
            "weeks" | "week" | "w" => diff_seconds as f64 / 604800.0,
            _ => return Err(custom_error(ctx, &format!("invalid time unit: {}", unit))),
        };

        Ok(Rc::new(Variable::Number(
            serde_json::Number::from_f64(result).unwrap(),
        )))
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
    fn test_now() {
        let runtime = setup();
        let expr = runtime.compile("now()").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let ts = result.as_number().unwrap();
        // Should be a reasonable timestamp (after 2020)
        assert!(ts > 1577836800.0);
    }

    #[test]
    fn test_now_millis() {
        let runtime = setup();
        let expr = runtime.compile("now_millis()").unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        let ts = result.as_number().unwrap();
        // Should be a reasonable timestamp in millis (after 2020)
        assert!(ts > 1577836800000.0);
    }

    #[test]
    fn test_format_date() {
        let runtime = setup();
        // 1720000000 = 2024-07-03T10:26:40Z
        let expr = runtime
            .compile("format_date(`1720000000`, '%Y-%m-%d')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "2024-07-03");
    }

    #[test]
    fn test_format_date_with_time() {
        let runtime = setup();
        // Use a known timestamp and verify output format
        let expr = runtime
            .compile("format_date(`0`, '%Y-%m-%dT%H:%M:%S')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_string().unwrap(), "1970-01-01T00:00:00");
    }

    #[test]
    fn test_parse_date_iso() {
        let runtime = setup();
        let data = Variable::String("1970-01-01T00:00:00Z".to_string());
        let expr = runtime.compile("parse_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);
    }

    #[test]
    fn test_parse_date_date_only() {
        let runtime = setup();
        let data = Variable::String("2024-07-03".to_string());
        let expr = runtime.compile("parse_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        // Should parse as midnight UTC
        assert_eq!(result.as_number().unwrap(), 1719964800.0);
    }

    #[test]
    fn test_parse_date_with_format() {
        let runtime = setup();
        // Use datetime format for custom parsing
        let data = Variable::String("03/07/2024 00:00:00".to_string());
        let expr = runtime
            .compile("parse_date(@, '%d/%m/%Y %H:%M:%S')")
            .unwrap();
        let result = expr.search(&data).unwrap();
        // Should parse as 2024-07-03 midnight UTC
        assert_eq!(result.as_number().unwrap(), 1719964800.0);
    }

    #[test]
    fn test_parse_date_invalid() {
        let runtime = setup();
        let data = Variable::String("not a date".to_string());
        let expr = runtime.compile("parse_date(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_date_add_days() {
        let runtime = setup();
        // Add 7 days to 1720000000
        let expr = runtime
            .compile("date_add(`1720000000`, `7`, 'days')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1720604800.0);
    }

    #[test]
    fn test_date_add_hours() {
        let runtime = setup();
        let expr = runtime
            .compile("date_add(`1720000000`, `24`, 'hours')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1720086400.0);
    }

    #[test]
    fn test_date_add_negative() {
        let runtime = setup();
        // Subtract 1 day
        let expr = runtime
            .compile("date_add(`1720000000`, `-1`, 'day')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 1719913600.0);
    }

    #[test]
    fn test_date_diff_days() {
        let runtime = setup();
        // 7 days apart
        let expr = runtime
            .compile("date_diff(`1720604800`, `1720000000`, 'days')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 7.0);
    }

    #[test]
    fn test_date_diff_hours() {
        let runtime = setup();
        let expr = runtime
            .compile("date_diff(`1720086400`, `1720000000`, 'hours')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), 24.0);
    }

    #[test]
    fn test_date_diff_negative() {
        let runtime = setup();
        // Earlier timestamp first
        let expr = runtime
            .compile("date_diff(`1720000000`, `1720604800`, 'days')")
            .unwrap();
        let result = expr.search(&Variable::Null).unwrap();
        assert_eq!(result.as_number().unwrap(), -7.0);
    }

    #[test]
    fn test_date_add_invalid_unit() {
        let runtime = setup();
        let expr = runtime
            .compile("date_add(`1720000000`, `1`, 'invalid')")
            .unwrap();
        let result = expr.search(&Variable::Null);
        assert!(result.is_err());
    }
}
