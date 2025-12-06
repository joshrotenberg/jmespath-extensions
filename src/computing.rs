//! Computing utility functions.
//!
//! This module provides functions for working with bytes, bitwise operations, and other computing utilities.
//!
//! # Feature
//!
//! This module requires the `computing` feature flag (no external dependencies).
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `parse_bytes(string)` | Parse byte string to number ("1.5 GB" → 1500000000) |
//! | `format_bytes(number)` | Format bytes as human-readable (1500000000 → "1.5 GB") |
//! | `format_bytes_binary(number)` | Format using binary units (1073741824 → "1 GiB") |
//! | `bit_and(a, b)` | Bitwise AND |
//! | `bit_or(a, b)` | Bitwise OR |
//! | `bit_xor(a, b)` | Bitwise XOR |
//! | `bit_not(a)` | Bitwise NOT |
//! | `bit_shift_left(a, n)` | Left shift |
//! | `bit_shift_right(a, n)` | Right shift |
//!
//! # Example
//!
//! ```
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::register_all;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! register_all(&mut runtime);
//!
//! // Parse byte string
//! let expr = runtime.compile("parse_bytes('1.5 GB')").unwrap();
//! let result = expr.search(&Variable::Null).unwrap();
//! assert_eq!(result.as_number().unwrap() as i64, 1_500_000_000);
//!
//! // Format bytes
//! let expr = runtime.compile("format_bytes(`1500000000`)").unwrap();
//! let result = expr.search(&Variable::Null).unwrap();
//! assert_eq!(result.as_string().unwrap(), "1.5 GB");
//! ```

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Variable, rcvar,
};
use crate::define_function;

// Decimal units (SI): KB, MB, GB, TB, PB
const DECIMAL_UNITS: &[(&str, f64)] = &[
    ("PB", 1e15),
    ("TB", 1e12),
    ("GB", 1e9),
    ("MB", 1e6),
    ("KB", 1e3),
    ("B", 1.0),
];

// Binary units (IEC): KiB, MiB, GiB, TiB, PiB
const BINARY_UNITS: &[(&str, f64)] = &[
    ("PiB", 1125899906842624.0),
    ("TiB", 1099511627776.0),
    ("GiB", 1073741824.0),
    ("MiB", 1048576.0),
    ("KiB", 1024.0),
    ("B", 1.0),
];

define_function!(ParseBytesFn, vec![ArgumentType::String], None);

impl Function for ParseBytesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        match parse_bytes_str(s) {
            Some(bytes) => Ok(rcvar(Variable::Number(
                serde_json::Number::from_f64(bytes).unwrap(),
            ))),
            None => Ok(rcvar(Variable::Null)),
        }
    }
}

define_function!(FormatBytesFn, vec![ArgumentType::Number], None);

impl Function for FormatBytesFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let bytes = num;
        let formatted = format_bytes_with_units(bytes, DECIMAL_UNITS);

        Ok(rcvar(Variable::String(formatted)))
    }
}

define_function!(FormatBytesBinaryFn, vec![ArgumentType::Number], None);

impl Function for FormatBytesBinaryFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let num = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let bytes = num;
        let formatted = format_bytes_with_units(bytes, BINARY_UNITS);

        Ok(rcvar(Variable::String(formatted)))
    }
}

define_function!(
    BitAndFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for BitAndFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        Ok(rcvar(Variable::Number(serde_json::Number::from(a & b))))
    }
}

define_function!(
    BitOrFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for BitOrFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        Ok(rcvar(Variable::Number(serde_json::Number::from(a | b))))
    }
}

define_function!(
    BitXorFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for BitXorFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        let b = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        Ok(rcvar(Variable::Number(serde_json::Number::from(a ^ b))))
    }
}

define_function!(BitNotFn, vec![ArgumentType::Number], None);

impl Function for BitNotFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        Ok(rcvar(Variable::Number(serde_json::Number::from(!a))))
    }
}

define_function!(
    BitShiftLeftFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for BitShiftLeftFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        let n = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected non-negative integer".to_owned()),
            )
        })? as u32;

        Ok(rcvar(Variable::Number(serde_json::Number::from(a << n))))
    }
}

define_function!(
    BitShiftRightFn,
    vec![ArgumentType::Number, ArgumentType::Number],
    None
);

impl Function for BitShiftRightFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let a = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected integer".to_owned()),
            )
        })? as i64;

        let n = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected non-negative integer".to_owned()),
            )
        })? as u32;

        Ok(rcvar(Variable::Number(serde_json::Number::from(a >> n))))
    }
}

// Helper functions

