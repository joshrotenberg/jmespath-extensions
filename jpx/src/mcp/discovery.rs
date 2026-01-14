//! MCP Discovery Protocol implementation.
//!
//! This module implements a meta-protocol for capability registration and search
//! across MCP servers. It uses BM25 search indexing for efficient tool discovery.
//!
//! # Discovery Spec
//!
//! MCP servers can register their tools with jpx using a structured discovery spec:
//!
//! ```json
//! {
//!   "server": {"name": "my-server", "version": "1.0.0"},
//!   "tools": [
//!     {"name": "my_tool", "description": "Does something useful", "tags": ["read"]}
//!   ]
//! }
//! ```

use super::bm25::{Bm25Index, IndexOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Discovery spec schema version
pub const SCHEMA_VERSION: &str = "1.0";

/// Common English stop words to filter from search indexing.
/// These words are too common to be useful for search relevance.
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is", "it",
    "its", "of", "on", "or", "that", "the", "to", "was", "were", "will", "with", "this", "but",
    "they", "have", "had", "what", "when", "where", "who", "which", "why", "how", "all", "each",
    "every", "both", "few", "more", "most", "other", "some", "such", "no", "nor", "not", "only",
    "own", "same", "so", "than", "too", "very", "just", "can", "could", "should", "would", "may",
    "might", "must", "shall", "about", "above", "after", "again", "against", "below", "between",
    "into", "through", "during", "before", "under", "over",
];

/// Discovery spec - the schema MCP servers use to register their tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySpec {
    /// JSON Schema reference (optional)
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Server metadata
    pub server: ServerInfo,

    /// List of tools provided by this server
    pub tools: Vec<ToolSpec>,

    /// Category definitions (optional)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub categories: HashMap<String, CategoryInfo>,
}

/// Server metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name (required)
    pub name: String,

    /// Server version (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Server description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool name (required)
    pub name: String,

    /// Alternative names/aliases
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// Primary category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Subcategory within the primary category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<String>,

    /// Tags for filtering and search
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Short summary (for search results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Full description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter definitions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<ParamSpec>,

    /// Return type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnSpec>,

    /// Usage examples
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<ExampleSpec>,

    /// Related tools (author-declared relationships)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,

    /// Version when tool was added
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,

    /// Stability level (stable, beta, deprecated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<String>,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSpec {
    /// Parameter name
    pub name: String,

    /// Parameter type (string, number, boolean, object, array)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub param_type: Option<String>,

    /// Whether parameter is required
    #[serde(default)]
    pub required: bool,

    /// Parameter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Allowed values (for enums)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
}

/// Return type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnSpec {
    /// Return type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,

    /// Description of return value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Example specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleSpec {
    /// Example description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Example arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,

    /// Expected result (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    /// Category description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Subcategories
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcategories: Vec<String>,
}

/// Discovery registry - holds registered specs and search index
#[derive(Debug)]
pub struct DiscoveryRegistry {
    /// Registered servers: name -> spec
    servers: HashMap<String, DiscoverySpec>,

    /// All tools flattened for indexing: tool_id -> (server_name, tool_spec)
    tools: HashMap<String, (String, ToolSpec)>,

    /// BM25 search index (rebuilt on registration changes)
    index: Option<Bm25Index>,
}

