//! Data validation functions.
//!
//! This module provides validation functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category validation`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::validation;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! validation::register(&mut runtime);
//! ```

use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

#[cfg(feature = "regex")]
use regex::Regex;

/// Register all validation functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    #[cfg(feature = "regex")]
    {
        runtime.register_function("is_email", Box::new(IsEmailFn::new()));
        runtime.register_function("is_url", Box::new(IsUrlFn::new()));
        runtime.register_function("is_uuid", Box::new(IsUuidFn::new()));
        runtime.register_function("is_phone", Box::new(IsPhoneFn::new()));
    }
    runtime.register_function("is_ipv4", Box::new(IsIpv4Fn::new()));
    runtime.register_function("is_ipv6", Box::new(IsIpv6Fn::new()));
    runtime.register_function("luhn_check", Box::new(LuhnCheckFn::new()));
    runtime.register_function("is_credit_card", Box::new(IsCreditCardFn::new()));
    runtime.register_function("is_jwt", Box::new(IsJwtFn::new()));
    runtime.register_function("is_iso_date", Box::new(IsIsoDateFn::new()));
    runtime.register_function("is_json", Box::new(IsJsonFn::new()));
    runtime.register_function("is_base64", Box::new(IsBase64Fn::new()));
    runtime.register_function("is_hex", Box::new(IsHexFn::new()));
}

// =============================================================================
// is_email(string) -> boolean
// =============================================================================

#[cfg(feature = "regex")]
define_function!(IsEmailFn, vec![ArgumentType::String], None);

#[cfg(feature = "regex")]
impl Function for IsEmailFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        Ok(Rc::new(Variable::Bool(email_re.is_match(s))))
    }
}

// =============================================================================
// is_url(string) -> boolean
// =============================================================================

#[cfg(feature = "regex")]
define_function!(IsUrlFn, vec![ArgumentType::String], None);

#[cfg(feature = "regex")]
impl Function for IsUrlFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let url_re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        Ok(Rc::new(Variable::Bool(url_re.is_match(s))))
    }
}

// =============================================================================
// is_uuid(string) -> boolean
// =============================================================================

#[cfg(feature = "regex")]
define_function!(IsUuidFn, vec![ArgumentType::String], None);

#[cfg(feature = "regex")]
impl Function for IsUuidFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let uuid_re = Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
        )
        .unwrap();
        Ok(Rc::new(Variable::Bool(uuid_re.is_match(s))))
    }
}

// =============================================================================
// is_ipv4(string) -> boolean
// =============================================================================

define_function!(IsIpv4Fn, vec![ArgumentType::String], None);

impl Function for IsIpv4Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let is_valid = s.parse::<std::net::Ipv4Addr>().is_ok();
        Ok(Rc::new(Variable::Bool(is_valid)))
    }
}

// =============================================================================
// is_ipv6(string) -> boolean
// =============================================================================

define_function!(IsIpv6Fn, vec![ArgumentType::String], None);

impl Function for IsIpv6Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let is_valid = s.parse::<std::net::Ipv6Addr>().is_ok();
        Ok(Rc::new(Variable::Bool(is_valid)))
    }
}

// =============================================================================
// luhn_check(string) -> boolean - Generic Luhn algorithm check
// =============================================================================

define_function!(LuhnCheckFn, vec![ArgumentType::String], None);

impl Function for LuhnCheckFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        Ok(Rc::new(Variable::Bool(luhn_validate(s))))
    }
}

fn luhn_validate(s: &str) -> bool {
    // Remove spaces and dashes
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for c in digits.chars().rev() {
        if let Some(digit) = c.to_digit(10) {
            let mut d = digit;
            if double {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            sum += d;
            double = !double;
        } else {
            return false;
        }
    }

    sum % 10 == 0
}

// =============================================================================
// is_credit_card(string) -> boolean - Validate credit card number
// =============================================================================

define_function!(IsCreditCardFn, vec![ArgumentType::String], None);

impl Function for IsCreditCardFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Remove spaces and dashes
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();

        // Credit cards are typically 13-19 digits
        if digits.len() < 13 || digits.len() > 19 {
            return Ok(Rc::new(Variable::Bool(false)));
        }

        // Must pass Luhn check
        Ok(Rc::new(Variable::Bool(luhn_validate(&digits))))
    }
}

// =============================================================================
// is_phone(string) -> boolean - Validate phone number format
// =============================================================================

#[cfg(feature = "regex")]
define_function!(IsPhoneFn, vec![ArgumentType::String], None);

#[cfg(feature = "regex")]
impl Function for IsPhoneFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Basic phone pattern: optional + followed by digits, spaces, dashes, parens
        // Minimum 7 digits for a valid phone number
        let phone_re = Regex::new(r"^\+?[\d\s\-\(\)\.]{7,}$").unwrap();
        if !phone_re.is_match(s) {
            return Ok(Rc::new(Variable::Bool(false)));
        }

        // Count actual digits - need at least 7
        let digit_count = s.chars().filter(|c| c.is_ascii_digit()).count();
        Ok(Rc::new(Variable::Bool((7..=15).contains(&digit_count))))
    }
}