/// Parse a byte string like "1.5 GB" or "100 MiB" into bytes.
fn parse_bytes_str(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut num_end = 0;
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+' {
            num_end = i + 1;
        } else if !c.is_whitespace() {
            break;
        }
    }

    if num_end == 0 {
        return None;
    }

    let num_str: String = chars[..num_end].iter().collect();
    let num: f64 = num_str.trim().parse().ok()?;

    let unit: String = chars[num_end..].iter().collect();
    let unit = unit.trim().to_uppercase();

    if unit.is_empty() || unit == "B" || unit == "BYTES" || unit == "BYTE" {
        return Some(num);
    }

    let multiplier = match unit.as_str() {
        "PIB" | "PEBIBYTE" | "PEBIBYTES" => 1125899906842624.0,
        "TIB" | "TEBIBYTE" | "TEBIBYTES" => 1099511627776.0,
        "GIB" | "GIBIBYTE" | "GIBIBYTES" => 1073741824.0,
        "MIB" | "MEBIBYTE" | "MEBIBYTES" => 1048576.0,
        "KIB" | "KIBIBYTE" | "KIBIBYTES" => 1024.0,
        "PB" | "PETABYTE" | "PETABYTES" => 1e15,
        "TB" | "TERABYTE" | "TERABYTES" => 1e12,
        "GB" | "GIGABYTE" | "GIGABYTES" => 1e9,
        "MB" | "MEGABYTE" | "MEGABYTES" => 1e6,
        "KB" | "KILOBYTE" | "KILOBYTES" => 1e3,
        _ => return None,
    };

    Some(num * multiplier)
}

/// Format bytes with the given unit system.
fn format_bytes_with_units(bytes: f64, units: &[(&str, f64)]) -> String {
    if bytes == 0.0 {
        return "0 B".to_string();
    }

    let abs_bytes = bytes.abs();

    for (unit, threshold) in units {
        if abs_bytes >= *threshold {
            let value = bytes / threshold;
            let formatted = format!("{:.2}", value);
            let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
            return format!("{} {}", formatted, unit);
        }
    }

    format!("{} B", bytes)
}

/// Register all computing functions with the runtime.
pub fn register(runtime: &mut crate::Runtime) {
    runtime.register_function("parse_bytes", Box::new(ParseBytesFn::new()));
    runtime.register_function("format_bytes", Box::new(FormatBytesFn::new()));
    runtime.register_function("format_bytes_binary", Box::new(FormatBytesBinaryFn::new()));
    runtime.register_function("bit_and", Box::new(BitAndFn::new()));
    runtime.register_function("bit_or", Box::new(BitOrFn::new()));
    runtime.register_function("bit_xor", Box::new(BitXorFn::new()));
    runtime.register_function("bit_not", Box::new(BitNotFn::new()));
    runtime.register_function("bit_shift_left", Box::new(BitShiftLeftFn::new()));
    runtime.register_function("bit_shift_right", Box::new(BitShiftRightFn::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bytes() {
        assert_eq!(parse_bytes_str("100"), Some(100.0));
        assert_eq!(parse_bytes_str("100 B"), Some(100.0));
        assert_eq!(parse_bytes_str("1 KB"), Some(1000.0));
        assert_eq!(parse_bytes_str("1.5 KB"), Some(1500.0));
        assert_eq!(parse_bytes_str("1 MB"), Some(1_000_000.0));
        assert_eq!(parse_bytes_str("1.5 GB"), Some(1_500_000_000.0));
        assert_eq!(parse_bytes_str("1 TB"), Some(1_000_000_000_000.0));
        assert_eq!(parse_bytes_str("1 KiB"), Some(1024.0));
        assert_eq!(parse_bytes_str("1 MiB"), Some(1_048_576.0));
        assert_eq!(parse_bytes_str("1 GiB"), Some(1_073_741_824.0));
        assert_eq!(parse_bytes_str("1 gb"), Some(1_000_000_000.0));
        assert_eq!(parse_bytes_str(""), None);
        assert_eq!(parse_bytes_str("invalid"), None);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes_with_units(0.0, DECIMAL_UNITS), "0 B");
        assert_eq!(format_bytes_with_units(500.0, DECIMAL_UNITS), "500 B");
        assert_eq!(format_bytes_with_units(1000.0, DECIMAL_UNITS), "1 KB");
        assert_eq!(format_bytes_with_units(1500.0, DECIMAL_UNITS), "1.5 KB");
        assert_eq!(
            format_bytes_with_units(1_500_000_000.0, DECIMAL_UNITS),
            "1.5 GB"
        );
    }

    #[test]
    fn test_format_bytes_binary() {
        assert_eq!(format_bytes_with_units(1024.0, BINARY_UNITS), "1 KiB");
        assert_eq!(format_bytes_with_units(1536.0, BINARY_UNITS), "1.5 KiB");
        assert_eq!(
            format_bytes_with_units(1_073_741_824.0, BINARY_UNITS),
            "1 GiB"
        );
    }
}
