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
    Language,
    Discovery,
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
            Category::Language,
            Category::Discovery,
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
            Category::Language => "language",
            Category::Discovery => "discovery",
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
            #[cfg(feature = "language")]
            Category::Language => true,
            #[cfg(feature = "discovery")]
            Category::Discovery => true,
            // Explicit false cases for when features are disabled.
            // This ensures compile errors if a new category is added without handling it here.
            #[cfg(not(feature = "string"))]
            Category::String => false,
            #[cfg(not(feature = "array"))]
            Category::Array => false,
            #[cfg(not(feature = "object"))]
            Category::Object => false,
            #[cfg(not(feature = "math"))]
            Category::Math => false,
            #[cfg(not(feature = "type"))]
            Category::Type => false,
            #[cfg(not(feature = "utility"))]
            Category::Utility => false,
            #[cfg(not(feature = "validation"))]
            Category::Validation => false,
            #[cfg(not(feature = "path"))]
            Category::Path => false,
            #[cfg(not(feature = "expression"))]
            Category::Expression => false,
            #[cfg(not(feature = "text"))]
            Category::Text => false,
            #[cfg(not(feature = "hash"))]
            Category::Hash => false,
            #[cfg(not(feature = "encoding"))]
            Category::Encoding => false,
            #[cfg(not(feature = "regex"))]
            Category::Regex => false,
            #[cfg(not(feature = "url"))]
            Category::Url => false,
            #[cfg(not(feature = "uuid"))]
            Category::Uuid => false,
            #[cfg(not(feature = "rand"))]
            Category::Rand => false,
            #[cfg(not(feature = "datetime"))]
            Category::Datetime => false,
            #[cfg(not(feature = "fuzzy"))]
            Category::Fuzzy => false,
            #[cfg(not(feature = "phonetic"))]
            Category::Phonetic => false,
            #[cfg(not(feature = "geo"))]
            Category::Geo => false,
            #[cfg(not(feature = "semver"))]
            Category::Semver => false,
            #[cfg(not(feature = "network"))]
            Category::Network => false,
            #[cfg(not(feature = "ids"))]
            Category::Ids => false,
            #[cfg(not(feature = "duration"))]
            Category::Duration => false,
            #[cfg(not(feature = "color"))]
            Category::Color => false,
            #[cfg(not(feature = "computing"))]
            Category::Computing => false,
            #[cfg(not(feature = "multi-match"))]
            Category::MultiMatch => false,
            #[cfg(not(feature = "jsonpatch"))]
            Category::Jsonpatch => false,
            #[cfg(not(feature = "format"))]
            Category::Format => false,
            #[cfg(not(feature = "language"))]
            Category::Language => false,
            #[cfg(not(feature = "discovery"))]
            Category::Discovery => false,
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
    /// Discovery/search functions for tool discovery
    #[allow(non_camel_case_types)]
    discovery,
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
            Feature::discovery,
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
            Feature::discovery => "discovery",
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
        // Check which functions in this category are enabled (not disabled)
        let enabled_in_category: HashSet<&str> = self
            .functions_in_category(category)
            .map(|f| f.name)
            .collect();

        if enabled_in_category.is_empty() {
            return;
        }

        // Pass the enabled set to each category's register function
        // Only functions in this set will be registered
        match category {
            #[cfg(feature = "string")]
            Category::String => crate::string::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "array")]
            Category::Array => crate::array::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "object")]
            Category::Object => crate::object::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "math")]
            Category::Math => crate::math::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "type")]
            Category::Type => crate::type_conv::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "utility")]
            Category::Utility => crate::utility::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "validation")]
            Category::Validation => {
                crate::validation::register_filtered(runtime, &enabled_in_category)
            }
            #[cfg(feature = "path")]
            Category::Path => crate::path::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "expression")]
            Category::Expression => {
                crate::expression::register_filtered(runtime, &enabled_in_category)
            }
            #[cfg(feature = "text")]
            Category::Text => crate::text::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "hash")]
            Category::Hash => crate::hash::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "encoding")]
            Category::Encoding => crate::encoding::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "regex")]
            Category::Regex => crate::regex_fns::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "url")]
            Category::Url => crate::url_fns::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "uuid")]
            Category::Uuid => crate::random::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "rand")]
            Category::Rand => crate::random::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "datetime")]
            Category::Datetime => crate::datetime::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "fuzzy")]
            Category::Fuzzy => crate::fuzzy::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "phonetic")]
            Category::Phonetic => crate::phonetic::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "geo")]
            Category::Geo => crate::geo::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "semver")]
            Category::Semver => crate::semver_fns::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "network")]
            Category::Network => crate::network::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "ids")]
            Category::Ids => crate::ids::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "duration")]
            Category::Duration => crate::duration::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "color")]
            Category::Color => crate::color::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "computing")]
            Category::Computing => {
                crate::computing::register_filtered(runtime, &enabled_in_category)
            }
            #[cfg(feature = "multi-match")]
            Category::MultiMatch => {
                crate::multi_match::register_filtered(runtime, &enabled_in_category)
            }
            #[cfg(feature = "jsonpatch")]
            Category::Jsonpatch => {
                crate::jsonpatch::register_filtered(runtime, &enabled_in_category)
            }
            #[cfg(feature = "format")]
            Category::Format => crate::format::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "language")]
            Category::Language => crate::language::register_filtered(runtime, &enabled_in_category),
            #[cfg(feature = "discovery")]
            Category::Discovery => {
                crate::discovery::register_filtered(runtime, &enabled_in_category)
            }
            // Explicit no-op cases for when features are disabled.
            // This ensures compile errors if a new category is added without handling it here.
            Category::Standard => {} // Standard functions are registered via runtime.register_builtin_functions()
            #[cfg(not(feature = "string"))]
            Category::String => {}
            #[cfg(not(feature = "array"))]
            Category::Array => {}
            #[cfg(not(feature = "object"))]
            Category::Object => {}
            #[cfg(not(feature = "math"))]
            Category::Math => {}
            #[cfg(not(feature = "type"))]
            Category::Type => {}
            #[cfg(not(feature = "utility"))]
            Category::Utility => {}
            #[cfg(not(feature = "validation"))]
            Category::Validation => {}
            #[cfg(not(feature = "path"))]
            Category::Path => {}
            #[cfg(not(feature = "expression"))]
            Category::Expression => {}
            #[cfg(not(feature = "text"))]
            Category::Text => {}
            #[cfg(not(feature = "hash"))]
            Category::Hash => {}
            #[cfg(not(feature = "encoding"))]
            Category::Encoding => {}
            #[cfg(not(feature = "regex"))]
            Category::Regex => {}
            #[cfg(not(feature = "url"))]
            Category::Url => {}
            #[cfg(not(feature = "uuid"))]
            Category::Uuid => {}
            #[cfg(not(feature = "rand"))]
            Category::Rand => {}
            #[cfg(not(feature = "datetime"))]
            Category::Datetime => {}
            #[cfg(not(feature = "fuzzy"))]
            Category::Fuzzy => {}
            #[cfg(not(feature = "phonetic"))]
            Category::Phonetic => {}
            #[cfg(not(feature = "geo"))]
            Category::Geo => {}
            #[cfg(not(feature = "semver"))]
            Category::Semver => {}
            #[cfg(not(feature = "network"))]
            Category::Network => {}
            #[cfg(not(feature = "ids"))]
            Category::Ids => {}
            #[cfg(not(feature = "duration"))]
            Category::Duration => {}
            #[cfg(not(feature = "color"))]
            Category::Color => {}
            #[cfg(not(feature = "computing"))]
            Category::Computing => {}
            #[cfg(not(feature = "multi-match"))]
            Category::MultiMatch => {}
            #[cfg(not(feature = "jsonpatch"))]
            Category::Jsonpatch => {}
            #[cfg(not(feature = "format"))]
            Category::Format => {}
            #[cfg(not(feature = "language"))]
            Category::Language => {}
            #[cfg(not(feature = "discovery"))]
            Category::Discovery => {}
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