impl Default for DiscoveryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            tools: HashMap::new(),
            index: None,
        }
    }

    /// Register a discovery spec
    pub fn register(&mut self, spec: DiscoverySpec, replace: bool) -> RegistrationResult {
        let server_name = spec.server.name.clone();

        // Check if server already registered
        if self.servers.contains_key(&server_name) && !replace {
            return RegistrationResult {
                ok: false,
                tools_indexed: 0,
                warnings: vec![format!(
                    "Server '{}' already registered. Use replace=true to update.",
                    server_name
                )],
            };
        }

        // Remove old tools from this server if replacing
        if replace {
            self.tools.retain(|_, (srv, _)| srv != &server_name);
        }

        // Add new tools
        let mut warnings = Vec::new();
        let mut tools_added = 0;

        for tool in &spec.tools {
            let tool_id = format!("{}:{}", server_name, tool.name);

            if self.tools.contains_key(&tool_id) && !replace {
                warnings.push(format!("Tool '{}' already exists, skipping", tool_id));
                continue;
            }

            self.tools
                .insert(tool_id, (server_name.clone(), tool.clone()));
            tools_added += 1;
        }

        // Store the spec
        self.servers.insert(server_name, spec);

        // Rebuild the search index
        self.rebuild_index();

        RegistrationResult {
            ok: true,
            tools_indexed: tools_added,
            warnings,
        }
    }

    /// Unregister a server
    pub fn unregister(&mut self, server_name: &str) -> bool {
        if self.servers.remove(server_name).is_some() {
            self.tools.retain(|_, (srv, _)| srv != server_name);
            self.rebuild_index();
            true
        } else {
            false
        }
    }

    /// Rebuild the BM25 search index from all registered tools
    fn rebuild_index(&mut self) {
        if self.tools.is_empty() {
            self.index = None;
            return;
        }

        // Convert tools to indexable documents
        let docs: Vec<Value> = self
            .tools
            .iter()
            .map(|(id, (server, tool))| {
                serde_json::json!({
                    "id": id,
                    "server": server,
                    "name": tool.name,
                    "aliases": tool.aliases.join(" "),
                    "category": tool.category.as_deref().unwrap_or(""),
                    "tags": tool.tags.join(" "),
                    "summary": tool.summary.as_deref().unwrap_or(""),
                    "description": tool.description.as_deref().unwrap_or(""),
                    "params": tool.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(" "),
                })
            })
            .collect();

        let options = IndexOptions {
            fields: vec![
                "name".to_string(),
                "aliases".to_string(),
                "category".to_string(),
                "tags".to_string(),
                "summary".to_string(),
                "description".to_string(),
                "params".to_string(),
            ],
            id_field: Some("id".to_string()),
            stopwords: STOP_WORDS.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };

        self.index = Some(Bm25Index::build(&docs, options));
    }

    /// Query tools across all registered servers
    pub fn query(&self, query: &str, top_k: usize) -> Vec<ToolQueryResult> {
        let Some(index) = &self.index else {
            return Vec::new();
        };

        let results = index.search(query, top_k);

        results
            .into_iter()
            .filter_map(|r| {
                let (server, tool) = self.tools.get(&r.id)?;
                Some(ToolQueryResult {
                    id: r.id,
                    server: server.clone(),
                    tool: tool.clone(),
                    score: r.score,
                    matches: r.matches,
                })
            })
            .collect()
    }

    /// Find tools similar to a given tool
    pub fn similar(&self, tool_id: &str, top_k: usize) -> Vec<ToolQueryResult> {
        let Some(index) = &self.index else {
            return Vec::new();
        };

        let results = index.similar(tool_id, top_k);

        results
            .into_iter()
            .filter_map(|r| {
                let (server, tool) = self.tools.get(&r.id)?;
                Some(ToolQueryResult {
                    id: r.id,
                    server: server.clone(),
                    tool: tool.clone(),
                    score: r.score,
                    matches: r.matches,
                })
            })
            .collect()
    }

    /// List all registered servers
    pub fn list_servers(&self) -> Vec<ServerSummary> {
        self.servers
            .iter()
            .map(|(name, spec)| ServerSummary {
                name: name.clone(),
                version: spec.server.version.clone(),
                description: spec.server.description.clone(),
                tool_count: spec.tools.len(),
            })
            .collect()
    }

    /// List all categories across all servers
    pub fn list_categories(&self) -> HashMap<String, CategorySummary> {
        let mut categories: HashMap<String, CategorySummary> = HashMap::new();

        for (server, tool) in self.tools.values() {
            if let Some(cat) = &tool.category {
                let entry = categories.entry(cat.clone()).or_insert(CategorySummary {
                    name: cat.clone(),
                    tool_count: 0,
                    servers: Vec::new(),
                    subcategories: Vec::new(),
                });
                entry.tool_count += 1;
                if !entry.servers.contains(server) {
                    entry.servers.push(server.clone());
                }
                if let Some(subcat) = tool
                    .subcategory
                    .as_ref()
                    .filter(|s| !entry.subcategories.contains(s))
                {
                    entry.subcategories.push(subcat.clone());
                }
            }
        }

        categories
    }

    /// Get index statistics
    pub fn index_stats(&self) -> Option<IndexStats> {
        let index = self.index.as_ref()?;

        Some(IndexStats {
            doc_count: index.doc_count,
            term_count: index.terms.len(),
            avg_doc_length: index.avg_doc_length,
            server_count: self.servers.len(),
            top_terms: index.terms().into_iter().take(20).collect(),
        })
    }

    /// Get the discovery schema as JSON
    pub fn get_schema() -> Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$id": "https://jpx.dev/schemas/mcp-discovery/v1.json",
            "title": "MCP Discovery Spec",
            "description": "Schema for registering MCP server capabilities with jpx",
            "type": "object",
            "required": ["server", "tools"],
            "properties": {
                "$schema": {
                    "type": "string",
                    "description": "JSON Schema reference"
                },
                "server": {
                    "type": "object",
                    "required": ["name"],
                    "properties": {
                        "name": {"type": "string", "description": "Server name"},
                        "version": {"type": "string", "description": "Server version"},
                        "description": {"type": "string", "description": "Server description"}
                    }
                },
                "tools": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": {"type": "string", "description": "Tool name"},
                            "aliases": {"type": "array", "items": {"type": "string"}},
                            "category": {"type": "string"},
                            "subcategory": {"type": "string"},
                            "tags": {"type": "array", "items": {"type": "string"}},
                            "summary": {"type": "string", "description": "Short summary"},
                            "description": {"type": "string", "description": "Full description"},
                            "params": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "required": ["name"],
                                    "properties": {
                                        "name": {"type": "string"},
                                        "type": {"type": "string"},
                                        "required": {"type": "boolean"},
                                        "description": {"type": "string"},
                                        "enum": {"type": "array", "items": {"type": "string"}},
                                        "default": {}
                                    }
                                }
                            },
                            "returns": {
                                "type": "object",
                                "properties": {
                                    "type": {"type": "string"},
                                    "description": {"type": "string"}
                                }
                            },
                            "examples": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "description": {"type": "string"},
                                        "args": {},
                                        "result": {}
                                    }
                                }
                            },
                            "related": {"type": "array", "items": {"type": "string"}},
                            "since": {"type": "string"},
                            "stability": {"type": "string", "enum": ["stable", "beta", "deprecated"]}
                        }
                    }
                },
                "categories": {
                    "type": "object",
                    "additionalProperties": {
                        "type": "object",
                        "properties": {
                            "description": {"type": "string"},
                            "subcategories": {"type": "array", "items": {"type": "string"}}
                        }
                    }
                }
            }
        })
    }
}

