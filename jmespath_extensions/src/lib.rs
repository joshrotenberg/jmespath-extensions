// Allow patterns that clippy suggests replacing with unstable features
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_is_multiple_of)]

//! # JMESPath Extensions
//!
//! A comprehensive collection of 150+ extension functions for [JMESPath](https://jmespath.org/) queries.
//!
//! ## Built on jmespath.rs
//!
//! This crate extends the [`jmespath`](https://crates.io/crates/jmespath) crate by
//! [@mtdowling](https://github.com/mtdowling), which provides the complete Rust implementation
//! of the [JMESPath specification](https://jmespath.org/specification.html). All spec-compliant
//! parsing, evaluation, and the 26 built-in functions come from that foundational library—we
//! simply add extra functions on top.
//!
//! **If you only need standard JMESPath functionality, use [`jmespath`](https://crates.io/crates/jmespath) directly.**
//!
//! ## Non-Standard Extension Warning
//!
//! > **These functions are NOT part of the [JMESPath specification](https://jmespath.org/specification.html).**
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
//! ## Quick Start
//!
//! ```rust
//! # #[cfg(feature = "string")]
//! # fn main() {
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::register_all;
//!
//! // Create a runtime and register all extension functions
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! register_all(&mut runtime);
//!
//! // Use string functions
//! let expr = runtime.compile("upper(@)").unwrap();
//! let data = Variable::String("hello".to_string());
//! let result = expr.search(&data).unwrap();
//! assert_eq!(result.as_string().unwrap(), "HELLO");
//! # }
//! # #[cfg(not(feature = "string"))]
//! # fn main() {}
//! ```
//!
//! ## Working with JSON Data
//!
//! Most real-world usage involves querying JSON data:
//!
//! ```rust
//! # #[cfg(feature = "string")]
//! # fn main() {
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::register_all;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! register_all(&mut runtime);
//!
//! // Parse JSON data
//! let json = r#"{
//!     "users": [
//!         {"name": "alice", "email": "ALICE@EXAMPLE.COM"},
//!         {"name": "bob", "email": "BOB@EXAMPLE.COM"}
//!     ]
//! }"#;
//! let data = Variable::from_json(json).unwrap();
//!
//! // Query with extension functions
//! let expr = runtime.compile("users[*].{name: upper(name), email: lower(email)}").unwrap();
//! let result = expr.search(&data).unwrap();
//!
//! // Result: [{"name": "ALICE", "email": "alice@example.com"}, {"name": "BOB", "email": "bob@example.com"}]
//! # }
//! # #[cfg(not(feature = "string"))]
//! # fn main() {}
//! ```
//!
//! ## Feature Flags
//!
//! This crate uses feature flags to control which functions are included.
//! This allows you to minimize dependencies and binary size.
//!
//! | Feature | Dependencies | Description |
//! |---------|--------------|-------------|
//! | `full` (default) | all | All functions |
//! | `core` | none | String, array, object, math, type, utility, path |
//! | `string` | none | [String manipulation](string/index.html) |
//! | `array` | none | [Array operations](array/index.html) |
//! | `object` | none | [Object utilities](object/index.html) |
//! | `math` | none | [Mathematical operations](math/index.html) |
//! | `type` | none | [Type conversion and checking](type_conv/index.html) |
//! | `utility` | none | [Utility functions](utility/index.html) |
//! | `path` | none | [Path manipulation](path/index.html) |
//! | `validation` | none | [Validation functions](validation/index.html) |
//! | `hash` | md-5, sha1, sha2, crc32fast | [Hash functions](hash/index.html) |
//! | `encoding` | base64, hex | [Encoding functions](encoding/index.html) |
//! | `url` | url | [URL functions](url_fns/index.html) |
//! | `regex` | regex | [Regex functions](regex_fns/index.html) |
//! | `uuid` | uuid | UUID generation |
//! | `rand` | rand | [Random functions](random/index.html) |
//! | `datetime` | chrono | [Date/time functions](datetime/index.html) |
//! | `fuzzy` | strsim | [Fuzzy matching functions](fuzzy/index.html) |
//! | `expression` | none | [Expression-based functions](expression/index.html) |
//! | `phonetic` | rphonetic | [Phonetic encoding functions](phonetic/index.html) |
//! | `geo` | geoutils | [Geospatial functions](geo/index.html) |
//! | `semver` | semver | [Semantic versioning](semver_fns/index.html) |
//! | `network` | ipnetwork | [Network/IP functions](network/index.html) |
//! | `ids` | nanoid, ulid | [ID generation](ids/index.html) |
//! | `text` | none | [Text analysis](text/index.html) |
//! | `duration` | none | [Duration parsing](duration/index.html) |
//! | `color` | none | [Color manipulation](color/index.html) |
//! | `computing` | none | [Computing utilities](computing/index.html) |
//! | `jsonpatch` | json-patch | [JSON Patch functions](jsonpatch/index.html) |
//!
//! ### Using Specific Features
//!
//! ```toml
//! # Only include string and array functions (no external dependencies)
//! [dependencies]
//! jmespath_extensions = { version = "0.1", default-features = false, features = ["string", "array"] }
//!
//! # Include core functions plus regex
//! [dependencies]
//! jmespath_extensions = { version = "0.1", default-features = false, features = ["core", "regex"] }
//! ```
//!
//! ## Module Overview
//!
//! See each module's documentation for detailed function reference with examples:
//!
//! - [`string`] - String manipulation (`upper`, `lower`, `split`, `replace`, `camel_case`, etc.)
//! - [`mod@array`] - Array operations (`first`, `last`, `unique`, `chunk`, `zip`, `range`, etc.)
//! - [`object`] - Object utilities (`items`, `pick`, `omit`, `deep_merge`, etc.)
//! - [`math`] - Math operations (`round`, `sqrt`, `pow`, `median`, `sin`, `cos`, etc.)
//! - [`type_conv`] - Type functions (`type_of`, `is_string`, `is_empty`, `to_number`, etc.)
//! - [`utility`] - Utilities (`default`, `if`, `coalesce`, `json_encode`, etc.)
//! - [`datetime`] - Date/time (`now`, `now_millis`, `parse_date`, `format_date`, `date_add`, `date_diff`)
//! - [`fuzzy`] - Fuzzy matching (`levenshtein`, `jaro_winkler`, `sorensen_dice`, etc.)
//! - [`expression`] - Expression functions (`map_expr`, `filter_expr`, `any_expr`, `all_expr`, `find_expr`, `sort_by_expr`)
//! - [`path`] - Path functions (`path_basename`, `path_dirname`, `path_ext`, `path_join`)
//! - [`validation`] - Validation (`is_email`, `is_url`, `is_uuid`, `is_ipv4`, `is_ipv6`)
//! - [`hash`] - Hashing (`md5`, `sha1`, `sha256`, `crc32`)
//! - [`encoding`] - Encoding (`base64_encode`, `base64_decode`, `hex_encode`, `hex_decode`)
//! - [`url_fns`] - URL functions (`url_encode`, `url_decode`, `url_parse`)
//! - [`regex_fns`] - Regex (`regex_match`, `regex_extract`, `regex_replace`)
//! - [`random`] - Random (`random`, `shuffle`, `sample`, `uuid`)
//! - [`phonetic`] - Phonetic encoding (`soundex`, `metaphone`, `double_metaphone`, `nysiis`, `sounds_like`)
//! - [`geo`] - Geospatial (`haversine`, `haversine_km`, `haversine_mi`, `bearing`)
//! - [`semver_fns`] - Semantic versioning (`semver_parse`, `semver_compare`, `semver_matches`, `is_semver`)
//! - [`network`] - Network/IP (`ip_to_int`, `int_to_ip`, `cidr_contains`, `cidr_network`, `is_private_ip`)
//! - [`ids`] - ID generation (`nanoid`, `ulid`, `ulid_timestamp`)
//! - [`text`] - Text analysis (`word_count`, `char_count`, `reading_time`, `word_frequencies`)
//! - [`duration`] - Duration parsing (`parse_duration`, `format_duration`)
//! - [`color`] - Color manipulation (`hex_to_rgb`, `rgb_to_hex`, `lighten`, `darken`, `color_mix`)
//! - [`computing`] - Computing utilities (`parse_bytes`, `format_bytes`, `bit_and`, `bit_or`, `bit_xor`)
//! - [`jsonpatch`] - JSON Patch (RFC 6902) and Merge Patch (RFC 7386) (`json_patch`, `json_merge_patch`, `json_diff`)
//!
//! ## Error Handling
//!
//! Extension functions follow JMESPath conventions:
//! - Type errors return an error (e.g., passing a number to `upper`)
//! - Invalid operations return null (e.g., `index_at` with out-of-bounds index)
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::register_all;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! register_all(&mut runtime);
//!
//! // Type error - upper expects a string
//! let expr = runtime.compile("upper(@)").unwrap();
//! let data = Variable::Number(serde_json::Number::from(42));
//! assert!(expr.search(&data).is_err());
//! ```
//!
//! ```rust
//! # use jmespath::{Runtime, Variable};
//! # use jmespath_extensions::register_all;
//! # let mut runtime = Runtime::new();
//! # runtime.register_builtin_functions();
//! # register_all(&mut runtime);
//! // Out of bounds - returns null (requires "array" feature)
//! # #[cfg(feature = "array")]
//! # {
//! let expr = runtime.compile("index_at(@, `10`)").unwrap();
//! let data = Variable::from_json("[1, 2, 3]").unwrap();
//! let result = expr.search(&data).unwrap();
//! assert!(result.is_null());
//! # }
//! ```