// =============================================================================
// Synonym mapping for function discovery
// =============================================================================

/// Synonym entry mapping a search term to related function names/keywords
pub struct SynonymEntry {
    /// The search term (e.g., "aggregate")
    pub term: &'static str,
    /// Related function names or keywords that should match
    pub targets: &'static [&'static str],
}

/// Get all synonym mappings for function discovery
///
/// Maps common search terms to function names/keywords that should match.
/// Useful for improving search when users don't know exact function names.
///
/// # Example
///
/// ```
/// use jmespath_extensions::registry::get_synonyms;
///
/// let synonyms = get_synonyms();
/// // Find what "aggregate" maps to
/// if let Some(entry) = synonyms.iter().find(|s| s.term == "aggregate") {
///     assert!(entry.targets.contains(&"group_by"));
/// }
/// ```
pub fn get_synonyms() -> &'static [SynonymEntry] {
    static SYNONYMS: &[SynonymEntry] = &[
        // Aggregation/grouping synonyms
        SynonymEntry {
            term: "aggregate",
            targets: &[
                "group_by",
                "group_by_expr",
                "sum",
                "avg",
                "count",
                "reduce",
                "fold",
            ],
        },
        SynonymEntry {
            term: "group",
            targets: &["group_by", "group_by_expr", "chunk", "partition"],
        },
        SynonymEntry {
            term: "collect",
            targets: &["group_by", "group_by_expr", "flatten", "merge"],
        },
        SynonymEntry {
            term: "bucket",
            targets: &["group_by", "group_by_expr", "chunk"],
        },
        // Counting synonyms
        SynonymEntry {
            term: "count",
            targets: &["length", "count_by", "count_if", "size"],
        },
        SynonymEntry {
            term: "size",
            targets: &["length", "count_by"],
        },
        SynonymEntry {
            term: "len",
            targets: &["length"],
        },
        // String manipulation synonyms
        SynonymEntry {
            term: "concat",
            targets: &["join", "merge", "combine"],
        },
        SynonymEntry {
            term: "combine",
            targets: &["join", "merge", "concat"],
        },
        SynonymEntry {
            term: "substring",
            targets: &["slice", "substr", "mid"],
        },
        SynonymEntry {
            term: "cut",
            targets: &["slice", "split", "trim"],
        },
        SynonymEntry {
            term: "strip",
            targets: &["trim", "trim_left", "trim_right"],
        },
        SynonymEntry {
            term: "lowercase",
            targets: &["lower", "to_lower", "downcase"],
        },
        SynonymEntry {
            term: "uppercase",
            targets: &["upper", "to_upper", "upcase"],
        },
        SynonymEntry {
            term: "replace",
            targets: &["substitute", "gsub", "regex_replace"],
        },
        SynonymEntry {
            term: "find",
            targets: &["contains", "index_of", "search", "match"],
        },
        SynonymEntry {
            term: "search",
            targets: &["contains", "find", "index_of", "regex_match"],
        },
        // Array/list synonyms
        SynonymEntry {
            term: "filter",
            targets: &["select", "where", "find_all", "keep"],
        },
        SynonymEntry {
            term: "select",
            targets: &["filter", "map", "pluck"],
        },
        SynonymEntry {
            term: "transform",
            targets: &["map", "transform_values", "map_values"],
        },
        SynonymEntry {
            term: "first",
            targets: &["head", "take", "front"],
        },
        SynonymEntry {
            term: "last",
            targets: &["tail", "end", "back"],
        },
        SynonymEntry {
            term: "remove",
            targets: &["reject", "delete", "drop", "exclude"],
        },
        SynonymEntry {
            term: "unique",
            targets: &["distinct", "uniq", "dedupe", "deduplicate"],
        },
        SynonymEntry {
            term: "dedupe",
            targets: &["unique", "distinct", "uniq"],
        },
        SynonymEntry {
            term: "shuffle",
            targets: &["random", "randomize", "permute"],
        },
        // Sorting synonyms
        SynonymEntry {
            term: "order",
            targets: &["sort", "sort_by", "order_by", "arrange"],
        },
        SynonymEntry {
            term: "arrange",
            targets: &["sort", "sort_by", "order_by"],
        },
        SynonymEntry {
            term: "rank",
            targets: &["sort", "sort_by", "order_by"],
        },
        // Math synonyms
        SynonymEntry {
            term: "average",
            targets: &["avg", "mean", "arithmetic_mean"],
        },
        SynonymEntry {
            term: "mean",
            targets: &["avg", "average"],
        },
        SynonymEntry {
            term: "total",
            targets: &["sum", "add", "accumulate"],
        },
        SynonymEntry {
            term: "add",
            targets: &["sum", "plus", "addition"],
        },
        SynonymEntry {
            term: "subtract",
            targets: &["minus", "difference"],
        },
        SynonymEntry {
            term: "multiply",
            targets: &["times", "product", "mul"],
        },
        SynonymEntry {
            term: "divide",
            targets: &["quotient", "div"],
        },
        SynonymEntry {
            term: "remainder",
            targets: &["mod", "modulo", "modulus"],
        },
        SynonymEntry {
            term: "power",
            targets: &["pow", "exponent", "exp"],
        },
        SynonymEntry {
            term: "absolute",
            targets: &["abs", "magnitude"],
        },
        SynonymEntry {
            term: "round",
            targets: &["round", "round_to", "nearest"],
        },
        SynonymEntry {
            term: "random",
            targets: &["rand", "random_int", "random_float"],
        },
        // Date/time synonyms
        SynonymEntry {
            term: "date",
            targets: &["now", "today", "parse_date", "format_date", "datetime"],
        },
        SynonymEntry {
            term: "time",
            targets: &["now", "time_now", "parse_time", "datetime"],
        },
        SynonymEntry {
            term: "timestamp",
            targets: &["now", "epoch", "unix_time", "to_epoch"],
        },
        SynonymEntry {
            term: "format",
            targets: &["format_date", "strftime", "date_format"],
        },
        SynonymEntry {
            term: "parse",
            targets: &["parse_date", "parse_time", "strptime", "from_string"],
        },
        // Type conversion synonyms
        SynonymEntry {
            term: "convert",
            targets: &["to_string", "to_number", "to_array", "type"],
        },
        SynonymEntry {
            term: "cast",
            targets: &["to_string", "to_number", "to_bool"],
        },
        SynonymEntry {
            term: "stringify",
            targets: &["to_string", "string", "str"],
        },
        SynonymEntry {
            term: "numberify",
            targets: &["to_number", "number", "int", "float"],
        },
        // Object/hash synonyms
        SynonymEntry {
            term: "object",
            targets: &["from_items", "to_object", "merge", "object_from_items"],
        },
        SynonymEntry {
            term: "dict",
            targets: &["from_items", "to_object", "object"],
        },
        SynonymEntry {
            term: "hash",
            targets: &["md5", "sha256", "sha1", "crc32"],
        },
        SynonymEntry {
            term: "encrypt",
            targets: &["md5", "sha256", "sha1", "hmac"],
        },
        SynonymEntry {
            term: "checksum",
            targets: &["md5", "sha256", "crc32"],
        },
        // Encoding synonyms
        SynonymEntry {
            term: "encode",
            targets: &["base64_encode", "url_encode", "hex_encode"],
        },
        SynonymEntry {
            term: "decode",
            targets: &["base64_decode", "url_decode", "hex_decode"],
        },
        SynonymEntry {
            term: "escape",
            targets: &["url_encode", "html_escape"],
        },
        SynonymEntry {
            term: "unescape",
            targets: &["url_decode", "html_unescape"],
        },
        // Validation synonyms
        SynonymEntry {
            term: "check",
            targets: &[
                "is_string",
                "is_number",
                "is_array",
                "is_object",
                "validate",
            ],
        },
        SynonymEntry {
            term: "validate",
            targets: &["is_email", "is_url", "is_uuid", "is_valid"],
        },
        SynonymEntry {
            term: "test",
            targets: &["regex_match", "contains", "starts_with", "ends_with"],
        },
        // Null/empty handling synonyms
        SynonymEntry {
            term: "default",
            targets: &["coalesce", "if_null", "or_else", "default_value"],
        },
        SynonymEntry {
            term: "empty",
            targets: &["is_empty", "blank", "null"],
        },
        SynonymEntry {
            term: "null",
            targets: &["is_null", "coalesce", "not_null"],
        },
        SynonymEntry {
            term: "fallback",
            targets: &["coalesce", "default", "or_else"],
        },
        // Comparison synonyms
        SynonymEntry {
            term: "equal",
            targets: &["eq", "equals", "same"],
        },
        SynonymEntry {
            term: "compare",
            targets: &["eq", "lt", "gt", "lte", "gte", "cmp"],
        },
        SynonymEntry {
            term: "between",
            targets: &["range", "in_range", "clamp"],
        },
        // Misc synonyms
        SynonymEntry {
            term: "copy",
            targets: &["clone", "dup", "duplicate"],
        },
        SynonymEntry {
            term: "debug",
            targets: &["debug", "inspect", "dump", "print"],
        },
        SynonymEntry {
            term: "reverse",
            targets: &["reverse", "flip", "invert"],
        },
        SynonymEntry {
            term: "repeat",
            targets: &["repeat", "replicate", "times"],
        },
        SynonymEntry {
            term: "uuid",
            targets: &["uuid", "uuid4", "guid", "generate_uuid"],
        },
        SynonymEntry {
            term: "id",
            targets: &["uuid", "nanoid", "ulid", "unique_id"],
        },
    ];
    SYNONYMS
}

