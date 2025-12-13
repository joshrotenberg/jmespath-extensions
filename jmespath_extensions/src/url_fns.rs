//! URL parsing and manipulation functions.
//!
//! This module provides url_fns functions for JMESPath queries.
//!
//! For complete function reference with signatures and examples, see the
//! [`functions`](crate::functions) module documentation or use `jpx --list-category url_fns`.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::url_fns;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! url_fns::register(&mut runtime);
//! ```

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Variable,
};
use crate::define_function;

/// Register all URL functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("url_encode", Box::new(UrlEncodeFn::new()));
    runtime.register_function("url_decode", Box::new(UrlDecodeFn::new()));
    runtime.register_function("url_parse", Box::new(UrlParseFn::new()));
}

// =============================================================================
// url_encode(string) -> string
// =============================================================================

define_function!(UrlEncodeFn, vec![ArgumentType::String], None);

impl Function for UrlEncodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        let encoded = urlencoding::encode(input);
        Ok(Rc::new(Variable::String(encoded.into_owned())))
    }
}

// =============================================================================
// url_decode(string) -> string
// =============================================================================

define_function!(UrlDecodeFn, vec![ArgumentType::String], None);

impl Function for UrlDecodeFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match urlencoding::decode(input) {
            Ok(decoded) => Ok(Rc::new(Variable::String(decoded.into_owned()))),
            Err(_) => Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Invalid URL-encoded input".to_owned()),
            )),
        }
    }
}

// =============================================================================
// url_parse(string) -> object (parse URL into components)
// =============================================================================

define_function!(UrlParseFn, vec![ArgumentType::String], None);

impl Function for UrlParseFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let input = args[0].as_string().ok_or_else(|| {
            JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Expected string argument".to_owned()),
            )
        })?;

        match url::Url::parse(input) {
            Ok(parsed) => {
                let mut result: BTreeMap<String, Rcvar> = BTreeMap::new();

                result.insert(
                    "scheme".to_string(),
                    Rc::new(Variable::String(parsed.scheme().to_string())),
                );

                if let Some(host) = parsed.host_str() {
                    result.insert(
                        "host".to_string(),
                        Rc::new(Variable::String(host.to_string())),
                    );
                } else {
                    result.insert("host".to_string(), Rc::new(Variable::Null));
                }

                if let Some(port) = parsed.port() {
                    result.insert(
                        "port".to_string(),
                        Rc::new(Variable::Number(serde_json::Number::from(port))),
                    );
                } else {
                    result.insert("port".to_string(), Rc::new(Variable::Null));
                }

                result.insert(
                    "path".to_string(),
                    Rc::new(Variable::String(parsed.path().to_string())),
                );

                if let Some(query) = parsed.query() {
                    result.insert(
                        "query".to_string(),
                        Rc::new(Variable::String(query.to_string())),
                    );
                } else {
                    result.insert("query".to_string(), Rc::new(Variable::Null));
                }

                if let Some(fragment) = parsed.fragment() {
                    result.insert(
                        "fragment".to_string(),
                        Rc::new(Variable::String(fragment.to_string())),
                    );
                } else {
                    result.insert("fragment".to_string(), Rc::new(Variable::Null));
                }

                if !parsed.username().is_empty() {
                    result.insert(
                        "username".to_string(),
                        Rc::new(Variable::String(parsed.username().to_string())),
                    );
                }

                if let Some(password) = parsed.password() {
                    result.insert(
                        "password".to_string(),
                        Rc::new(Variable::String(password.to_string())),
                    );
                }

                // Add origin field (scheme + host + port)
                let origin = parsed.origin().ascii_serialization();
                result.insert("origin".to_string(), Rc::new(Variable::String(origin)));

                Ok(Rc::new(Variable::Object(result)))
            }
            // Return null for invalid URLs instead of an error
            Err(_) => Ok(Rc::new(Variable::Null)),
        }
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
    fn test_url_encode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_encode(@)").unwrap();
        let data = Variable::String("hello world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello%20world");
    }

    #[test]
    fn test_url_decode() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_decode(@)").unwrap();
        let data = Variable::String("hello%20world".to_string());
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_url_parse() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_parse(@)").unwrap();
        let data = Variable::String("https://example.com:8080/path?query=1#frag".to_string());
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("scheme").unwrap().as_string().unwrap(), "https");
        assert_eq!(obj.get("host").unwrap().as_string().unwrap(), "example.com");
        assert_eq!(obj.get("port").unwrap().as_number().unwrap() as u16, 8080);
    }

    #[test]
    fn test_url_parse_origin() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_parse(@)").unwrap();
        let data = Variable::String("https://example.com:8080/path".to_string());
        let result = expr.search(&data).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("origin").unwrap().as_string().unwrap(),
            "https://example.com:8080"
        );
    }

    #[test]
    fn test_url_parse_invalid_returns_null() {
        let runtime = setup_runtime();
        let expr = runtime.compile("url_parse(@)").unwrap();
        let data = Variable::String("not a valid url".to_string());
        let result = expr.search(&data).unwrap();
        assert!(result.is_null());
    }
}
