//! Function registry for runtime control and introspection.
//!
//! The registry provides:
//! - Runtime enable/disable of functions (for ACLs, config-based gating)
//! - Introspection (list available functions, their signatures, descriptions)
//! - Category-based registration
//! - Metadata about standard vs extension functions and JEP alignment
//!
//! # Standard vs Extension Functions
//!
//! The registry distinguishes between:
//! - **Standard functions** (26): Built into JMESPath spec (`abs`, `length`, `sort`, etc.)
//! - **Extension functions** (189): Additional functions provided by this crate
//!
//! Standard functions are registered via `runtime.register_builtin_functions()` (from the
//! `jmespath` crate), not via `registry.apply()`. The registry's `Category::Standard` provides
//! metadata about these functions for introspection purposes, but does not re-register them.
//!
//! # JEP Alignment
//!
//! Some extension functions align with JMESPath Enhancement Proposals (JEPs):
//! - **JEP-014**: String functions (`upper`, `lower`, `trim`, `split`, `replace`, etc.)
//! - **JEP-013**: Object functions (`items`, `from_items`, `zip`)
//!
//! Check `FunctionInfo::jep` to see if a function aligns with a proposal.
//!
//! # Example
//!
//! ```
//! use jmespath::Runtime;
//! use jmespath_extensions::registry::{FunctionRegistry, Category};
//!
//! let mut registry = FunctionRegistry::new();
//!
//! // Register specific categories
//! registry.register_category(Category::String);
//! registry.register_category(Category::Math);
//!
//! // Or register all (includes Standard for introspection)
//! // registry.register_all();
//!
//! // Disable specific functions for ACL
//! registry.disable_function("md5");
//! registry.disable_function("sha256");
//!
//! // Apply to runtime - registers extension functions
//! // Note: Standard functions come from runtime.register_builtin_functions()
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions(); // Standard JMESPath functions
//! registry.apply(&mut runtime);          // Extension functions
//!
//! // Introspection - includes both standard and extension metadata
//! for func in registry.functions() {
//!     let type_label = if func.is_standard { "std" } else { "ext" };
//!     let jep_label = func.jep.unwrap_or("-");
//!     println!("[{}] {} ({}): {}", type_label, func.name, jep_label, func.description);
//! }
//! ```

use jmespath::Runtime;
use std::collections::{HashMap, HashSet};

/// Function category matching compile-time features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Standard JMESPath built-in functions (always available)
    Standard,
    String,
    Array,
    Object,
    Math,
    Type,
    Utility,
    Validation,
    Path,
    Expression,
    Text,
    Hash,
    Encoding,
    Regex,
    Url,
    Uuid,
    Rand,
    Datetime,
    Fuzzy,
    Phonetic,
    Geo,
    Semver,
    Network,
    Ids,
    Duration,
    Color,
    Computing,
}

impl Category {
    /// Returns all categories (including Standard)
    pub fn all() -> &'static [Category] {
        &[
            Category::Standard,
            Category::String,
            Category::Array,
            Category::Object,
            Category::Math,
            Category::Type,
            Category::Utility,
            Category::Validation,
            Category::Path,
            Category::Expression,
            Category::Text,
            Category::Hash,
            Category::Encoding,
            Category::Regex,
            Category::Url,
            Category::Uuid,
            Category::Rand,
            Category::Datetime,
            Category::Fuzzy,
            Category::Phonetic,
            Category::Geo,
            Category::Semver,
            Category::Network,
            Category::Ids,
            Category::Duration,
            Category::Color,
            Category::Computing,
        ]
    }

    /// Returns the category name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Category::Standard => "standard",
            Category::String => "string",
            Category::Array => "array",
            Category::Object => "object",
            Category::Math => "math",
            Category::Type => "type",
            Category::Utility => "utility",
            Category::Validation => "validation",
            Category::Path => "path",
            Category::Expression => "expression",
            Category::Text => "text",
            Category::Hash => "hash",
            Category::Encoding => "encoding",
            Category::Regex => "regex",
            Category::Url => "url",
            Category::Uuid => "uuid",
            Category::Rand => "rand",
            Category::Datetime => "datetime",
            Category::Fuzzy => "fuzzy",
            Category::Phonetic => "phonetic",
            Category::Geo => "geo",
            Category::Semver => "semver",
            Category::Network => "network",
            Category::Ids => "ids",
            Category::Duration => "duration",
            Category::Color => "color",
            Category::Computing => "computing",
        }
    }

    /// Check if this category is available (compiled in)
    pub fn is_available(&self) -> bool {
        match self {
            // Standard functions are always available
            Category::Standard => true,
            #[cfg(feature = "string")]
            Category::String => true,
            #[cfg(feature = "array")]
            Category::Array => true,
            #[cfg(feature = "object")]
            Category::Object => true,
            #[cfg(feature = "math")]
            Category::Math => true,
            #[cfg(feature = "type")]
            Category::Type => true,
            #[cfg(feature = "utility")]
            Category::Utility => true,
            #[cfg(feature = "validation")]
            Category::Validation => true,
            #[cfg(feature = "path")]
            Category::Path => true,
            #[cfg(feature = "expression")]
            Category::Expression => true,
            #[cfg(feature = "text")]
            Category::Text => true,
            #[cfg(feature = "hash")]
            Category::Hash => true,
            #[cfg(feature = "encoding")]
            Category::Encoding => true,
            #[cfg(feature = "regex")]
            Category::Regex => true,
            #[cfg(feature = "url")]
            Category::Url => true,
            #[cfg(feature = "uuid")]
            Category::Uuid => true,
            #[cfg(feature = "rand")]
            Category::Rand => true,
            #[cfg(feature = "datetime")]
            Category::Datetime => true,
            #[cfg(feature = "fuzzy")]
            Category::Fuzzy => true,
            #[cfg(feature = "phonetic")]
            Category::Phonetic => true,
            #[cfg(feature = "geo")]
            Category::Geo => true,
            #[cfg(feature = "semver")]
            Category::Semver => true,
            #[cfg(feature = "network")]
            Category::Network => true,
            #[cfg(feature = "ids")]
            Category::Ids => true,
            #[cfg(feature = "duration")]
            Category::Duration => true,
            #[cfg(feature = "color")]
            Category::Color => true,
            #[cfg(feature = "computing")]
            Category::Computing => true,
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

/// Feature tags for function classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Standard JMESPath spec functions
    Spec,
    /// Core jmespath-extensions functions
    Core,
    /// Functional programming style functions
    Fp,
    /// JEP-aligned functions
    Jep,
}

impl Feature {
    /// Returns all features
    pub fn all() -> &'static [Feature] {
        &[Feature::Spec, Feature::Core, Feature::Fp, Feature::Jep]
    }

    /// Returns the feature name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Feature::Spec => "spec",
            Feature::Core => "core",
            Feature::Fp => "fp",
            Feature::Jep => "jep",
        }
    }
}

/// Metadata about a function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name as used in JMESPath expressions
    pub name: &'static str,
    /// Category this function belongs to
    pub category: Category,
    /// Human-readable description
    pub description: &'static str,
    /// Argument signature (e.g., "string, string -> string")
    pub signature: &'static str,
    /// Example usage
    pub example: &'static str,
    /// Whether this is a standard JMESPath function (vs extension)
    pub is_standard: bool,
    /// JMESPath Enhancement Proposal reference (e.g., "JEP-014")
    /// See: <https://github.com/jmespath-community/jmespath.spec>
    pub jep: Option<&'static str>,
    /// Alternative names for this function (e.g., "some" for "any_expr")
    pub aliases: &'static [&'static str],
    /// Feature tags for classification (e.g., "fp", "core")
    pub features: &'static [Feature],
}