/// Look up synonyms for a search term
///
/// Returns the list of related function names/keywords if the term has synonyms,
/// or None if not found.
///
/// # Example
///
/// ```
/// use jmespath_extensions::registry::lookup_synonyms;
///
/// let targets = lookup_synonyms("aggregate");
/// assert!(targets.is_some());
/// assert!(targets.unwrap().contains(&"group_by"));
/// ```
pub fn lookup_synonyms(term: &str) -> Option<&'static [&'static str]> {
    let term_lower = term.to_lowercase();
    get_synonyms()
        .iter()
        .find(|s| s.term == term_lower)
        .map(|s| s.targets)
}

/// Expand a search query using synonyms
///
/// Takes a search query (possibly multi-word) and returns all terms that should
/// be searched, including the original terms and any synonym expansions.
///
/// # Example
///
/// ```
/// use jmespath_extensions::registry::expand_search_terms;
///
/// let terms = expand_search_terms("group aggregate");
/// assert!(terms.contains(&"group_by".to_string()));
/// assert!(terms.contains(&"sum".to_string()));
/// ```
pub fn expand_search_terms(query: &str) -> Vec<String> {
    let mut expanded = Vec::new();

    for word in query.split_whitespace() {
        let word_lower = word.to_lowercase();
        expanded.push(word_lower.clone());

        if let Some(targets) = lookup_synonyms(&word_lower) {
            for target in targets {
                let target_str = (*target).to_string();
                if !expanded.contains(&target_str) {
                    expanded.push(target_str);
                }
            }
        }
    }

    expanded
}
