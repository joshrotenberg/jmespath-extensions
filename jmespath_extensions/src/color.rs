//! Color manipulation functions.
//!
//! This module provides color functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category color`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::color;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! color::register(&mut runtime);
//! ```

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Variable, rcvar,
};
use crate::define_function;
use std::collections::BTreeMap;

define_function!(HexToRgbFn, vec![ArgumentType::String], None);

impl Function for HexToRgbFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        match parse_hex_color(hex) {
            Some((r, g, b)) => {
                let mut map = BTreeMap::new();
                map.insert(
                    "r".to_string(),
                    rcvar(Variable::Number(serde_json::Number::from(r))),
                );
                map.insert(
                    "g".to_string(),
                    rcvar(Variable::Number(serde_json::Number::from(g))),
                );
                map.insert(
                    "b".to_string(),
                    rcvar(Variable::Number(serde_json::Number::from(b))),
                );
                Ok(rcvar(Variable::Object(map)))
            }
            None => Ok(rcvar(Variable::Null)),
        }
    }
}

define_function!(
    RgbToHexFn,
    vec![
        ArgumentType::Number,
        ArgumentType::Number,
        ArgumentType::Number
    ],
    None
);

impl Function for RgbToHexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let r = args[0].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for r".to_owned()),
            )
        })? as u8;

        let g = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for g".to_owned()),
            )
        })? as u8;

        let b = args[2].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number for b".to_owned()),
            )
        })? as u8;

        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(rcvar(Variable::String(hex)))
    }
}

define_function!(
    LightenFn,
    vec![ArgumentType::String, ArgumentType::Number],
    None
);

impl Function for LightenFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let amount = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        let factor = (amount / 100.0).clamp(0.0, 1.0);
        let r = (r as f64 + (255.0 - r as f64) * factor).round() as u8;
        let g = (g as f64 + (255.0 - g as f64) * factor).round() as u8;
        let b = (b as f64 + (255.0 - b as f64) * factor).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(rcvar(Variable::String(result)))
    }
}

define_function!(
    DarkenFn,
    vec![ArgumentType::String, ArgumentType::Number],
    None
);

impl Function for DarkenFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let amount = args[1].as_number().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected number".to_owned()),
            )
        })?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        let factor = 1.0 - (amount / 100.0).clamp(0.0, 1.0);
        let r = (r as f64 * factor).round() as u8;
        let g = (g as f64 * factor).round() as u8;
        let b = (b as f64 * factor).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(rcvar(Variable::String(result)))
    }
}

define_function!(
    ColorMixFn,
    vec![ArgumentType::String, ArgumentType::String],
    Some(ArgumentType::Number)
);

impl Function for ColorMixFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex1 = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let hex2 = args[1].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let weight = if args.len() > 2 {
            args[2].as_number().unwrap_or(0.5)
        } else {
            0.5
        };

        let (r1, g1, b1) = match parse_hex_color(hex1) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        let (r2, g2, b2) = match parse_hex_color(hex2) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        let w = weight.clamp(0.0, 1.0);
        let r = (r1 as f64 * (1.0 - w) + r2 as f64 * w).round() as u8;
        let g = (g1 as f64 * (1.0 - w) + g2 as f64 * w).round() as u8;
        let b = (b1 as f64 * (1.0 - w) + b2 as f64 * w).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(rcvar(Variable::String(result)))
    }
}

define_function!(ColorInvertFn, vec![ArgumentType::String], None);

impl Function for ColorInvertFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        let result = format!("#{:02x}{:02x}{:02x}", 255 - r, 255 - g, 255 - b);
        Ok(rcvar(Variable::String(result)))
    }
}

define_function!(ColorGrayscaleFn, vec![ArgumentType::String], None);

impl Function for ColorGrayscaleFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        // Use luminance formula: 0.299*R + 0.587*G + 0.114*B
        let gray = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64).round() as u8;

        let result = format!("#{:02x}{:02x}{:02x}", gray, gray, gray);
        Ok(rcvar(Variable::String(result)))
    }
}

define_function!(ColorComplementFn, vec![ArgumentType::String], None);

impl Function for ColorComplementFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let hex = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string".to_owned()),
            )
        })?;

        let (r, g, b) = match parse_hex_color(hex) {
            Some(rgb) => rgb,
            None => return Ok(rcvar(Variable::Null)),
        };

        // Convert to HSL, rotate hue by 180, convert back
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let new_h = (h + 180.0) % 360.0;
        let (r, g, b) = hsl_to_rgb(new_h, s, l);

        let result = format!("#{:02x}{:02x}{:02x}", r, g, b);
        Ok(rcvar(Variable::String(result)))
    }
}

// Helper functions

/// Parse a hex color string into RGB components.
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_start_matches('#');

    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

/// Convert RGB to HSL.
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < f64::EPSILON {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() < f64::EPSILON {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h * 360.0, s, l)
}

/// Convert HSL to RGB.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let h = h / 360.0;

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Register all color functions with the runtime.
pub fn register(runtime: &mut crate::Runtime) {
    runtime.register_function("hex_to_rgb", Box::new(HexToRgbFn::new()));
    runtime.register_function("rgb_to_hex", Box::new(RgbToHexFn::new()));
    runtime.register_function("lighten", Box::new(LightenFn::new()));
    runtime.register_function("darken", Box::new(DarkenFn::new()));
    runtime.register_function("color_mix", Box::new(ColorMixFn::new()));
    runtime.register_function("color_invert", Box::new(ColorInvertFn::new()));
    runtime.register_function("color_grayscale", Box::new(ColorGrayscaleFn::new()));
    runtime.register_function("color_complement", Box::new(ColorComplementFn::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff5500"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("ff5500"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("#f50"), Some((255, 85, 0)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_rgb_to_hsl_roundtrip() {
        let colors = [
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (128, 128, 128),
            (255, 128, 64),
        ];
        for (r, g, b) in colors {
            let (h, s, l) = rgb_to_hsl(r, g, b);
            let (r2, g2, b2) = hsl_to_rgb(h, s, l);
            assert!(
                (r as i16 - r2 as i16).abs() <= 1,
                "Red mismatch: {} vs {}",
                r,
                r2
            );
            assert!(
                (g as i16 - g2 as i16).abs() <= 1,
                "Green mismatch: {} vs {}",
                g,
                g2
            );
            assert!(
                (b as i16 - b2 as i16).abs() <= 1,
                "Blue mismatch: {} vs {}",
                b,
                b2
            );
        }
    }
}