/// Result of registering a discovery spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub ok: bool,
    pub tools_indexed: usize,
    pub warnings: Vec<String>,
}

/// Tool query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolQueryResult {
    pub id: String,
    pub server: String,
    pub tool: ToolSpec,
    pub score: f64,
    pub matches: HashMap<String, Vec<String>>,
}

/// Server summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSummary {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub tool_count: usize,
}

/// Category summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub name: String,
    pub tool_count: usize,
    pub servers: Vec<String>,
    pub subcategories: Vec<String>,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub doc_count: usize,
    pub term_count: usize,
    pub avg_doc_length: f64,
    pub server_count: usize,
    pub top_terms: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> DiscoverySpec {
        serde_json::from_value(serde_json::json!({
            "server": {
                "name": "redisctl",
                "version": "0.5.0",
                "description": "Redis Enterprise management"
            },
            "tools": [
                {
                    "name": "create_cluster",
                    "category": "clusters",
                    "tags": ["write", "provisioning"],
                    "summary": "Create a new Redis cluster",
                    "description": "Creates a new Redis Enterprise cluster with specified configuration"
                },
                {
                    "name": "delete_cluster",
                    "category": "clusters",
                    "tags": ["write", "destructive"],
                    "summary": "Delete a cluster",
                    "description": "Permanently deletes a Redis cluster"
                },
                {
                    "name": "list_backups",
                    "category": "backups",
                    "tags": ["read"],
                    "summary": "List all backups",
                    "description": "Lists all available backups for a cluster"
                }
            ]
        })).unwrap()
    }

    #[test]
    fn test_register_spec() {
        let mut registry = DiscoveryRegistry::new();
        let spec = sample_spec();

        let result = registry.register(spec, false);

        assert!(result.ok);
        assert_eq!(result.tools_indexed, 3);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_query_tools() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let results = registry.query("cluster", 10);

        // All tools mention cluster in their descriptions, but cluster tools rank higher
        assert!(!results.is_empty());
        // Top results should be the cluster tools (they have "cluster" in name)
        let top_names: Vec<_> = results
            .iter()
            .take(2)
            .map(|r| r.tool.name.as_str())
            .collect();
        assert!(top_names.contains(&"create_cluster"));
        assert!(top_names.contains(&"delete_cluster"));
    }

    #[test]
    fn test_query_by_tag() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let results = registry.query("read", 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool.name, "list_backups");
    }

    #[test]
    fn test_list_servers() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let servers = registry.list_servers();

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "redisctl");
        assert_eq!(servers[0].tool_count, 3);
    }

    #[test]
    fn test_list_categories() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let categories = registry.list_categories();

        assert_eq!(categories.len(), 2);
        assert!(categories.contains_key("clusters"));
        assert!(categories.contains_key("backups"));
        assert_eq!(categories.get("clusters").unwrap().tool_count, 2);
    }

    #[test]
    fn test_unregister() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        assert!(registry.unregister("redisctl"));
        assert!(registry.list_servers().is_empty());
        assert!(registry.query("cluster", 10).is_empty());
    }

    #[test]
    fn test_replace_registration() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        // Try to register again without replace - should fail
        let result = registry.register(sample_spec(), false);
        assert!(!result.ok);

        // With replace - should succeed
        let result = registry.register(sample_spec(), true);
        assert!(result.ok);
    }

    #[test]
    fn test_similar_tools() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let similar = registry.similar("redisctl:create_cluster", 10);

        // delete_cluster should be similar (shares "cluster" terms)
        assert!(!similar.is_empty());
        assert_eq!(similar[0].tool.name, "delete_cluster");
    }

    #[test]
    fn test_minimal_spec() {
        let minimal: DiscoverySpec = serde_json::from_value(serde_json::json!({
            "server": {"name": "minimal"},
            "tools": [{"name": "foo"}]
        }))
        .unwrap();

        let mut registry = DiscoveryRegistry::new();
        let result = registry.register(minimal, false);

        assert!(result.ok);
        assert_eq!(result.tools_indexed, 1);
    }

    #[test]
    fn test_get_schema() {
        let schema = DiscoveryRegistry::get_schema();

        assert!(schema.get("$schema").is_some());
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_index_stats() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(sample_spec(), false);

        let stats = registry.index_stats().unwrap();

        assert_eq!(stats.doc_count, 3);
        assert_eq!(stats.server_count, 1);
        assert!(stats.term_count > 0);
    }
}