/// Registry for managing function availability at runtime
#[derive(Debug, Clone)]
pub struct FunctionRegistry {
    /// Functions that have been registered
    registered: HashMap<&'static str, FunctionInfo>,
    /// Functions that have been explicitly disabled
    disabled: HashSet<String>,
    /// Categories that have been registered
    categories: HashSet<Category>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
            disabled: HashSet::new(),
            categories: HashSet::new(),
        }
    }

    /// Register all available functions (respects compile-time features)
    pub fn register_all(&mut self) -> &mut Self {
        for category in Category::all() {
            if category.is_available() {
                self.register_category(*category);
            }
        }
        self
    }

    /// Register all functions in a category
    pub fn register_category(&mut self, category: Category) -> &mut Self {
        if !category.is_available() {
            return self;
        }

        self.categories.insert(category);

        for info in get_category_functions(category) {
            self.registered.insert(info.name, info);
        }
        self
    }

    /// Disable a specific function (for ACLs)
    pub fn disable_function(&mut self, name: &str) -> &mut Self {
        self.disabled.insert(name.to_string());
        self
    }

    /// Enable a previously disabled function
    pub fn enable_function(&mut self, name: &str) -> &mut Self {
        self.disabled.remove(name);
        self
    }

    /// Check if a function is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.registered.contains_key(name) && !self.disabled.contains(name)
    }

    /// Get info about a specific function
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        if self.disabled.contains(name) {
            None
        } else {
            self.registered.get(name)
        }
    }

    /// Iterate over all enabled functions
    pub fn functions(&self) -> impl Iterator<Item = &FunctionInfo> {
        self.registered
            .values()
            .filter(|f| !self.disabled.contains(f.name))
    }

    /// Iterate over functions in a specific category
    pub fn functions_in_category(&self, category: Category) -> impl Iterator<Item = &FunctionInfo> {
        self.registered
            .values()
            .filter(move |f| f.category == category && !self.disabled.contains(f.name))
    }

    /// Get all registered categories
    pub fn categories(&self) -> impl Iterator<Item = &Category> {
        self.categories.iter()
    }

    /// Get count of enabled functions
    pub fn len(&self) -> usize {
        self.registered.len() - self.disabled.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over functions with a specific feature tag
    pub fn functions_with_feature(&self, feature: Feature) -> impl Iterator<Item = &FunctionInfo> {
        self.registered
            .values()
            .filter(move |f| f.features.contains(&feature) && !self.disabled.contains(f.name))
    }

    /// Get all spec-only (standard JMESPath) function names
    pub fn spec_function_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.functions_with_feature(Feature::Spec).map(|f| f.name)
    }

    /// Check if a function is a standard JMESPath spec function
    pub fn is_spec_function(&self, name: &str) -> bool {
        self.registered
            .get(name)
            .map(|f| f.features.contains(&Feature::Spec))
            .unwrap_or(false)
    }

    /// Get function info by name or alias
    pub fn get_function_by_name_or_alias(&self, name: &str) -> Option<&FunctionInfo> {
        // First try direct lookup
        if let Some(info) = self.get_function(name) {
            return Some(info);
        }
        // Then search aliases
        self.registered
            .values()
            .find(|f| f.aliases.contains(&name) && !self.disabled.contains(f.name))
    }

    /// Get all aliases for all functions as (alias, canonical_name) pairs
    pub fn all_aliases(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.registered
            .values()
            .flat_map(|f| f.aliases.iter().map(move |alias| (*alias, f.name)))
    }

    /// Apply the registry to a JMESPath runtime
    ///
    /// This registers all enabled functions with the runtime.
    pub fn apply(&self, runtime: &mut Runtime) {
        for category in &self.categories {
            if category.is_available() {
                self.apply_category(runtime, *category);
            }
        }
    }

    #[allow(unused_variables)]
    fn apply_category(&self, runtime: &mut Runtime, category: Category) {
        // Check which functions in this category are enabled
        let enabled_in_category: HashSet<&str> = self
            .functions_in_category(category)
            .map(|f| f.name)
            .collect();

        if enabled_in_category.is_empty() {
            return;
        }

        // Register the category, but we need to handle disabled functions
        // For now, we register all and rely on a wrapper for disabled check
        // TODO: More granular registration
        match category {
            #[cfg(feature = "string")]
            Category::String => crate::string::register(runtime),
            #[cfg(feature = "array")]
            Category::Array => crate::array::register(runtime),
            #[cfg(feature = "object")]
            Category::Object => crate::object::register(runtime),
            #[cfg(feature = "math")]
            Category::Math => crate::math::register(runtime),
            #[cfg(feature = "type")]
            Category::Type => crate::type_conv::register(runtime),
            #[cfg(feature = "utility")]
            Category::Utility => crate::utility::register(runtime),
            #[cfg(feature = "validation")]
            Category::Validation => crate::validation::register(runtime),
            #[cfg(feature = "path")]
            Category::Path => crate::path::register(runtime),
            #[cfg(feature = "expression")]
            Category::Expression => crate::expression::register(runtime),
            #[cfg(feature = "text")]
            Category::Text => crate::text::register(runtime),
            #[cfg(feature = "hash")]
            Category::Hash => crate::hash::register(runtime),
            #[cfg(feature = "encoding")]
            Category::Encoding => crate::encoding::register(runtime),
            #[cfg(feature = "regex")]
            Category::Regex => crate::regex_fns::register(runtime),
            #[cfg(feature = "url")]
            Category::Url => crate::url_fns::register(runtime),
            #[cfg(feature = "uuid")]
            Category::Uuid => crate::random::register(runtime),
            #[cfg(feature = "rand")]
            Category::Rand => crate::random::register(runtime),
            #[cfg(feature = "datetime")]
            Category::Datetime => crate::datetime::register(runtime),
            #[cfg(feature = "fuzzy")]
            Category::Fuzzy => crate::fuzzy::register(runtime),
            #[cfg(feature = "phonetic")]
            Category::Phonetic => crate::phonetic::register(runtime),
            #[cfg(feature = "geo")]
            Category::Geo => crate::geo::register(runtime),
            #[cfg(feature = "semver")]
            Category::Semver => crate::semver_fns::register(runtime),
            #[cfg(feature = "network")]
            Category::Network => crate::network::register(runtime),
            #[cfg(feature = "ids")]
            Category::Ids => crate::ids::register(runtime),
            #[cfg(feature = "duration")]
            Category::Duration => crate::duration::register(runtime),
            #[cfg(feature = "color")]
            Category::Color => crate::color::register(runtime),
            #[cfg(feature = "computing")]
            Category::Computing => crate::computing::register(runtime),
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
}

/// Get function metadata for a category
fn get_category_functions(category: Category) -> Vec<FunctionInfo> {
    match category {
        Category::Standard => standard_functions(),
        Category::String => string_functions(),
        Category::Array => array_functions(),
        Category::Object => object_functions(),
        Category::Math => math_functions(),
        Category::Type => type_functions(),
        Category::Utility => utility_functions(),
        Category::Validation => validation_functions(),
        Category::Path => path_functions(),
        Category::Expression => expression_functions(),
        Category::Text => text_functions(),
        Category::Hash => hash_functions(),
        Category::Encoding => encoding_functions(),
        Category::Regex => regex_functions(),
        Category::Url => url_functions(),
        Category::Uuid => uuid_functions(),
        Category::Rand => rand_functions(),
        Category::Datetime => datetime_functions(),
        Category::Fuzzy => fuzzy_functions(),
        Category::Phonetic => phonetic_functions(),
        Category::Geo => geo_functions(),
        Category::Semver => semver_functions(),
        Category::Network => network_functions(),
        Category::Ids => ids_functions(),
        Category::Duration => duration_functions(),
        Category::Color => color_functions(),
        Category::Computing => computing_functions(),
    }
}

// Function metadata definitions

/// Standard JMESPath built-in functions (always available)
fn standard_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "abs",
            category: Category::Standard,
            description: "Returns the absolute value of a number",
            signature: "number -> number",
            example: "abs(`-5`) -> 5",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "avg",
            category: Category::Standard,
            description: "Returns the average of an array of numbers",
            signature: "array[number] -> number",
            example: "avg([1, 2, 3]) -> 2",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "ceil",
            category: Category::Standard,
            description: "Returns the smallest integer greater than or equal to the number",
            signature: "number -> number",
            example: "ceil(`1.5`) -> 2",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "contains",
            category: Category::Standard,
            description: "Returns true if the subject contains the search value",
            signature: "array|string, any -> boolean",
            example: "contains([1, 2, 3], `2`) -> true",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "ends_with",
            category: Category::Standard,
            description: "Returns true if the subject ends with the suffix",
            signature: "string, string -> boolean",
            example: "ends_with('hello', 'lo') -> true",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "floor",
            category: Category::Standard,
            description: "Returns the largest integer less than or equal to the number",
            signature: "number -> number",
            example: "floor(`1.9`) -> 1",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "join",
            category: Category::Standard,
            description: "Returns array elements joined into a string with a separator",
            signature: "string, array[string] -> string",
            example: "join(', ', ['a', 'b', 'c']) -> \"a, b, c\"",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "keys",
            category: Category::Standard,
            description: "Returns an array of keys from an object",
            signature: "object -> array[string]",
            example: "keys({a: 1, b: 2}) -> [\"a\", \"b\"]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "length",
            category: Category::Standard,
            description: "Returns the length of an array, object, or string",
            signature: "array|object|string -> number",
            example: "length([1, 2, 3]) -> 3",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "map",
            category: Category::Standard,
            description: "Applies an expression to each element of an array",
            signature: "expression, array -> array",
            example: "map(&a, [{a: 1}, {a: 2}]) -> [1, 2]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "max",
            category: Category::Standard,
            description: "Returns the maximum value in an array",
            signature: "array[number]|array[string] -> number|string",
            example: "max([1, 3, 2]) -> 3",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "max_by",
            category: Category::Standard,
            description: "Returns the element with maximum value by expression",
            signature: "array, expression -> any",
            example: "max_by([{a: 1}, {a: 2}], &a) -> {a: 2}",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "merge",
            category: Category::Standard,
            description: "Merges objects into a single object",
            signature: "object... -> object",
            example: "merge({a: 1}, {b: 2}) -> {a: 1, b: 2}",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "min",
            category: Category::Standard,
            description: "Returns the minimum value in an array",
            signature: "array[number]|array[string] -> number|string",
            example: "min([1, 3, 2]) -> 1",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "min_by",
            category: Category::Standard,
            description: "Returns the element with minimum value by expression",
            signature: "array, expression -> any",
            example: "min_by([{a: 1}, {a: 2}], &a) -> {a: 1}",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "not_null",
            category: Category::Standard,
            description: "Returns the first non-null argument",
            signature: "any... -> any",
            example: "not_null(`null`, 'a', 'b') -> \"a\"",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "reverse",
            category: Category::Standard,
            description: "Reverses an array or string",
            signature: "array|string -> array|string",
            example: "reverse([1, 2, 3]) -> [3, 2, 1]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "sort",
            category: Category::Standard,
            description: "Sorts an array of numbers or strings",
            signature: "array[number]|array[string] -> array",
            example: "sort([3, 1, 2]) -> [1, 2, 3]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "sort_by",
            category: Category::Standard,
            description: "Sorts an array by expression result",
            signature: "array, expression -> array",
            example: "sort_by([{a: 2}, {a: 1}], &a) -> [{a: 1}, {a: 2}]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "starts_with",
            category: Category::Standard,
            description: "Returns true if the subject starts with the prefix",
            signature: "string, string -> boolean",
            example: "starts_with('hello', 'he') -> true",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "sum",
            category: Category::Standard,
            description: "Returns the sum of an array of numbers",
            signature: "array[number] -> number",
            example: "sum([1, 2, 3]) -> 6",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "to_array",
            category: Category::Standard,
            description: "Converts a value to an array",
            signature: "any -> array",
            example: "to_array('hello') -> [\"hello\"]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "to_number",
            category: Category::Standard,
            description: "Converts a value to a number",
            signature: "any -> number",
            example: "to_number('42') -> 42",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "to_string",
            category: Category::Standard,
            description: "Converts a value to a string",
            signature: "any -> string",
            example: "to_string(`42`) -> \"42\"",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "type",
            category: Category::Standard,
            description: "Returns the type of a value as a string",
            signature: "any -> string",
            example: "type('hello') -> \"string\"",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
        FunctionInfo {
            name: "values",
            category: Category::Standard,
            description: "Returns an array of values from an object",
            signature: "object -> array",
            example: "values({a: 1, b: 2}) -> [1, 2]",
            is_standard: true,
            jep: None,
            aliases: &[],
            features: &[Feature::Spec],
        },
    ]
}

