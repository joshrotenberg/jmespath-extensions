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
    MultiMatch,
    Jsonpatch,
    Format,
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
            Category::MultiMatch,
            Category::Jsonpatch,
            Category::Format,
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
            Category::MultiMatch => "multi-match",
            Category::Jsonpatch => "jsonpatch",
            Category::Format => "format",
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
            #[cfg(feature = "multi-match")]
            Category::MultiMatch => true,
            #[cfg(feature = "jsonpatch")]
            Category::Jsonpatch => true,
            #[cfg(feature = "format")]
            Category::Format => true,
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
    /// Format output functions (CSV, TSV)
    #[allow(non_camel_case_types)]
    format,
    /// Environment variable access (opt-in for security)
    #[allow(non_camel_case_types)]
    env,
}

impl Feature {
    /// Returns all features
    pub fn all() -> &'static [Feature] {
        &[
            Feature::Spec,
            Feature::Core,
            Feature::Fp,
            Feature::Jep,
            Feature::format,
            Feature::env,
        ]
    }

    /// Returns the feature name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Feature::Spec => "spec",
            Feature::Core => "core",
            Feature::Fp => "fp",
            Feature::Jep => "jep",
            Feature::format => "format",
            Feature::env => "env",
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
            #[cfg(feature = "multi-match")]
            Category::MultiMatch => crate::multi_match::register(runtime),
            #[cfg(feature = "jsonpatch")]
            Category::Jsonpatch => crate::jsonpatch::register(runtime),
            #[cfg(feature = "format")]
            Category::Format => crate::format::register(runtime),
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
}

/// Get function metadata for a category (from generated data)
fn get_category_functions(category: Category) -> Vec<FunctionInfo> {
    generated::FUNCTIONS
        .iter()
        .filter(|f| f.category == category)
        .cloned()
        .collect()
}

// Include the generated function data from build.rs
mod generated {
    include!(concat!(env!("OUT_DIR"), "/registry_data.rs"));
}
