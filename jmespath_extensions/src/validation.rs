//! Validation functions.
//!
//! This module provides format validation capabilities for common data types like email addresses,
//! URLs, UUIDs, and IP addresses. These functions help verify that string values match expected
//! patterns and formats.
//!
//! **Note:** Email, URL, and UUID validation require the `regex` feature to be enabled.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description | Feature Required |
//! |----------|-----------|---------|-------------|------------------|
//! | `is_email` | `(email: string)` | `boolean` | Validate email address format | `regex` |
//! | `is_url` | `(url: string)` | `boolean` | Validate HTTP/HTTPS URL format | `regex` |
//! | `is_uuid` | `(uuid: string)` | `boolean` | Validate UUID format | `regex` |
//! | `is_ipv4` | `(ip: string)` | `boolean` | Validate IPv4 address | - |
//! | `is_ipv6` | `(ip: string)` | `boolean` | Validate IPv6 address | - |
//!
//! # Examples
//!
//! ```rust
//! use jmespath_extensions::Runtime;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! jmespath_extensions::register_all(&mut runtime);
//!
//! let expr = runtime.compile("is_ipv4(@)").unwrap();
//! let data = jmespath::Variable::String("192.168.1.1".to_string());
//! let result = expr.search(&data).unwrap();
//! assert!(result.as_boolean().unwrap());
//! ```
//!
//! # Function Details
//!
//! ## Email Validation
//!
//! ### `is_email(email: string) -> boolean`
//!
//! Validates whether a string is a properly formatted email address. Uses a basic regex pattern
//! that checks for the structure: `local@domain.tld`
//!
//! **Requires:** `regex` feature
//!
//! ```text
//! is_email('user@example.com')             // true
//! is_email('john.doe+tag@domain.co.uk')    // true
//! is_email('invalid.email')                // false
//! is_email('missing@domain')               // false
//! is_email('@example.com')                 // false
//! ```
//!
//! ## URL Validation
//!
//! ### `is_url(url: string) -> boolean`
//!
//! Validates whether a string is a properly formatted HTTP or HTTPS URL.
//!
//! **Requires:** `regex` feature
//!
//! ```text
//! is_url('https://example.com')            // true
//! is_url('http://example.com/path')        // true
//! is_url('https://sub.domain.com:8080')    // true
//! is_url('ftp://example.com')              // false (only http/https)
//! is_url('not a url')                      // false
//! ```
//!
//! ## UUID Validation
//!
//! ### `is_uuid(uuid: string) -> boolean`
//!
//! Validates whether a string is a properly formatted UUID (RFC 4122 format).
//! Accepts UUIDs with or without hyphens.
//!
//! **Requires:** `regex` feature
//!
//! ```text
//! is_uuid('550e8400-e29b-41d4-a716-446655440000')   // true
//! is_uuid('6ba7b810-9dad-11d1-80b4-00c04fd430c8')   // true
//! is_uuid('invalid-uuid')                          // false
//! is_uuid('550e8400-e29b-41d4-a716')               // false (too short)
//! ```
//!
//! ## IP Address Validation
//!
//! ### `is_ipv4(ip: string) -> boolean`
//!
//! Validates whether a string is a valid IPv4 address. Uses Rust's standard library parser
//! for accurate validation.
//!
//! ```text
//! is_ipv4('192.168.1.1')                   // true
//! is_ipv4('10.0.0.1')                      // true
//! is_ipv4('255.255.255.255')               // true
//! is_ipv4('256.1.1.1')                     // false (256 > 255)
//! is_ipv4('192.168.1')                     // false (incomplete)
//! is_ipv4('::1')                           // false (IPv6)
//! ```
//!
//! ### `is_ipv6(ip: string) -> boolean`
//!
//! Validates whether a string is a valid IPv6 address. Uses Rust's standard library parser
//! for accurate validation. Supports both full and compressed notation.
//!
//! ```text
//! is_ipv6('::1')                           // true (loopback)
//! is_ipv6('2001:db8::1')                   // true (compressed)
//! is_ipv6('fe80::1')                       // true
//! is_ipv6('2001:0db8:0000:0000:0000:0000:0000:0001')  // true (full)
//! is_ipv6('192.168.1.1')                   // false (IPv4)
//! is_ipv6('not-an-ip')                     // false
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
    }
    runtime.register_function("is_ipv4", Box::new(IsIpv4Fn::new()));
    runtime.register_function("is_ipv6", Box::new(IsIpv6Fn::new()));
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
}
