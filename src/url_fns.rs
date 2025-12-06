//! URL encoding and parsing functions.
//!
//! This module provides URL manipulation capabilities for JMESPath expressions, including
//! URL percent-encoding/decoding and URL parsing into components. These functions are useful
//! for working with web addresses, query strings, and URL components.
//!
//! **Note:** This module requires the `url` feature to be enabled.
//!
//! # Function Reference
//!
//! | Function | Arguments | Returns | Description |
//! |----------|-----------|---------|-------------|
//! | `url_encode` | `(text: string)` | `string` | Percent-encode string for URLs |
//! | `url_decode` | `(encoded: string)` | `string` | Decode percent-encoded URL string |
//! | `url_parse` | `(url: string)` | `object` | Parse URL into components |
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
//! let expr = runtime.compile("url_encode(@)").unwrap();
//! let data = jmespath::Variable::String("hello world".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(), "hello%20world");
//! ```
//!
//! # Function Details
//!
//! ## URL Encoding
//!
//! ### `url_encode(text: string) -> string`
//!
//! Percent-encodes a string for safe use in URLs. Special characters are converted to
//! `%XX` format where XX is the hexadecimal value of the byte.
//!
//! ```text
//! url_encode('hello world')                // "hello%20world"
//! url_encode('name=John Doe')              // "name%3DJohn%20Doe"
//! url_encode('path/to/file')               // "path%2Fto%2Efile"
//! url_encode('email@example.com')          // "email%40example.com"
//! url_encode('100%')                       // "100%25"
//! ```
//!
//! ### `url_decode(encoded: string) -> string`
//!
//! Decodes a percent-encoded URL string back to its original form. Returns an error if
//! the encoding is invalid.
//!
//! ```text
//! url_decode('hello%20world')              // "hello world"
//! url_decode('name%3DJohn%20Doe')          // "name=John Doe"
//! url_decode('path%2Fto%2Efile')           // "path/to/file"
//! url_decode('email%40example.com')        // "email@example.com"
//! url_decode('100%25')                     // "100%"
//! ```
//!
//! ## URL Parsing
//!
//! ### `url_parse(url: string) -> object`
//!
//! Parses a URL string into its component parts and returns an object with the following fields:
//!
//! - `scheme`: Protocol (e.g., "http", "https", "ftp")
//! - `host`: Hostname or IP address (null if not present)
//! - `port`: Port number (null if not specified or default)
//! - `path`: URL path component
//! - `query`: Query string without the '?' (null if not present)
//! - `fragment`: Fragment identifier without the '#' (null if not present)
//! - `username`: Username for authentication (only if present)
//! - `password`: Password for authentication (only if present)
//!
//! Returns an error if the URL is malformed.
//!
//! ```text
//! url_parse('https://example.com/path')
//! // {
//! //   "scheme": "https",
//! //   "host": "example.com",
//! //   "port": null,
//! //   "path": "/path",
//! //   "query": null,
//! //   "fragment": null
//! // }
//!
//! url_parse('http://user:pass@example.com:8080/path?key=value#section')
//! // {
//! //   "scheme": "http",
//! //   "host": "example.com",
//! //   "port": 8080,
//! //   "path": "/path",
//! //   "query": "key=value",
//! //   "fragment": "section",
//! //   "username": "user",
//! //   "password": "pass"
//! // }
//!
//! url_parse('https://api.example.com/v1/users?limit=10&offset=20')
//! // {
//! //   "scheme": "https",
//! //   "host": "api.example.com",
//! //   "port": null,
//! //   "path": "/v1/users",
//! //   "query": "limit=10&offset=20",
//! //   "fragment": null
//! // }
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

                Ok(Rc::new(Variable::Object(result)))
            }
            Err(_) => Err(JmespathError::new(
                ctx.expression,
                0,
                ErrorReason::Parse("Invalid URL".to_owned()),
            )),
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
}
