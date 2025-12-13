//! ID generation functions (nanoid, ulid).
//!
//! This module provides ids functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category ids`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::ids;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! ids::register(&mut runtime);
//! ```

use std::rc::Rc;

use crate::common::Function;
use crate::{ArgumentType, Context, JmespathError, Rcvar, Runtime, Signature, Variable};

/// Register all ID functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("nanoid", Box::new(NanoidFn::new()));
    runtime.register_function("ulid", Box::new(UlidFn::new()));
    runtime.register_function("ulid_timestamp", Box::new(UlidTimestampFn::new()));
}

// =============================================================================
// nanoid(size?) -> string
// =============================================================================

pub struct NanoidFn {
    signature: Signature,
}

impl Default for NanoidFn {
    fn default() -> Self {
        Self::new()
    }
}

impl NanoidFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![], Some(ArgumentType::Number)),
        }
    }
}

impl Function for NanoidFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let id = if args.is_empty() {
            nanoid::nanoid!()
        } else {
            let size = args[0].as_number().unwrap_or(21.0) as usize;
            nanoid::nanoid!(size)
        };

        Ok(Rc::new(Variable::String(id)))
    }
}

// =============================================================================
// ulid() -> string
// =============================================================================

pub struct UlidFn {
    signature: Signature,
}

impl Default for UlidFn {
    fn default() -> Self {
        Self::new()
    }
}

impl UlidFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![], None),
        }
    }
}

impl Function for UlidFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let id = ulid::Ulid::new().to_string();
        Ok(Rc::new(Variable::String(id)))
    }
}

// =============================================================================
// ulid_timestamp(ulid) -> number (unix ms)
// =============================================================================

pub struct UlidTimestampFn {
    signature: Signature,
}

impl Default for UlidTimestampFn {
    fn default() -> Self {
        Self::new()
    }
}

impl UlidTimestampFn {
    pub fn new() -> Self {
        Self {
            signature: Signature::new(vec![ArgumentType::String], None),
        }
    }
}

impl Function for UlidTimestampFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;
        let ulid_str = args[0].as_string().unwrap();

        match ulid::Ulid::from_string(ulid_str) {
            Ok(id) => {
                let ts = id.timestamp_ms();
                Ok(Rc::new(Variable::Number(
                    serde_json::Number::from_f64(ts as f64).unwrap(),
                )))
            }
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
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
    fn test_nanoid_default() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("nanoid()").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_string().unwrap();
        assert_eq!(id.len(), 21);
    }

    #[test]
    fn test_nanoid_custom_size() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("nanoid(`10`)").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_string().unwrap();
        assert_eq!(id.len(), 10);
    }

    #[test]
    fn test_nanoid_unique() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("nanoid()").unwrap();
        let id1 = expr.search(&data).unwrap();
        let id2 = expr.search(&data).unwrap();
        assert_ne!(id1.as_string().unwrap(), id2.as_string().unwrap());
    }

    #[test]
    fn test_ulid() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("ulid()").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_string().unwrap();
        // ULID is 26 characters
        assert_eq!(id.len(), 26);
    }

    #[test]
    fn test_ulid_unique() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("ulid()").unwrap();
        let id1 = expr.search(&data).unwrap();
        let id2 = expr.search(&data).unwrap();
        assert_ne!(id1.as_string().unwrap(), id2.as_string().unwrap());
    }

    #[test]
    fn test_ulid_timestamp() {
        let runtime = setup();
        // Generate a ULID and extract its timestamp
        let ulid = ulid::Ulid::new();
        let ulid_str = ulid.to_string();
        let expected_ts = ulid.timestamp_ms();

        let data = Variable::String(ulid_str);
        let expr = runtime.compile("ulid_timestamp(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), expected_ts as f64);
    }

    #[test]
    fn test_ulid_timestamp_invalid() {
        let runtime = setup();
        let data = Variable::from_json(r#""not-a-ulid""#).unwrap();
        let expr = runtime.compile("ulid_timestamp(@)").unwrap();
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }

    #[test]
    fn test_ulid_format() {
        let runtime = setup();
        let data = Variable::Null;
        let expr = runtime.compile("ulid()").unwrap();
        let result = expr.search(&data).unwrap();
        let id = result.as_string().unwrap();

        // ULID should be 26 characters of Crockford's Base32
        assert_eq!(id.len(), 26);
        // All characters should be valid Base32
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