fn string_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "upper",
            category: Category::String,
            description: "Convert string to uppercase",
            signature: "string -> string",
            example: "upper('hello') -> \"HELLO\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "lower",
            category: Category::String,
            description: "Convert string to lowercase",
            signature: "string -> string",
            example: "lower('HELLO') -> \"hello\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "trim",
            category: Category::String,
            description: "Remove leading and trailing whitespace",
            signature: "string -> string",
            example: "trim('  hello  ') -> \"hello\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "trim_left",
            category: Category::String,
            description: "Remove leading whitespace",
            signature: "string -> string",
            example: "trim_left('  hello') -> \"hello\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "trim_right",
            category: Category::String,
            description: "Remove trailing whitespace",
            signature: "string -> string",
            example: "trim_right('hello  ') -> \"hello\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "split",
            category: Category::String,
            description: "Split string by delimiter",
            signature: "string, string -> array",
            example: "split('a,b,c', ',') -> [\"a\", \"b\", \"c\"]",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "replace",
            category: Category::String,
            description: "Replace occurrences of a substring",
            signature: "string, string, string -> string",
            example: "replace('hello', 'l', 'L') -> \"heLLo\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "repeat",
            category: Category::String,
            description: "Repeat a string n times",
            signature: "string, number -> string",
            example: "repeat('ab', `3`) -> \"ababab\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "pad_left",
            category: Category::String,
            description: "Pad string on the left to reach target length",
            signature: "string, number, string -> string",
            example: "pad_left('5', `3`, '0') -> \"005\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "pad_right",
            category: Category::String,
            description: "Pad string on the right to reach target length",
            signature: "string, number, string -> string",
            example: "pad_right('5', `3`, '0') -> \"500\"",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "capitalize",
            category: Category::String,
            description: "Capitalize the first character",
            signature: "string -> string",
            example: "capitalize('hello') -> \"Hello\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "title",
            category: Category::String,
            description: "Convert to title case",
            signature: "string -> string",
            example: "title('hello world') -> \"Hello World\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "camel_case",
            category: Category::String,
            description: "Convert to camelCase",
            signature: "string -> string",
            example: "camel_case('hello_world') -> \"helloWorld\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "snake_case",
            category: Category::String,
            description: "Convert to snake_case",
            signature: "string -> string",
            example: "snake_case('helloWorld') -> \"hello_world\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "kebab_case",
            category: Category::String,
            description: "Convert to kebab-case",
            signature: "string -> string",
            example: "kebab_case('helloWorld') -> \"hello-world\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "substr",
            category: Category::String,
            description: "Extract substring by start index and length",
            signature: "string, number, number -> string",
            example: "substr('hello', `1`, `3`) -> \"ell\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "slice",
            category: Category::String,
            description: "Extract substring by start and end index",
            signature: "string, number, number -> string",
            example: "slice('hello', `1`, `4`) -> \"ell\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "find_first",
            category: Category::String,
            description: "Find first occurrence of substring",
            signature: "string, string -> number | null",
            example: "find_first('hello', 'l') -> 2",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "find_last",
            category: Category::String,
            description: "Find last occurrence of substring",
            signature: "string, string -> number | null",
            example: "find_last('hello', 'l') -> 3",
            is_standard: false,
            jep: Some("JEP-014"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "concat",
            category: Category::String,
            description: "Concatenate strings",
            signature: "string... -> string",
            example: "concat('hello', ' ', 'world') -> \"hello world\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "wrap",
            category: Category::String,
            description: "Wrap text to specified width",
            signature: "string, number -> string",
            example: "wrap('hello world', `5`) -> \"hello\\nworld\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "ltrimstr",
            category: Category::String,
            description: "Remove prefix from string if present",
            signature: "string, string -> string",
            example: "ltrimstr('foobar', 'foo') -> \"bar\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "rtrimstr",
            category: Category::String,
            description: "Remove suffix from string if present",
            signature: "string, string -> string",
            example: "rtrimstr('foobar', 'bar') -> \"foo\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "indices",
            category: Category::String,
            description: "Find all indices of substring occurrences",
            signature: "string, string -> array",
            example: "indices('hello', 'l') -> [2, 3]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "inside",
            category: Category::String,
            description: "Check if search string is contained in string",
            signature: "string, string -> boolean",
            example: "inside('world', 'hello world') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sprintf",
            category: Category::String,
            description: "Printf-style string formatting",
            signature: "string, any... -> string",
            example: "sprintf('Pi is %.2f', `3.14159`) -> \"Pi is 3.14\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn array_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "first",
            category: Category::Array,
            description: "Get first element of array",
            signature: "array -> any",
            example: "first([1, 2, 3]) -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "last",
            category: Category::Array,
            description: "Get last element of array",
            signature: "array -> any",
            example: "last([1, 2, 3]) -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "unique",
            category: Category::Array,
            description: "Remove duplicate values",
            signature: "array -> array",
            example: "unique([1, 2, 1, 3]) -> [1, 2, 3]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "take",
            category: Category::Array,
            description: "Take first n elements",
            signature: "array, number -> array",
            example: "take([1, 2, 3, 4], `2`) -> [1, 2]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "drop",
            category: Category::Array,
            description: "Drop first n elements",
            signature: "array, number -> array",
            example: "drop([1, 2, 3, 4], `2`) -> [3, 4]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "chunk",
            category: Category::Array,
            description: "Split array into chunks of size n",
            signature: "array, number -> array",
            example: "chunk([1, 2, 3, 4], `2`) -> [[1, 2], [3, 4]]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "zip",
            category: Category::Array,
            description: "Zip two arrays together",
            signature: "array, array -> array",
            example: "zip([1, 2], ['a', 'b']) -> [[1, 'a'], [2, 'b']]",
            is_standard: false,
            jep: Some("JEP-013"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "flatten_deep",
            category: Category::Array,
            description: "Recursively flatten nested arrays",
            signature: "array -> array",
            example: "flatten_deep([[1, [2]], [3]]) -> [1, 2, 3]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "compact",
            category: Category::Array,
            description: "Remove null values from array",
            signature: "array -> array",
            example: "compact([1, null, 2, null]) -> [1, 2]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "range",
            category: Category::Array,
            description: "Generate array of numbers",
            signature: "number, number -> array",
            example: "range(`1`, `5`) -> [1, 2, 3, 4]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "index_at",
            category: Category::Array,
            description: "Get element at index (supports negative)",
            signature: "array, number -> any",
            example: "index_at([1, 2, 3], `-1`) -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "includes",
            category: Category::Array,
            description: "Check if array contains value",
            signature: "array, any -> boolean",
            example: "includes([1, 2, 3], `2`) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "find_index",
            category: Category::Array,
            description: "Find index of value in array",
            signature: "array, any -> number | null",
            example: "find_index([1, 2, 3], `2`) -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "difference",
            category: Category::Array,
            description: "Elements in first array not in second",
            signature: "array, array -> array",
            example: "difference([1, 2, 3], [2]) -> [1, 3]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "intersection",
            category: Category::Array,
            description: "Elements common to both arrays",
            signature: "array, array -> array",
            example: "intersection([1, 2], [2, 3]) -> [2]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "union",
            category: Category::Array,
            description: "Unique elements from both arrays",
            signature: "array, array -> array",
            example: "union([1, 2], [2, 3]) -> [1, 2, 3]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "group_by",
            category: Category::Array,
            description: "Group array elements by key",
            signature: "array, string -> object",
            example: "group_by([{t: 'a'}, {t: 'b'}], 't') -> {a: [...], b: [...]}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "frequencies",
            category: Category::Array,
            description: "Count occurrences of each value",
            signature: "array -> object",
            example: "frequencies(['a', 'b', 'a']) -> {a: 2, b: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn object_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "items",
            category: Category::Object,
            description: "Convert object to array of [key, value] pairs",
            signature: "object -> array",
            example: "items({a: 1}) -> [[\"a\", 1]]",
            is_standard: false,
            jep: Some("JEP-013"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "from_items",
            category: Category::Object,
            description: "Convert array of [key, value] pairs to object",
            signature: "array -> object",
            example: "from_items([['a', 1]]) -> {a: 1}",
            is_standard: false,
            jep: Some("JEP-013"),
            aliases: &[],
            features: &[Feature::Core, Feature::Jep],
        },
        FunctionInfo {
            name: "pick",
            category: Category::Object,
            description: "Select specific keys from object",
            signature: "object, array -> object",
            example: "pick({a: 1, b: 2}, ['a']) -> {a: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "omit",
            category: Category::Object,
            description: "Remove specific keys from object",
            signature: "object, array -> object",
            example: "omit({a: 1, b: 2}, ['a']) -> {b: 2}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "deep_merge",
            category: Category::Object,
            description: "Recursively merge objects",
            signature: "object, object -> object",
            example: "deep_merge({a: {b: 1}}, {a: {c: 2}}) -> {a: {b: 1, c: 2}}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "deep_equals",
            category: Category::Object,
            description: "Deep equality check for any two values",
            signature: "any, any -> boolean",
            example: "deep_equals({a: {b: 1}}, {a: {b: 1}}) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "deep_diff",
            category: Category::Object,
            description: "Structural diff between two objects",
            signature: "object, object -> object",
            example: "deep_diff({a: 1}, {a: 2}) -> {added: {}, removed: {}, changed: {a: {from: 1, to: 2}}}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "invert",
            category: Category::Object,
            description: "Swap keys and values",
            signature: "object -> object",
            example: "invert({a: 'x'}) -> {x: 'a'}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "rename_keys",
            category: Category::Object,
            description: "Rename object keys",
            signature: "object, object -> object",
            example: "rename_keys({a: 1}, {a: 'b'}) -> {b: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "flatten_keys",
            category: Category::Object,
            description: "Flatten nested object with dot notation keys",
            signature: "object -> object",
            example: "flatten_keys({a: {b: 1}}) -> {\"a.b\": 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn math_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "round",
            category: Category::Math,
            description: "Round to specified decimal places",
            signature: "number, number -> number",
            example: "round(`3.14159`, `2`) -> 3.14",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "floor_fn",
            category: Category::Math,
            description: "Round down to nearest integer",
            signature: "number -> number",
            example: "floor_fn(`3.7`) -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "ceil_fn",
            category: Category::Math,
            description: "Round up to nearest integer",
            signature: "number -> number",
            example: "ceil_fn(`3.2`) -> 4",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "abs_fn",
            category: Category::Math,
            description: "Absolute value",
            signature: "number -> number",
            example: "abs_fn(`-5`) -> 5",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "mod_fn",
            category: Category::Math,
            description: "Modulo operation",
            signature: "number, number -> number",
            example: "mod_fn(`10`, `3`) -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "pow",
            category: Category::Math,
            description: "Raise to power",
            signature: "number, number -> number",
            example: "pow(`2`, `3`) -> 8",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sqrt",
            category: Category::Math,
            description: "Square root",
            signature: "number -> number",
            example: "sqrt(`16`) -> 4",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "log",
            category: Category::Math,
            description: "Natural logarithm",
            signature: "number -> number",
            example: "log(`2.718`) -> ~1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "clamp",
            category: Category::Math,
            description: "Clamp value to range",
            signature: "number, number, number -> number",
            example: "clamp(`15`, `0`, `10`) -> 10",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "median",
            category: Category::Math,
            description: "Calculate median of array",
            signature: "array -> number",
            example: "median([1, 2, 3, 4, 5]) -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "percentile",
            category: Category::Math,
            description: "Calculate percentile of array",
            signature: "array, number -> number",
            example: "percentile([1, 2, 3, 4, 5], `50`) -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "variance",
            category: Category::Math,
            description: "Calculate variance of array",
            signature: "array -> number",
            example: "variance([1, 2, 3, 4, 5]) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "stddev",
            category: Category::Math,
            description: "Calculate standard deviation of array",
            signature: "array -> number",
            example: "stddev([1, 2, 3, 4, 5]) -> 1.414...",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sin",
            category: Category::Math,
            description: "Sine function",
            signature: "number -> number",
            example: "sin(`0`) -> 0",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "cos",
            category: Category::Math,
            description: "Cosine function",
            signature: "number -> number",
            example: "cos(`0`) -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "tan",
            category: Category::Math,
            description: "Tangent function",
            signature: "number -> number",
            example: "tan(`0`) -> 0",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "add",
            category: Category::Math,
            description: "Add two numbers",
            signature: "number, number -> number",
            example: "add(`2`, `3`) -> 5",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "subtract",
            category: Category::Math,
            description: "Subtract second number from first",
            signature: "number, number -> number",
            example: "subtract(`5`, `3`) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "multiply",
            category: Category::Math,
            description: "Multiply two numbers",
            signature: "number, number -> number",
            example: "multiply(`4`, `3`) -> 12",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "divide",
            category: Category::Math,
            description: "Divide first number by second",
            signature: "number, number -> number",
            example: "divide(`10`, `2`) -> 5",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "mode",
            category: Category::Math,
            description: "Find the most common value in an array",
            signature: "array -> any",
            example: "mode([1, 2, 2, 3]) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "to_fixed",
            category: Category::Math,
            description: "Format number with exact decimal places",
            signature: "number, number -> string",
            example: "to_fixed(`3.14159`, `2`) -> \"3.14\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "format_number",
            category: Category::Math,
            description: "Format number with separators and optional suffix",
            signature: "number, number?, string? -> string",
            example: "format_number(`1234567`, `0`) -> \"1,234,567\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn type_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "type_of",
            category: Category::Type,
            description: "Get the type of a value",
            signature: "any -> string",
            example: "type_of(`42`) -> \"number\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_string",
            category: Category::Type,
            description: "Check if value is a string",
            signature: "any -> boolean",
            example: "is_string('hello') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_number",
            category: Category::Type,
            description: "Check if value is a number",
            signature: "any -> boolean",
            example: "is_number(`42`) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_boolean",
            category: Category::Type,
            description: "Check if value is a boolean",
            signature: "any -> boolean",
            example: "is_boolean(`true`) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_array",
            category: Category::Type,
            description: "Check if value is an array",
            signature: "any -> boolean",
            example: "is_array([1, 2]) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_object",
            category: Category::Type,
            description: "Check if value is an object",
            signature: "any -> boolean",
            example: "is_object({a: 1}) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_null",
            category: Category::Type,
            description: "Check if value is null",
            signature: "any -> boolean",
            example: "is_null(`null`) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_empty",
            category: Category::Type,
            description: "Check if value is empty",
            signature: "any -> boolean",
            example: "is_empty([]) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "to_boolean",
            category: Category::Type,
            description: "Convert value to boolean",
            signature: "any -> boolean",
            example: "to_boolean('true') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn utility_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "default",
            category: Category::Utility,
            description: "Return default value if null",
            signature: "any, any -> any",
            example: "default(`null`, 'fallback') -> \"fallback\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "if",
            category: Category::Utility,
            description: "Conditional expression",
            signature: "boolean, any, any -> any",
            example: "if(`true`, 'yes', 'no') -> \"yes\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "coalesce",
            category: Category::Utility,
            description: "Return first non-null value",
            signature: "any... -> any",
            example: "coalesce(`null`, `null`, 'value') -> \"value\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "now",
            category: Category::Utility,
            description: "Current Unix timestamp in seconds",
            signature: "-> number",
            example: "now() -> 1699900000",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "now_ms",
            category: Category::Utility,
            description: "Current Unix timestamp in milliseconds",
            signature: "-> number",
            example: "now_ms() -> 1699900000000",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "json_decode",
            category: Category::Utility,
            description: "Parse JSON string",
            signature: "string -> any",
            example: "json_decode('{\"a\": 1}') -> {a: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "json_encode",
            category: Category::Utility,
            description: "Serialize value to JSON string",
            signature: "any -> string",
            example: "json_encode({a: 1}) -> \"{\\\"a\\\":1}\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "json_pointer",
            category: Category::Utility,
            description: "Access value using JSON Pointer (RFC 6901)",
            signature: "any, string -> any",
            example: "json_pointer({foo: {bar: 1}}, '/foo/bar') -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn validation_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "is_email",
            category: Category::Validation,
            description: "Validate email address format",
            signature: "string -> boolean",
            example: "is_email('user@example.com') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_url",
            category: Category::Validation,
            description: "Validate URL format",
            signature: "string -> boolean",
            example: "is_url('https://example.com') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_uuid",
            category: Category::Validation,
            description: "Validate UUID format",
            signature: "string -> boolean",
            example: "is_uuid('550e8400-e29b-41d4-a716-446655440000') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_ipv4",
            category: Category::Validation,
            description: "Validate IPv4 address format",
            signature: "string -> boolean",
            example: "is_ipv4('192.168.1.1') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_ipv6",
            category: Category::Validation,
            description: "Validate IPv6 address format",
            signature: "string -> boolean",
            example: "is_ipv6('::1') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn path_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "path_basename",
            category: Category::Path,
            description: "Get filename from path",
            signature: "string -> string",
            example: "path_basename('/foo/bar.txt') -> \"bar.txt\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "path_dirname",
            category: Category::Path,
            description: "Get directory from path",
            signature: "string -> string",
            example: "path_dirname('/foo/bar.txt') -> \"/foo\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "path_ext",
            category: Category::Path,
            description: "Get file extension",
            signature: "string -> string",
            example: "path_ext('/foo/bar.txt') -> \"txt\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "path_join",
            category: Category::Path,
            description: "Join path segments",
            signature: "string... -> string",
            example: "path_join('/foo', 'bar', 'baz') -> \"/foo/bar/baz\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn expression_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "map_expr",
            category: Category::Expression,
            description: "Apply a JMESPath expression to each element, returning transformed array",
            signature: "array, expression -> array",
            example: "map_expr([1, 2], &@ * `2`) -> [2, 4]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "filter_expr",
            category: Category::Expression,
            description: "Keep elements where JMESPath expression evaluates to truthy value",
            signature: "array, expression -> array",
            example: "filter_expr([1, 2, 3], &@ > `1`) -> [2, 3]",
            is_standard: false,
            jep: None,
            aliases: &["filter"],
            features: &[Feature::Core, Feature::Fp],
        },
        // Note: reject, map_keys, map_values will be added when PR #73 merges
        FunctionInfo {
            name: "any_expr",
            category: Category::Expression,
            description: "Return true if any element satisfies the expression (short-circuits)",
            signature: "array, expression -> boolean",
            example: "any_expr([1, 2, 3], &@ > `2`) -> true",
            is_standard: false,
            jep: None,
            aliases: &["some"],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "all_expr",
            category: Category::Expression,
            description: "Return true if every element satisfies the expression (short-circuits on false)",
            signature: "array, expression -> boolean",
            example: "all_expr([1, 2, 3], &@ > `0`) -> true",
            is_standard: false,
            jep: None,
            aliases: &["every"],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "every",
            category: Category::Expression,
            description: "Check if all elements match (alias for all_expr)",
            signature: "string, array -> boolean",
            example: "every('@ > `0`', [1, 2, 3]) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "some",
            category: Category::Expression,
            description: "Check if any element matches (alias for any_expr)",
            signature: "string, array -> boolean",
            example: "some('@ > `2`', [1, 2, 3]) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "reject",
            category: Category::Expression,
            description: "Keep elements where expression is falsy (inverse of filter_expr)",
            signature: "string, array -> array",
            example: "reject('@ > `2`', [1, 2, 3, 4]) -> [1, 2]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "map_keys",
            category: Category::Expression,
            description: "Transform object keys using expression",
            signature: "string, object -> object",
            example: "map_keys('upper(@)', {a: 1}) -> {A: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "map_values",
            category: Category::Expression,
            description: "Transform object values using expression",
            signature: "string, object -> object",
            example: "map_values('@ * `2`', {a: 1, b: 2}) -> {a: 2, b: 4}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core, Feature::Fp],
        },
        FunctionInfo {
            name: "find_expr",
            category: Category::Expression,
            description: "Return first element where expression is truthy, or null if none match",
            signature: "array, expression -> any",
            example: "find_expr([1, 2, 3], &@ > `1`) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "find_index_expr",
            category: Category::Expression,
            description: "Return zero-based index of first matching element, or -1 if none match",
            signature: "array, expression -> number | null",
            example: "find_index_expr([1, 2, 3], &@ > `1`) -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "count_expr",
            category: Category::Expression,
            description: "Count how many elements satisfy the expression",
            signature: "array, expression -> number",
            example: "count_expr([1, 2, 3], &@ > `1`) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sort_by_expr",
            category: Category::Expression,
            description: "Sort array by expression result in ascending order",
            signature: "array, expression -> array",
            example: "sort_by_expr([{a: 2}, {a: 1}], &a) -> [{a: 1}, {a: 2}]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "group_by_expr",
            category: Category::Expression,
            description: "Group elements into object keyed by expression result",
            signature: "array, expression -> object",
            example: "group_by_expr([{t: 'a'}, {t: 'b'}], &t) -> {a: [...], b: [...]}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "partition_expr",
            category: Category::Expression,
            description: "Split array into [matches, non-matches] based on expression",
            signature: "array, expression -> array",
            example: "partition_expr([1, 2, 3], &@ > `1`) -> [[2, 3], [1]]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "min_by_expr",
            category: Category::Expression,
            description: "Return element with smallest expression result, or null for empty array",
            signature: "array, expression -> any",
            example: "min_by_expr([{a: 2}, {a: 1}], &a) -> {a: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "max_by_expr",
            category: Category::Expression,
            description: "Return element with largest expression result, or null for empty array",
            signature: "array, expression -> any",
            example: "max_by_expr([{a: 2}, {a: 1}], &a) -> {a: 2}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "unique_by_expr",
            category: Category::Expression,
            description: "Remove duplicates by expression result, keeping first occurrence",
            signature: "array, expression -> array",
            example: "unique_by_expr([{a: 1}, {a: 1}], &a) -> [{a: 1}]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "flat_map_expr",
            category: Category::Expression,
            description: "Apply expression to each element and flatten all results into one array",
            signature: "array, expression -> array",
            example: "flat_map_expr([[1], [2]], &@) -> [1, 2]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn text_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "word_count",
            category: Category::Text,
            description: "Count words in text",
            signature: "string -> number",
            example: "word_count('hello world') -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "char_count",
            category: Category::Text,
            description: "Count characters in text",
            signature: "string -> number",
            example: "char_count('hello') -> 5",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sentence_count",
            category: Category::Text,
            description: "Count sentences in text",
            signature: "string -> number",
            example: "sentence_count('Hello. World!') -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "paragraph_count",
            category: Category::Text,
            description: "Count paragraphs in text",
            signature: "string -> number",
            example: "paragraph_count('A\\n\\nB') -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "reading_time",
            category: Category::Text,
            description: "Estimate reading time",
            signature: "string -> string",
            example: "reading_time('...long text...') -> \"2 min read\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "reading_time_seconds",
            category: Category::Text,
            description: "Estimate reading time in seconds",
            signature: "string -> number",
            example: "reading_time_seconds('...text...') -> 120",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "char_frequencies",
            category: Category::Text,
            description: "Count character frequencies",
            signature: "string -> object",
            example: "char_frequencies('aab') -> {a: 2, b: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "word_frequencies",
            category: Category::Text,
            description: "Count word frequencies",
            signature: "string -> object",
            example: "word_frequencies('a a b') -> {a: 2, b: 1}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn hash_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "md5",
            category: Category::Hash,
            description: "Calculate MD5 hash",
            signature: "string -> string",
            example: "md5('hello') -> \"5d41402abc4b2a76b9719d911017c592\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sha1",
            category: Category::Hash,
            description: "Calculate SHA-1 hash",
            signature: "string -> string",
            example: "sha1('hello') -> \"aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sha256",
            category: Category::Hash,
            description: "Calculate SHA-256 hash",
            signature: "string -> string",
            example: "sha256('hello') -> \"2cf24dba...\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "crc32",
            category: Category::Hash,
            description: "Calculate CRC32 checksum",
            signature: "string -> number",
            example: "crc32('hello') -> 907060870",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn encoding_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "base64_encode",
            category: Category::Encoding,
            description: "Encode string to base64",
            signature: "string -> string",
            example: "base64_encode('hello') -> \"aGVsbG8=\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "base64_decode",
            category: Category::Encoding,
            description: "Decode base64 string",
            signature: "string -> string",
            example: "base64_decode('aGVsbG8=') -> \"hello\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "hex_encode",
            category: Category::Encoding,
            description: "Encode string to hex",
            signature: "string -> string",
            example: "hex_encode('hello') -> \"68656c6c6f\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "hex_decode",
            category: Category::Encoding,
            description: "Decode hex string",
            signature: "string -> string",
            example: "hex_decode('68656c6c6f') -> \"hello\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn regex_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "regex_match",
            category: Category::Regex,
            description: "Test if string matches regex",
            signature: "string, string -> boolean",
            example: "regex_match('hello', '^h.*o$') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "regex_extract",
            category: Category::Regex,
            description: "Extract regex matches",
            signature: "string, string -> array",
            example: "regex_extract('a1b2', '\\\\d+') -> [\"1\", \"2\"]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "regex_replace",
            category: Category::Regex,
            description: "Replace regex matches",
            signature: "string, string, string -> string",
            example: "regex_replace('a1b2', '\\\\d+', 'X') -> \"aXbX\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn url_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "url_encode",
            category: Category::Url,
            description: "URL encode a string",
            signature: "string -> string",
            example: "url_encode('hello world') -> \"hello%20world\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "url_decode",
            category: Category::Url,
            description: "URL decode a string",
            signature: "string -> string",
            example: "url_decode('hello%20world') -> \"hello world\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "url_parse",
            category: Category::Url,
            description: "Parse URL into components",
            signature: "string -> object",
            example: "url_parse('https://example.com/path') -> {scheme: 'https', ...}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn uuid_functions() -> Vec<FunctionInfo> {
    vec![FunctionInfo {
        name: "uuid",
        category: Category::Uuid,
        description: "Generate a UUID v4",
        signature: "-> string",
        example: "uuid() -> \"550e8400-e29b-41d4-a716-446655440000\"",
        is_standard: false,
        jep: None,
        aliases: &[],
        features: &[Feature::Core],
    }]
}

fn rand_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "random",
            category: Category::Rand,
            description: "Generate random number between 0 and 1",
            signature: "-> number",
            example: "random() -> 0.123456...",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "shuffle",
            category: Category::Rand,
            description: "Randomly shuffle array",
            signature: "array -> array",
            example: "shuffle([1, 2, 3]) -> [2, 3, 1]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sample",
            category: Category::Rand,
            description: "Random sample from array",
            signature: "array, number -> array",
            example: "sample([1, 2, 3, 4], `2`) -> [3, 1]",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn datetime_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "parse_date",
            category: Category::Datetime,
            description: "Parse date string to timestamp",
            signature: "string, string? -> number",
            example: "parse_date('2024-01-15', '%Y-%m-%d') -> 1705276800",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "format_date",
            category: Category::Datetime,
            description: "Format timestamp to string",
            signature: "number, string -> string",
            example: "format_date(`1705276800`, '%Y-%m-%d') -> \"2024-01-15\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "date_add",
            category: Category::Datetime,
            description: "Add time to timestamp",
            signature: "number, number, string -> number",
            example: "date_add(`0`, `1`, 'days') -> 86400",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "date_diff",
            category: Category::Datetime,
            description: "Difference between timestamps",
            signature: "number, number, string -> number",
            example: "date_diff(`86400`, `0`, 'days') -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "timezone_convert",
            category: Category::Datetime,
            description: "Convert timestamp between timezones (IANA timezone names)",
            signature: "string, string, string -> string",
            example: "timezone_convert('2024-01-15T10:00:00', 'America/New_York', 'Europe/London') -> \"2024-01-15T15:00:00\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_weekend",
            category: Category::Datetime,
            description: "Check if timestamp falls on weekend (Saturday or Sunday)",
            signature: "number -> boolean",
            example: "is_weekend(`1705104000`) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_weekday",
            category: Category::Datetime,
            description: "Check if timestamp falls on weekday (Monday-Friday)",
            signature: "number -> boolean",
            example: "is_weekday(`1705276800`) -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "business_days_between",
            category: Category::Datetime,
            description: "Count business days (weekdays) between two timestamps",
            signature: "number, number -> number",
            example: "business_days_between(`1704067200`, `1705276800`) -> 10",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "relative_time",
            category: Category::Datetime,
            description: "Human-readable relative time from timestamp",
            signature: "number -> string",
            example: "relative_time(now() - 3600) -> \"1 hour ago\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "quarter",
            category: Category::Datetime,
            description: "Get quarter of year (1-4) from timestamp",
            signature: "number -> number",
            example: "quarter(`1713139200`) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn fuzzy_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "levenshtein",
            category: Category::Fuzzy,
            description: "Levenshtein edit distance",
            signature: "string, string -> number",
            example: "levenshtein('kitten', 'sitting') -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "normalized_levenshtein",
            category: Category::Fuzzy,
            description: "Normalized Levenshtein (0-1)",
            signature: "string, string -> number",
            example: "normalized_levenshtein('ab', 'abc') -> 0.666...",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "damerau_levenshtein",
            category: Category::Fuzzy,
            description: "Damerau-Levenshtein distance",
            signature: "string, string -> number",
            example: "damerau_levenshtein('ab', 'ba') -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "jaro",
            category: Category::Fuzzy,
            description: "Jaro similarity (0-1)",
            signature: "string, string -> number",
            example: "jaro('hello', 'hallo') -> 0.866...",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "jaro_winkler",
            category: Category::Fuzzy,
            description: "Jaro-Winkler similarity (0-1)",
            signature: "string, string -> number",
            example: "jaro_winkler('hello', 'hallo') -> 0.88",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sorensen_dice",
            category: Category::Fuzzy,
            description: "Sorensen-Dice coefficient (0-1)",
            signature: "string, string -> number",
            example: "sorensen_dice('night', 'nacht') -> 0.25",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn phonetic_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "soundex",
            category: Category::Phonetic,
            description: "Soundex phonetic code",
            signature: "string -> string",
            example: "soundex('Robert') -> \"R163\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "metaphone",
            category: Category::Phonetic,
            description: "Metaphone phonetic code",
            signature: "string -> string",
            example: "metaphone('Smith') -> \"SM0\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "double_metaphone",
            category: Category::Phonetic,
            description: "Double Metaphone codes",
            signature: "string -> object",
            example: "double_metaphone('Smith') -> {primary: 'SM0', secondary: 'XMT'}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "nysiis",
            category: Category::Phonetic,
            description: "NYSIIS phonetic code",
            signature: "string -> string",
            example: "nysiis('Smith') -> \"SNAT\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "match_rating_codex",
            category: Category::Phonetic,
            description: "Match Rating codex",
            signature: "string -> string",
            example: "match_rating_codex('Smith') -> \"SMTH\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "caverphone",
            category: Category::Phonetic,
            description: "Caverphone code",
            signature: "string -> string",
            example: "caverphone('Smith') -> \"SMT1111111\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "caverphone2",
            category: Category::Phonetic,
            description: "Caverphone 2 code",
            signature: "string -> string",
            example: "caverphone2('Smith') -> \"SMT1111111\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "sounds_like",
            category: Category::Phonetic,
            description: "Check if strings sound similar",
            signature: "string, string -> boolean",
            example: "sounds_like('Robert', 'Rupert') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "phonetic_match",
            category: Category::Phonetic,
            description: "Check phonetic match with algorithm",
            signature: "string, string, string -> boolean",
            example: "phonetic_match('Smith', 'Smyth', 'soundex') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn geo_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "geo_distance",
            category: Category::Geo,
            description: "Haversine distance in meters",
            signature: "number, number, number, number -> number",
            example: "geo_distance(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 5570222",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "geo_distance_km",
            category: Category::Geo,
            description: "Haversine distance in kilometers",
            signature: "number, number, number, number -> number",
            example: "geo_distance_km(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 5570.2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "geo_distance_miles",
            category: Category::Geo,
            description: "Haversine distance in miles",
            signature: "number, number, number, number -> number",
            example: "geo_distance_miles(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 3461.0",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "geo_bearing",
            category: Category::Geo,
            description: "Bearing between coordinates",
            signature: "number, number, number, number -> number",
            example: "geo_bearing(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) -> 51.2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn semver_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "semver_parse",
            category: Category::Semver,
            description: "Parse semantic version",
            signature: "string -> object",
            example: "semver_parse('1.2.3') -> {major: 1, minor: 2, patch: 3}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "semver_major",
            category: Category::Semver,
            description: "Get major version",
            signature: "string -> number",
            example: "semver_major('1.2.3') -> 1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "semver_minor",
            category: Category::Semver,
            description: "Get minor version",
            signature: "string -> number",
            example: "semver_minor('1.2.3') -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "semver_patch",
            category: Category::Semver,
            description: "Get patch version",
            signature: "string -> number",
            example: "semver_patch('1.2.3') -> 3",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "semver_compare",
            category: Category::Semver,
            description: "Compare versions (-1, 0, 1)",
            signature: "string, string -> number",
            example: "semver_compare('1.0.0', '2.0.0') -> -1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "semver_satisfies",
            category: Category::Semver,
            description: "Check if version matches constraint",
            signature: "string, string -> boolean",
            example: "semver_satisfies('1.2.3', '^1.0.0') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "semver_is_valid",
            category: Category::Semver,
            description: "Check if string is valid semver",
            signature: "string -> boolean",
            example: "semver_is_valid('1.2.3') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn network_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "ip_to_int",
            category: Category::Network,
            description: "Convert IP address to integer",
            signature: "string -> number",
            example: "ip_to_int('192.168.1.1') -> 3232235777",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "int_to_ip",
            category: Category::Network,
            description: "Convert integer to IP address",
            signature: "number -> string",
            example: "int_to_ip(`3232235777`) -> \"192.168.1.1\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "cidr_contains",
            category: Category::Network,
            description: "Check if IP is in CIDR range",
            signature: "string, string -> boolean",
            example: "cidr_contains('192.168.0.0/16', '192.168.1.1') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "cidr_network",
            category: Category::Network,
            description: "Get network address from CIDR",
            signature: "string -> string",
            example: "cidr_network('192.168.1.0/24') -> \"192.168.1.0\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "cidr_broadcast",
            category: Category::Network,
            description: "Get broadcast address from CIDR",
            signature: "string -> string",
            example: "cidr_broadcast('192.168.1.0/24') -> \"192.168.1.255\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "cidr_prefix",
            category: Category::Network,
            description: "Get prefix length from CIDR",
            signature: "string -> number",
            example: "cidr_prefix('192.168.1.0/24') -> 24",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "is_private_ip",
            category: Category::Network,
            description: "Check if IP is in private range",
            signature: "string -> boolean",
            example: "is_private_ip('192.168.1.1') -> true",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn ids_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "nanoid",
            category: Category::Ids,
            description: "Generate nanoid",
            signature: "number? -> string",
            example: "nanoid() -> \"V1StGXR8_Z5jdHi6B-myT\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "ulid",
            category: Category::Ids,
            description: "Generate ULID",
            signature: "-> string",
            example: "ulid() -> \"01ARZ3NDEKTSV4RRFFQ69G5FAV\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "ulid_timestamp",
            category: Category::Ids,
            description: "Extract timestamp from ULID",
            signature: "string -> number",
            example: "ulid_timestamp('01ARZ3NDEKTSV4RRFFQ69G5FAV') -> 1469918176385",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn duration_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "parse_duration",
            category: Category::Duration,
            description: "Parse duration string to seconds",
            signature: "string -> number",
            example: "parse_duration('1h30m') -> 5400",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "format_duration",
            category: Category::Duration,
            description: "Format seconds as duration string",
            signature: "number -> string",
            example: "format_duration(`5400`) -> \"1h30m\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "duration_hours",
            category: Category::Duration,
            description: "Convert seconds to hours",
            signature: "number -> number",
            example: "duration_hours(`7200`) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "duration_minutes",
            category: Category::Duration,
            description: "Convert seconds to minutes",
            signature: "number -> number",
            example: "duration_minutes(`120`) -> 2",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "duration_seconds",
            category: Category::Duration,
            description: "Get seconds component",
            signature: "number -> number",
            example: "duration_seconds(`65`) -> 5",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn color_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "hex_to_rgb",
            category: Category::Color,
            description: "Convert hex color to RGB",
            signature: "string -> object",
            example: "hex_to_rgb('#ff5500') -> {r: 255, g: 85, b: 0}",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "rgb_to_hex",
            category: Category::Color,
            description: "Convert RGB to hex color",
            signature: "number, number, number -> string",
            example: "rgb_to_hex(`255`, `85`, `0`) -> \"#ff5500\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "lighten",
            category: Category::Color,
            description: "Lighten a color by percentage",
            signature: "string, number -> string",
            example: "lighten('#3366cc', `20`) -> \"#5c85d6\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "darken",
            category: Category::Color,
            description: "Darken a color by percentage",
            signature: "string, number -> string",
            example: "darken('#3366cc', `20`) -> \"#2952a3\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "color_mix",
            category: Category::Color,
            description: "Mix two colors",
            signature: "string, string, number -> string",
            example: "color_mix('#ff0000', '#0000ff', `50`) -> \"#800080\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "color_invert",
            category: Category::Color,
            description: "Invert a color",
            signature: "string -> string",
            example: "color_invert('#ff0000') -> \"#00ffff\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "color_grayscale",
            category: Category::Color,
            description: "Convert to grayscale",
            signature: "string -> string",
            example: "color_grayscale('#ff0000') -> \"#4c4c4c\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "color_complement",
            category: Category::Color,
            description: "Get complementary color",
            signature: "string -> string",
            example: "color_complement('#ff0000') -> \"#00ffff\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

fn computing_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo {
            name: "parse_bytes",
            category: Category::Computing,
            description: "Parse byte size string",
            signature: "string -> number",
            example: "parse_bytes('1.5 GB') -> 1500000000",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "format_bytes",
            category: Category::Computing,
            description: "Format bytes (decimal)",
            signature: "number -> string",
            example: "format_bytes(`1500000000`) -> \"1.50 GB\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "format_bytes_binary",
            category: Category::Computing,
            description: "Format bytes (binary)",
            signature: "number -> string",
            example: "format_bytes_binary(`1073741824`) -> \"1.00 GiB\"",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "bit_and",
            category: Category::Computing,
            description: "Bitwise AND",
            signature: "number, number -> number",
            example: "bit_and(`12`, `10`) -> 8",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "bit_or",
            category: Category::Computing,
            description: "Bitwise OR",
            signature: "number, number -> number",
            example: "bit_or(`12`, `10`) -> 14",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "bit_xor",
            category: Category::Computing,
            description: "Bitwise XOR",
            signature: "number, number -> number",
            example: "bit_xor(`12`, `10`) -> 6",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "bit_not",
            category: Category::Computing,
            description: "Bitwise NOT",
            signature: "number -> number",
            example: "bit_not(`0`) -> -1",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "bit_shift_left",
            category: Category::Computing,
            description: "Bitwise left shift",
            signature: "number, number -> number",
            example: "bit_shift_left(`1`, `4`) -> 16",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
        FunctionInfo {
            name: "bit_shift_right",
            category: Category::Computing,
            description: "Bitwise right shift",
            signature: "number, number -> number",
            example: "bit_shift_right(`16`, `2`) -> 4",
            is_standard: false,
            jep: None,
            aliases: &[],
            features: &[Feature::Core],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry() {
        let registry = FunctionRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_category() {
        // Use Standard category which is always available
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);
        assert!(!registry.is_empty());
        assert!(registry.is_enabled("abs"));
        assert!(registry.is_enabled("length"));
    }

    #[test]
    fn test_disable_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);
        assert!(registry.is_enabled("abs"));

        registry.disable_function("abs");
        assert!(!registry.is_enabled("abs"));
        assert!(registry.is_enabled("length")); // others still enabled
    }

    #[test]
    fn test_enable_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);
        registry.disable_function("abs");
        assert!(!registry.is_enabled("abs"));

        registry.enable_function("abs");
        assert!(registry.is_enabled("abs"));
    }

    #[test]
    fn test_get_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        let info = registry.get_function("abs").unwrap();
        assert_eq!(info.name, "abs");
        assert_eq!(info.category, Category::Standard);
        assert!(info.is_standard);
    }

    #[test]
    fn test_functions_in_category() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        let standard_fns: Vec<_> = registry.functions_in_category(Category::Standard).collect();
        assert!(
            standard_fns
                .iter()
                .all(|f| f.category == Category::Standard)
        );
        assert_eq!(standard_fns.len(), 26); // 26 standard JMESPath functions
    }

    #[test]
    fn test_register_all() {
        let mut registry = FunctionRegistry::new();
        registry.register_all();
        // Should have at least standard functions (26) plus any enabled features
        assert!(registry.len() >= 26);
    }

    #[test]
    fn test_apply_to_runtime() {
        // Standard functions are registered via runtime.register_builtin_functions()
        // not via registry.apply(), so we test with an extension if available
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        registry.apply(&mut runtime);

        // Verify standard function works (registered by register_builtin_functions)
        let expr = runtime.compile("abs(`-5`)").unwrap();
        let data = jmespath::Variable::Null;
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);
    }

    #[test]
    #[cfg(feature = "string")]
    fn test_apply_extension_to_runtime() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::String);

        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        registry.apply(&mut runtime);

        // Verify extension function works
        let expr = runtime.compile("upper('hello')").unwrap();
        let data = jmespath::Variable::Null;
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "HELLO");
    }

    #[test]
    fn test_functions_with_feature_spec() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        let spec_fns: Vec<_> = registry.functions_with_feature(Feature::Spec).collect();
        assert_eq!(spec_fns.len(), 26); // All 26 standard functions have Spec feature
        assert!(spec_fns.iter().all(|f| f.is_standard));
    }

    #[test]
    fn test_is_spec_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_all();

        // Standard functions should be spec
        assert!(registry.is_spec_function("abs"));
        assert!(registry.is_spec_function("length"));
        assert!(registry.is_spec_function("sort"));

        // Extension functions should not be spec
        #[cfg(feature = "string")]
        assert!(!registry.is_spec_function("upper"));
        #[cfg(feature = "array")]
        assert!(!registry.is_spec_function("unique"));
    }

    #[test]
    fn test_spec_function_names() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        let names: Vec<_> = registry.spec_function_names().collect();
        assert!(names.contains(&"abs"));
        assert!(names.contains(&"length"));
        assert!(names.contains(&"sort"));
        assert_eq!(names.len(), 26);
    }

    #[test]
    fn test_function_features() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        let abs_info = registry.get_function("abs").unwrap();
        assert!(abs_info.features.contains(&Feature::Spec));
        assert!(!abs_info.features.contains(&Feature::Fp));
    }

    #[test]
    #[cfg(feature = "string")]
    fn test_jep_functions_have_jep_feature() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::String);

        let upper_info = registry.get_function("upper").unwrap();
        assert!(upper_info.features.contains(&Feature::Jep));
        assert!(upper_info.jep.is_some());
    }

    #[test]
    fn test_function_aliases_field() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Standard);

        // Standard functions have no aliases by default
        let abs_info = registry.get_function("abs").unwrap();
        assert!(abs_info.aliases.is_empty());
    }

    #[test]
    fn test_feature_all() {
        let features = Feature::all();
        assert!(features.contains(&Feature::Spec));
        assert!(features.contains(&Feature::Core));
        assert!(features.contains(&Feature::Fp));
        assert!(features.contains(&Feature::Jep));
    }

    #[test]
    fn test_feature_name() {
        assert_eq!(Feature::Spec.name(), "spec");
        assert_eq!(Feature::Core.name(), "core");
        assert_eq!(Feature::Fp.name(), "fp");
        assert_eq!(Feature::Jep.name(), "jep");
    }

    #[test]
    #[cfg(feature = "expression")]
    fn test_function_with_aliases() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Expression);

        // any_expr has alias "some"
        let any_expr_info = registry.get_function("any_expr").unwrap();
        assert!(any_expr_info.aliases.contains(&"some"));
        assert!(any_expr_info.features.contains(&Feature::Fp));

        // all_expr has alias "every"
        let all_expr_info = registry.get_function("all_expr").unwrap();
        assert!(all_expr_info.aliases.contains(&"every"));
        assert!(all_expr_info.features.contains(&Feature::Fp));
    }

    #[test]
    #[cfg(feature = "expression")]
    fn test_get_function_by_alias() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Expression);

        // Look up by alias - filter_expr has alias "filter"
        let info = registry.get_function_by_name_or_alias("filter").unwrap();
        assert_eq!(info.name, "filter_expr");

        // Direct lookup works
        let info = registry.get_function_by_name_or_alias("any_expr").unwrap();
        assert_eq!(info.name, "any_expr");

        // some and every are standalone functions (not just aliases)
        let info = registry.get_function_by_name_or_alias("some").unwrap();
        assert_eq!(info.name, "some");

        let info = registry.get_function_by_name_or_alias("every").unwrap();
        assert_eq!(info.name, "every");
    }

    #[test]
    #[cfg(feature = "expression")]
    fn test_all_aliases() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Expression);

        let aliases: Vec<_> = registry.all_aliases().collect();
        assert!(
            aliases
                .iter()
                .any(|(alias, name)| *alias == "some" && *name == "any_expr")
        );
        assert!(
            aliases
                .iter()
                .any(|(alias, name)| *alias == "every" && *name == "all_expr")
        );
    }

    #[test]
    #[cfg(feature = "expression")]
    fn test_fp_functions() {
        let mut registry = FunctionRegistry::new();
        registry.register_category(Category::Expression);

        let fp_fns: Vec<_> = registry.functions_with_feature(Feature::Fp).collect();
        let fp_names: Vec<_> = fp_fns.iter().map(|f| f.name).collect();

        assert!(fp_names.contains(&"any_expr"));
        assert!(fp_names.contains(&"all_expr"));
        assert!(fp_names.contains(&"filter_expr"));
        // reject will be added when PR #73 merges
    }
}