// Re-export common types
pub mod common;

// Function registry for runtime control
pub mod registry;
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

#[cfg(feature = "expression")]
pub mod expression;

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

#[cfg(feature = "datetime")]
pub mod datetime;

#[cfg(feature = "fuzzy")]
pub mod fuzzy;

#[cfg(feature = "phonetic")]
pub mod phonetic;

#[cfg(feature = "geo")]
pub mod geo;

#[cfg(feature = "semver")]
pub mod semver_fns;

#[cfg(feature = "network")]
pub mod network;

#[cfg(feature = "ids")]
pub mod ids;

#[cfg(feature = "text")]
pub mod text;

#[cfg(feature = "duration")]
pub mod duration;

#[cfg(feature = "color")]
pub mod color;

#[cfg(feature = "computing")]
pub mod computing;

#[cfg(feature = "jsonpatch")]
pub mod jsonpatch;

/// Register all available extension functions with a JMESPath runtime.
///
/// This function registers all functions enabled by the current feature flags.
/// Call this after creating a new runtime and registering the built-in functions.
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
///
/// // Now use any extension function
/// let expr = runtime.compile("upper(@)").unwrap();
/// ```
///
/// # Feature Flags
///
/// Only functions enabled by feature flags will be registered:
///
/// ```rust,ignore
/// // With default features (all functions)
/// register_all(&mut runtime);  // Registers 100+ functions
///
/// // With only "string" feature
/// register_all(&mut runtime);  // Registers only string functions
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

    #[cfg(feature = "datetime")]
    datetime::register(runtime);

    #[cfg(feature = "fuzzy")]
    fuzzy::register(runtime);

    #[cfg(feature = "expression")]
    expression::register(runtime);

    #[cfg(feature = "phonetic")]
    phonetic::register(runtime);

    #[cfg(feature = "geo")]
    geo::register(runtime);

    #[cfg(feature = "semver")]
    semver_fns::register(runtime);

    #[cfg(feature = "network")]
    network::register(runtime);

    #[cfg(feature = "ids")]
    ids::register(runtime);

    #[cfg(feature = "text")]
    text::register(runtime);

    #[cfg(feature = "duration")]
    duration::register(runtime);

    #[cfg(feature = "color")]
    color::register(runtime);

    #[cfg(feature = "computing")]
    computing::register(runtime);

    #[cfg(feature = "jsonpatch")]
    jsonpatch::register(runtime);
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