// =============================================================================
// is_jwt(string) -> boolean - Check if valid JWT structure
// =============================================================================

define_function!(IsJwtFn, vec![ArgumentType::String], None);

impl Function for IsJwtFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // JWT has 3 base64url-encoded parts separated by dots
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Ok(Rc::new(Variable::Bool(false)));
        }

        // Check each part is valid base64url (alphanumeric, -, _, no padding required)
        let is_valid = parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=')
        });

        Ok(Rc::new(Variable::Bool(is_valid)))
    }
}

// =============================================================================
// is_iso_date(string) -> boolean - Validate ISO 8601 date format
// =============================================================================

define_function!(IsIsoDateFn, vec![ArgumentType::String], None);

impl Function for IsIsoDateFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Try parsing as RFC3339 (subset of ISO 8601)
        if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
            return Ok(Rc::new(Variable::Bool(true)));
        }

        // Try parsing as date only (YYYY-MM-DD)
        if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
            return Ok(Rc::new(Variable::Bool(true)));
        }

        // Try parsing as datetime without timezone
        if chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok() {
            return Ok(Rc::new(Variable::Bool(true)));
        }

        Ok(Rc::new(Variable::Bool(false)))
    }
}

// =============================================================================
// is_json(string) -> boolean - Check if string is valid JSON
// =============================================================================

define_function!(IsJsonFn, vec![ArgumentType::String], None);

impl Function for IsJsonFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let is_valid = serde_json::from_str::<serde_json::Value>(s).is_ok();
        Ok(Rc::new(Variable::Bool(is_valid)))
    }
}

// =============================================================================
// is_base64(string) -> boolean - Check if valid Base64 encoding
// =============================================================================

define_function!(IsBase64Fn, vec![ArgumentType::String], None);

impl Function for IsBase64Fn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        use base64::{Engine, engine::general_purpose::STANDARD};
        let is_valid = STANDARD.decode(s).is_ok();
        Ok(Rc::new(Variable::Bool(is_valid)))
    }
}

// =============================================================================
// is_hex(string) -> boolean - Check if valid hexadecimal string
// =============================================================================

define_function!(IsHexFn, vec![ArgumentType::String], None);

impl Function for IsHexFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let s = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        // Must be non-empty and all hex chars
        let is_valid = !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit());
        Ok(Rc::new(Variable::Bool(is_valid)))
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
    fn test_is_ipv4() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_ipv4(@)").unwrap();

        let data = Variable::String("192.168.1.1".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("not an ip".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_ipv6() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_ipv6(@)").unwrap();

        let data = Variable::String("::1".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("2001:db8::1".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_is_email() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_email(@)").unwrap();

        let data = Variable::String("test@example.com".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("not-an-email".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_luhn_check_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("luhn_check(@)").unwrap();

        // Valid Luhn number
        let data = Variable::String("79927398713".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_luhn_check_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("luhn_check(@)").unwrap();

        let data = Variable::String("79927398710".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_credit_card_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_credit_card(@)").unwrap();

        // Test Visa number (passes Luhn)
        let data = Variable::String("4111111111111111".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_credit_card_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_credit_card(@)").unwrap();

        // Invalid number
        let data = Variable::String("1234567890123456".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_credit_card_too_short() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_credit_card(@)").unwrap();

        let data = Variable::String("123456".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_is_phone_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_phone(@)").unwrap();

        let data = Variable::String("+1-555-123-4567".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("(555) 123-4567".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[cfg(feature = "regex")]
    #[test]
    fn test_is_phone_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_phone(@)").unwrap();

        let data = Variable::String("123".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_jwt_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_jwt(@)").unwrap();

        let data = Variable::String(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U".to_string()
        );
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_jwt_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_jwt(@)").unwrap();

        // Only two parts - invalid
        let data = Variable::String("only.twoparts".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Contains invalid characters for base64url
        let data = Variable::String("abc.def!ghi.jkl".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_iso_date_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_iso_date(@)").unwrap();

        let data = Variable::String("2023-12-13T15:30:00Z".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("2023-12-13".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_iso_date_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_iso_date(@)").unwrap();

        let data = Variable::String("12/13/2023".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_json_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_json(@)").unwrap();

        let data = Variable::String(r#"{"a": 1, "b": [2, 3]}"#.to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_json_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_json(@)").unwrap();

        let data = Variable::String("not json".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_base64_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_base64(@)").unwrap();

        let data = Variable::String("SGVsbG8gV29ybGQ=".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_base64_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_base64(@)").unwrap();

        let data = Variable::String("not valid base64!!!".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_hex_valid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_hex(@)").unwrap();

        let data = Variable::String("deadbeef".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());

        let data = Variable::String("ABCDEF0123456789".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_is_hex_invalid() {
        let runtime = setup_runtime();
        let expr = runtime.compile("is_hex(@)").unwrap();

        let data = Variable::String("not hex!".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());

        let data = Variable::String("".to_string());
        let result = expr.search(&data).unwrap();
        assert!(!result.as_boolean().unwrap());
    }
}
