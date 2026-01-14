//! Integration tests for MCP discovery protocol

#[cfg(feature = "mcp")]
mod discovery_integration {
    use jpx::mcp::discovery::{DiscoveryRegistry, DiscoverySpec};
    use serde_json::json;

    fn redisctl_spec() -> DiscoverySpec {
        serde_json::from_value(json!({
            "server": {
                "name": "redisctl",
                "version": "0.5.0",
                "description": "Redis Enterprise and Cloud management"
            },
            "tools": [
                {
                    "name": "create_cluster",
                    "category": "clusters",
                    "subcategory": "lifecycle",
                    "tags": ["write", "provisioning", "enterprise"],
                    "summary": "Create a new Redis cluster",
                    "description": "Creates a new Redis Enterprise cluster with specified configuration",
                    "params": [
                        {"name": "name", "type": "string", "required": true},
                        {"name": "region", "type": "string", "enum": ["us-east-1", "eu-west-1"]}
                    ],
                    "related": ["delete_cluster", "list_clusters"]
                },
                {
                    "name": "delete_cluster",
                    "category": "clusters",
                    "subcategory": "lifecycle",
                    "tags": ["write", "destructive"],
                    "summary": "Delete a Redis cluster",
                    "description": "Permanently deletes a Redis cluster and all its data"
                },
                {
                    "name": "list_clusters",
                    "category": "clusters",
                    "tags": ["read"],
                    "summary": "List all clusters",
                    "description": "Lists all Redis clusters in the account"
                },
                {
                    "name": "create_backup",
                    "category": "backups",
                    "tags": ["write"],
                    "summary": "Create a backup",
                    "description": "Creates a point-in-time backup of a cluster"
                },
                {
                    "name": "restore_backup",
                    "category": "backups",
                    "tags": ["write"],
                    "summary": "Restore from backup",
                    "description": "Restores a cluster from a backup"
                }
            ],
            "categories": {
                "clusters": {
                    "description": "Cluster lifecycle and configuration",
                    "subcategories": ["lifecycle", "config", "scaling"]
                },
                "backups": {
                    "description": "Backup and restore operations"
                }
            }
        }))
        .unwrap()
    }

    fn postgres_spec() -> DiscoverySpec {
        serde_json::from_value(json!({
            "server": {
                "name": "pgctl",
                "version": "1.0.0",
                "description": "PostgreSQL management"
            },
            "tools": [
                {
                    "name": "create_database",
                    "category": "databases",
                    "tags": ["write"],
                    "summary": "Create a database",
                    "description": "Creates a new PostgreSQL database"
                },
                {
                    "name": "create_backup",
                    "category": "backups",
                    "tags": ["write"],
                    "summary": "Create a backup",
                    "description": "Creates a database backup using pg_dump"
                },
                {
                    "name": "list_tables",
                    "category": "schema",
                    "tags": ["read"],
                    "summary": "List tables",
                    "description": "Lists all tables in a database"
                }
            ]
        }))
        .unwrap()
    }

    #[test]
    fn test_multi_server_registration() {
        let mut registry = DiscoveryRegistry::new();

        // Register both servers
        let r1 = registry.register(redisctl_spec(), false);
        let r2 = registry.register(postgres_spec(), false);

        assert!(r1.ok);
        assert_eq!(r1.tools_indexed, 5);

        assert!(r2.ok);
        assert_eq!(r2.tools_indexed, 3);

        // Should have 2 servers
        let servers = registry.list_servers();
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn test_cross_server_search() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);
        registry.register(postgres_spec(), false);

        // Search for "backup" - should find tools from both servers
        let results = registry.query("backup", 10);

        assert!(results.len() >= 2);

        let servers: Vec<_> = results.iter().map(|r| r.server.as_str()).collect();
        assert!(servers.contains(&"redisctl"));
        assert!(servers.contains(&"pgctl"));
    }

    #[test]
    fn test_search_by_tag() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);

        // Search for "destructive" tag
        let results = registry.query("destructive", 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool.name, "delete_cluster");
    }

    #[test]
    fn test_search_by_category() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);

        // Search for "backups" category
        let results = registry.query("backups", 10);

        assert!(!results.is_empty());
        // Should find backup-related tools
        let names: Vec<_> = results.iter().map(|r| r.tool.name.as_str()).collect();
        assert!(names.contains(&"create_backup") || names.contains(&"restore_backup"));
    }

    #[test]
    fn test_similar_tools_cross_server() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);
        registry.register(postgres_spec(), false);

        // Find tools similar to redisctl's create_backup
        let similar = registry.similar("redisctl:create_backup", 10);

        // pgctl's create_backup should be similar
        let names: Vec<_> = similar.iter().map(|r| r.tool.name.as_str()).collect();
        assert!(names.contains(&"create_backup")); // From pgctl
    }

    #[test]
    fn test_category_aggregation() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);
        registry.register(postgres_spec(), false);

        let categories = registry.list_categories();

        // "backups" category should have tools from both servers
        let backups = categories.get("backups").unwrap();
        assert_eq!(backups.servers.len(), 2);
        assert!(backups.servers.contains(&"redisctl".to_string()));
        assert!(backups.servers.contains(&"pgctl".to_string()));
    }

    #[test]
    fn test_unregister_removes_from_search() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);
        registry.register(postgres_spec(), false);

        // Search before unregister
        let results = registry.query("cluster", 10);
        let servers: Vec<_> = results.iter().map(|r| r.server.as_str()).collect();
        assert!(servers.contains(&"redisctl"));

        // Unregister redisctl
        assert!(registry.unregister("redisctl"));

        // Search after unregister
        let results = registry.query("cluster", 10);
        let servers: Vec<_> = results.iter().map(|r| r.server.as_str()).collect();
        assert!(!servers.contains(&"redisctl"));
    }

    #[test]
    fn test_index_stats_multi_server() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(redisctl_spec(), false);
        registry.register(postgres_spec(), false);

        let stats = registry.index_stats().unwrap();

        assert_eq!(stats.doc_count, 8); // 5 + 3 tools
        assert_eq!(stats.server_count, 2);
        assert!(stats.term_count > 0);
    }

    #[test]
    fn test_get_schema() {
        let schema = DiscoveryRegistry::get_schema();

        // Verify schema structure
        assert_eq!(
            schema.get("$id").and_then(|v| v.as_str()),
            Some("https://jpx.dev/schemas/mcp-discovery/v1.json")
        );
        assert!(schema.get("properties").is_some());
        assert!(
            schema
                .get("required")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().any(|v| v == "server"))
                .unwrap_or(false)
        );
    }
}
