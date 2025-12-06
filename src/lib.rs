//! # JMESPath Extensions
//!
//! Extended functions for JMESPath queries.
//!
//! # Non-Standard Extension Warning
//!
//! **These functions are NOT part of the [JMESPath specification](https://jmespath.org/specification.html).**
//!
//! Queries using these extension functions will **NOT work** in other JMESPath
//! implementations (Python, JavaScript, Go, etc.). If you need portable queries,
//! use only the [26 standard JMESPath built-in functions](https://jmespath.org/specification.html#built-in-functions):
//!
//! > `abs`, `avg`, `ceil`, `contains`, `ends_with`, `floor`, `join`, `keys`,
//! > `length`, `map`, `max`, `max_by`, `merge`, `min`, `min_by`, `not_null`,
//! > `reverse`, `sort`, `sort_by`, `starts_with`, `sum`, `to_array`, `to_number`,
//! > `to_string`, `type`, `values`
//!
//! These extensions are useful when:
//! - You control both the query author and the runtime environment
//! - You need functionality beyond standard JMESPath
//! - Cross-implementation compatibility is not required
//!
//! # Quick Start
//!
//! ```rust
//! use jmespath::Runtime;
//! use jmespath_extensions::register_all;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! register_all(&mut runtime);
//!
//! // Now you can use extended functions
//! let expr = runtime.compile("upper(@)").unwrap();
//! let data = jmespath::Variable::String("hello".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(), "HELLO");
//! ```
//!
//! # Features
//!
//! - `full` (default) - All functions
//! - `core` - No external dependencies (string, array, object, math, type, utility, path)
//! - Individual features: `string`, `array`, `object`, `math`, `type`, `utility`, `validation`, `path`
//! - With external deps: `hash`, `encoding`, `regex`, `url`, `uuid`, `rand`
//!
//! # Function Categories
//!
//! ## String Functions
//!
//! `lower`, `upper`, `trim`, `trim_start`, `trim_end`, `split`, `replace`,
//! `pad_left`, `pad_right`, `substr`, `capitalize`, `title`, `repeat`,
//! `index_of`, `last_index_of`, `slice`, `concat`, `camel_case`, `snake_case`,
//! `kebab_case`, `truncate`, `wrap`, `format`
//!
//! ## Array Functions
//!
//! `first`, `last`, `unique`, `take`, `drop`, `chunk`, `zip`, `flatten_deep`,
//! `compact`, `range`, `index_at`, `includes`, `find_index`, `group_by`, `nth`,
//! `interleave`, `rotate`, `partition`, `difference`, `intersection`, `union`,
//! `frequencies`, `mode`, `cartesian`
//!
//! ## Object Functions
//!
//! `entries`, `from_entries`, `pick`, `omit`, `invert`, `rename_keys`,
//! `flatten_keys`, `unflatten_keys`, `deep_merge`
//!
//! ## Math Functions
//!
//! `round`, `floor_fn`, `ceil_fn`, `abs_fn`, `mod_fn`, `pow`, `sqrt`, `log`,
//! `clamp`, `median`, `percentile`, `variance`, `stddev`, `sin`, `cos`, `tan`,
//! `asin`, `acos`, `atan`, `atan2`, `deg_to_rad`, `rad_to_deg`, `sign`
//!
//! ## Type Functions
//!
//! `to_string`, `to_number`, `to_boolean`, `type_of`, `is_string`, `is_number`,
//! `is_boolean`, `is_array`, `is_object`, `is_null`, `is_empty`, `is_blank`, `is_json`
//!
//! ## Utility Functions
//!
//! `now`, `now_ms`, `default`, `if`, `coalesce`, `json_encode`, `json_decode`
//!
//! ## Path Functions
//!
//! `path_basename`, `path_dirname`, `path_ext`, `path_join`
//!
//! ## Hash Functions (feature: `hash`)
//!
//! `md5`, `sha1`, `sha256`, `crc32`
//!
//! ## Encoding Functions (feature: `encoding`)
//!
//! `base64_encode`, `base64_decode`, `hex_encode`, `hex_decode`
//!
//! ## URL Functions (feature: `url`)
//!
//! `url_encode`, `url_decode`, `url_parse`
//!
//! ## Regex Functions (feature: `regex`)
//!
//! `regex_match`, `regex_extract`, `regex_replace`
//!
//! ## Validation Functions (feature: `regex` for some)
//!
//! `is_email`, `is_url`, `is_uuid`, `is_ipv4`, `is_ipv6`
//!
//! ## Random Functions (feature: `rand`, `uuid`)
//!
//! `random`, `shuffle`, `sample`, `uuid`

// Re-export common types
pub mod common;
pub use common::{
    ArgumentType, Context, ErrorReason, Function, JmespathError, Rcvar, Runtime, Signature,
    Variable,
};

// Core modules (no external dependencies)
#[cfg(feature = "string")]
pub mod string;

#[cfg(feature = "array")]
pub mod array;

#[cfg(feature = "object")]
pub mod object;

#[cfg(feature = "math")]
pub mod math;

#[cfg(feature = "type")]
pub mod type_conv;

#[cfg(feature = "utility")]
pub mod utility;

#[cfg(feature = "path")]
pub mod path;

#[cfg(feature = "validation")]
pub mod validation;

// Feature-gated modules with external dependencies
#[cfg(feature = "hash")]
pub mod hash;

#[cfg(feature = "encoding")]
pub mod encoding;

#[cfg(feature = "url")]
pub mod url_fns;

#[cfg(feature = "regex")]
pub mod regex_fns;

#[cfg(any(feature = "rand", feature = "uuid"))]
pub mod random;

/// Register all available extension functions with a JMESPath runtime.
///
/// This function registers all functions enabled by the current feature flags.
///
/// # Example
///
/// ```rust
/// use jmespath::Runtime;
/// use jmespath_extensions::register_all;
///
/// let mut runtime = Runtime::new();
/// runtime.register_builtin_functions();
/// register_all(&mut runtime);
/// ```
#[allow(unused_variables)]
pub fn register_all(runtime: &mut Runtime) {
    #[cfg(feature = "string")]
    string::register(runtime);

    #[cfg(feature = "array")]
    array::register(runtime);

    #[cfg(feature = "object")]
    object::register(runtime);

    #[cfg(feature = "math")]
    math::register(runtime);

    #[cfg(feature = "type")]
    type_conv::register(runtime);

    #[cfg(feature = "utility")]
    utility::register(runtime);

    #[cfg(feature = "path")]
    path::register(runtime);

    #[cfg(feature = "validation")]
    validation::register(runtime);

    #[cfg(feature = "hash")]
    hash::register(runtime);

    #[cfg(feature = "encoding")]
    encoding::register(runtime);

    #[cfg(feature = "url")]
    url_fns::register(runtime);

    #[cfg(feature = "regex")]
    regex_fns::register(runtime);

    #[cfg(any(feature = "rand", feature = "uuid"))]
    random::register(runtime);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all() {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register_all(&mut runtime);

        // Test a simple expression
        #[cfg(feature = "string")]
        {
            let expr = runtime.compile("upper(@)").unwrap();
            let data = Variable::String("hello".to_string());
            let result = expr.search(&data).unwrap();
            assert_eq!(result.as_string().unwrap(), "HELLO");
        }
    }
}
