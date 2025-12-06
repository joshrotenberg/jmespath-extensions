//! Validation functions.
//!
//! These functions provide validation for common formats like email, URL, IP addresses.
//! Requires the `regex` feature for pattern matching.

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
