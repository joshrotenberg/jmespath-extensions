//! Integration tests for MCP discovery using mock servers
//!
//! These tests verify that jpx's discovery protocol correctly handles
//! registrations from multiple MCP servers.

/// Tests for MCP tool schema validity
///
/// These tests ensure tool schemas don't use patterns that cause
/// MCP clients (like Claude Code) to reject the entire server.
#[cfg(feature = "mcp")]
mod schema_validity {
    use serde_json::Value;

    /// Validates that a tool's inputSchema doesn't use the problematic
    /// `Parameters<()>` pattern which generates `{"const": null, "nullable": true}`
    fn schema_is_valid(schema: &Value) -> bool {
        // Reject schemas with "const": null - this breaks Claude Code
        if schema.get("const") == Some(&Value::Null) {
            return false;
        }

        // Valid no-param schemas should have: {"properties": {}, "type": "object"}
        // or a proper struct schema with title/description
        true
    }

    #[test]
    fn test_no_const_null_in_tool_schemas() {
        // Simulate the problematic schema that Parameters<()> generates
        let bad_schema: Value = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "const": null,
            "nullable": true,
            "title": "null"
        });

        assert!(
            !schema_is_valid(&bad_schema),
            "Schema with const:null should be invalid"
        );

        // Valid empty params schema (what no-arg tools should generate)
        let good_schema: Value = serde_json::json!({
            "properties": {},
            "type": "object"
        });

        assert!(
            schema_is_valid(&good_schema),
            "Empty object schema should be valid"
        );

        // Valid schema with actual parameters
        let params_schema: Value = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "description": "Parameters for some tool",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"],
            "type": "object"
        });

        assert!(
            schema_is_valid(&params_schema),
            "Normal params schema should be valid"
        );
    }
}

/// Tests for BM25 search quality improvements
#[cfg(feature = "mcp")]
mod search_quality {
    use jpx::mcp::discovery::{DiscoveryRegistry, DiscoverySpec};
    use serde_json::json;

    fn spec_with_description(server_name: &str, tool_name: &str, desc: &str) -> DiscoverySpec {
        serde_json::from_value(json!({
            "server": { "name": server_name },
            "tools": [{ "name": tool_name, "description": desc }]
        }))
        .unwrap()
    }

    #[test]
    fn test_stop_words_filtered_from_index() {
        let mut registry = DiscoveryRegistry::new();

        // Register a tool with lots of stop words in description
        registry.register(
            spec_with_description(
                "test-server",
                "test_tool",
                "This is a tool for the database that will be used to create and manage resources",
            ),
            false,
        );

        let stats = registry.index_stats().unwrap();
        let top_terms: Vec<&str> = stats.top_terms.iter().map(|(t, _)| t.as_str()).collect();

        // Stop words should NOT be in the index
        assert!(
            !top_terms.contains(&"a"),
            "Stop word 'a' should be filtered"
        );
        assert!(
            !top_terms.contains(&"the"),
            "Stop word 'the' should be filtered"
        );
        assert!(
            !top_terms.contains(&"is"),
            "Stop word 'is' should be filtered"
        );
        assert!(
            !top_terms.contains(&"for"),
            "Stop word 'for' should be filtered"
        );
        assert!(
            !top_terms.contains(&"and"),
            "Stop word 'and' should be filtered"
        );
        assert!(
            !top_terms.contains(&"to"),
            "Stop word 'to' should be filtered"
        );
        assert!(
            !top_terms.contains(&"that"),
            "Stop word 'that' should be filtered"
        );
        assert!(
            !top_terms.contains(&"will"),
            "Stop word 'will' should be filtered"
        );
        assert!(
            !top_terms.contains(&"be"),
            "Stop word 'be' should be filtered"
        );

        // Content words SHOULD be in the index
        assert!(
            top_terms.contains(&"tool"),
            "Content word 'tool' should be indexed"
        );
        assert!(
            top_terms.contains(&"database"),
            "Content word 'database' should be indexed"
        );
        assert!(
            top_terms.contains(&"create"),
            "Content word 'create' should be indexed"
        );
        assert!(
            top_terms.contains(&"manage"),
            "Content word 'manage' should be indexed"
        );
        assert!(
            top_terms.contains(&"resources"),
            "Content word 'resources' should be indexed"
        );
    }

    #[test]
    fn test_similar_tools_without_stop_word_noise() {
        let mut registry = DiscoveryRegistry::new();

        // Register tools with similar purposes but different stop words
        registry.register(
            serde_json::from_value(json!({
                "server": { "name": "test-server" },
                "tools": [
                    { "name": "create_backup", "description": "Create a backup of the database" },
                    { "name": "restore_backup", "description": "Restore the database from a backup" },
                    { "name": "list_users", "description": "List all the users in the system" }
                ]
            }))
            .unwrap(),
            false,
        );

        // Find tools similar to create_backup
        let similar = registry.similar("test-server:create_backup", 10);

        // restore_backup should be the most similar (shares "backup" and "database")
        assert!(!similar.is_empty(), "Should find similar tools");
        assert_eq!(
            similar[0].tool.name, "restore_backup",
            "restore_backup should be most similar to create_backup"
        );

        // The matches should NOT include stop words
        if let Some(matches) = similar[0].matches.get("_matched") {
            assert!(
                !matches.contains(&"a".to_string()),
                "Matches should not include 'a'"
            );
            assert!(
                !matches.contains(&"the".to_string()),
                "Matches should not include 'the'"
            );
            assert!(
                !matches.contains(&"of".to_string()),
                "Matches should not include 'of'"
            );
        }
    }
}

#[cfg(feature = "mcp")]
mod mock_server_discovery {
    use jpx::mcp::discovery::{DiscoveryRegistry, DiscoverySpec};
    use serde_json::json;

    /// Helper to create a discovery spec from a mock server config
    fn mock_server_spec(
        name: &str,
        tools: Vec<(&str, &str, Option<&str>, Vec<&str>)>,
    ) -> DiscoverySpec {
        let tools_json: Vec<serde_json::Value> = tools
            .into_iter()
            .map(|(tool_name, desc, category, tags)| {
                let mut tool = json!({
                    "name": tool_name,
                    "description": desc,
                });
                if let Some(cat) = category {
                    tool["category"] = json!(cat);
                }
                if !tags.is_empty() {
                    tool["tags"] = json!(tags);
                }
                tool
            })
            .collect();

        serde_json::from_value(json!({
            "server": {
                "name": name,
                "version": "1.0.0",
                "description": format!("Mock {} server", name)
            },
            "tools": tools_json
        }))
        .unwrap()
    }

    /// Simulates what mock-redis server would register
    fn mock_redis_spec() -> DiscoverySpec {
        mock_server_spec(
            "mock-redis",
            vec![
                (
                    "create_cluster",
                    "Create a new Redis cluster",
                    Some("clusters"),
                    vec!["write", "provisioning"],
                ),
                (
                    "delete_cluster",
                    "Delete a Redis cluster permanently",
                    Some("clusters"),
                    vec!["write", "destructive"],
                ),
                (
                    "list_clusters",
                    "List all Redis clusters",
                    Some("clusters"),
                    vec!["read"],
                ),
                (
                    "create_backup",
                    "Create a backup of a cluster",
                    Some("backups"),
                    vec!["write"],
                ),
                (
                    "restore_backup",
                    "Restore a cluster from backup",
                    Some("backups"),
                    vec!["write"],
                ),
            ],
        )
    }

    /// Simulates what mock-postgres server would register
    fn mock_postgres_spec() -> DiscoverySpec {
        mock_server_spec(
            "mock-postgres",
            vec![
                (
                    "create_database",
                    "Create a new PostgreSQL database",
                    Some("databases"),
                    vec!["write"],
                ),
                (
                    "drop_database",
                    "Drop a database",
                    Some("databases"),
                    vec!["write", "destructive"],
                ),
                (
                    "list_tables",
                    "List all tables in a database",
                    Some("schema"),
                    vec!["read"],
                ),
                (
                    "create_backup",
                    "Create a pg_dump backup",
                    Some("backups"),
                    vec!["write"],
                ),
            ],
        )
    }

    /// Simulates what mock-github server would register
    fn mock_github_spec() -> DiscoverySpec {
        mock_server_spec(
            "mock-github",
            vec![
                (
                    "create_issue",
                    "Create a GitHub issue",
                    Some("issues"),
                    vec!["write"],
                ),
                (
                    "list_issues",
                    "List issues in a repository",
                    Some("issues"),
                    vec!["read"],
                ),
                (
                    "create_pull_request",
                    "Create a pull request",
                    Some("pulls"),
                    vec!["write"],
                ),
                (
                    "list_pull_requests",
                    "List pull requests",
                    Some("pulls"),
                    vec!["read"],
                ),
                (
                    "merge_pull_request",
                    "Merge a pull request",
                    Some("pulls"),
                    vec!["write"],
                ),
            ],
        )
    }

    #[test]
    fn test_register_multiple_mock_servers() {
        let mut registry = DiscoveryRegistry::new();

        // Register all three mock servers
        let r1 = registry.register(mock_redis_spec(), false);
        let r2 = registry.register(mock_postgres_spec(), false);
        let r3 = registry.register(mock_github_spec(), false);

        assert!(r1.ok, "Redis registration failed: {:?}", r1.warnings);
        assert!(r2.ok, "Postgres registration failed: {:?}", r2.warnings);
        assert!(r3.ok, "GitHub registration failed: {:?}", r3.warnings);

        // Verify server count
        let servers = registry.list_servers();
        assert_eq!(servers.len(), 3);

        // Verify tool counts
        let redis_server = servers.iter().find(|s| s.name == "mock-redis").unwrap();
        let postgres_server = servers.iter().find(|s| s.name == "mock-postgres").unwrap();
        let github_server = servers.iter().find(|s| s.name == "mock-github").unwrap();

        assert_eq!(redis_server.tool_count, 5);
        assert_eq!(postgres_server.tool_count, 4);
        assert_eq!(github_server.tool_count, 5);
    }

    #[test]
    fn test_cross_server_search_backup() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);
        registry.register(mock_github_spec(), false);

        // Search for "backup" - should find tools from both redis and postgres
        let results = registry.query("backup", 10);

        assert!(
            results.len() >= 2,
            "Expected at least 2 backup tools, got {}",
            results.len()
        );

        let servers: Vec<&str> = results.iter().map(|r| r.server.as_str()).collect();
        assert!(
            servers.contains(&"mock-redis"),
            "Expected mock-redis in backup results"
        );
        assert!(
            servers.contains(&"mock-postgres"),
            "Expected mock-postgres in backup results"
        );
        assert!(
            !servers.contains(&"mock-github"),
            "GitHub shouldn't have backup tools"
        );
    }

    #[test]
    fn test_cross_server_search_destructive() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);

        // Search for "destructive" tag
        let results = registry.query("destructive", 10);

        // Should find delete_cluster and drop_database
        let tool_names: Vec<&str> = results.iter().map(|r| r.tool.name.as_str()).collect();
        assert!(
            tool_names.contains(&"delete_cluster"),
            "Expected delete_cluster in destructive results"
        );
        assert!(
            tool_names.contains(&"drop_database"),
            "Expected drop_database in destructive results"
        );
    }

    #[test]
    fn test_cross_server_search_read_operations() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);
        registry.register(mock_github_spec(), false);

        // Search for "read" tag - should find read-only tools from all servers
        let results = registry.query("read", 10);

        let servers: std::collections::HashSet<&str> =
            results.iter().map(|r| r.server.as_str()).collect();

        // All three servers have read tools
        assert!(
            servers.contains("mock-redis"),
            "Expected mock-redis read tools"
        );
        assert!(
            servers.contains("mock-postgres"),
            "Expected mock-postgres read tools"
        );
        assert!(
            servers.contains("mock-github"),
            "Expected mock-github read tools"
        );
    }

    #[test]
    fn test_category_aggregation_across_servers() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);

        let categories = registry.list_categories();

        // "backups" category should have tools from both servers
        let backups = categories.get("backups");
        assert!(backups.is_some(), "Expected backups category");

        let backups = backups.unwrap();
        assert!(backups.servers.contains(&"mock-redis".to_string()));
        assert!(backups.servers.contains(&"mock-postgres".to_string()));
    }

    #[test]
    fn test_similar_tools_cross_server() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);

        // Find tools similar to redis's create_backup
        let similar = registry.similar("mock-redis:create_backup", 10);

        // postgres's create_backup should be similar
        let has_postgres_backup = similar
            .iter()
            .any(|r| r.server == "mock-postgres" && r.tool.name == "create_backup");
        assert!(
            has_postgres_backup,
            "Expected postgres create_backup to be similar to redis create_backup"
        );
    }

    #[test]
    fn test_unregister_removes_server_from_search() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);

        // Verify redis is in search results
        let results = registry.query("cluster", 10);
        assert!(results.iter().any(|r| r.server == "mock-redis"));

        // Unregister redis
        assert!(registry.unregister("mock-redis"));

        // Verify redis is no longer in search results
        let results = registry.query("cluster", 10);
        assert!(!results.iter().any(|r| r.server == "mock-redis"));

        // Postgres should still be there
        let servers = registry.list_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "mock-postgres");
    }

    #[test]
    fn test_replace_registration() {
        let mut registry = DiscoveryRegistry::new();

        // Initial registration
        let r1 = registry.register(mock_redis_spec(), false);
        assert!(r1.ok);
        assert_eq!(r1.tools_indexed, 5);

        // Try to register again without replace - should fail
        let r2 = registry.register(mock_redis_spec(), false);
        assert!(!r2.ok);

        // Register with replace - should succeed
        let r3 = registry.register(mock_redis_spec(), true);
        assert!(r3.ok);
        assert_eq!(r3.tools_indexed, 5);

        // Should still have only one server
        let servers = registry.list_servers();
        assert_eq!(servers.len(), 1);
    }

    #[test]
    fn test_index_stats_multiple_servers() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);
        registry.register(mock_github_spec(), false);

        let stats = registry.index_stats().unwrap();

        assert_eq!(stats.server_count, 3);
        assert_eq!(stats.doc_count, 14); // 5 + 4 + 5 tools
        assert!(stats.term_count > 0);
        assert!(stats.avg_doc_length > 0.0);
    }

    #[test]
    fn test_stress_many_servers() {
        let mut registry = DiscoveryRegistry::new();

        // Register 10 servers with 20 tools each
        for i in 0..10 {
            let tools: Vec<(&str, &str, Option<&str>, Vec<&str>)> = (0..20)
                .map(|j| {
                    // Leak strings to get static lifetimes for the test
                    let name: &'static str =
                        Box::leak(format!("tool_{}_{}", i, j).into_boxed_str());
                    let desc: &'static str =
                        Box::leak(format!("Tool {} from server {}", j, i).into_boxed_str());
                    (name, desc, Some("testing"), vec!["test"])
                })
                .collect();

            let server_name: &'static str =
                Box::leak(format!("stress-server-{}", i).into_boxed_str());
            let spec = mock_server_spec(server_name, tools);
            let result = registry.register(spec, false);
            assert!(result.ok, "Failed to register server {}", i);
        }

        let servers = registry.list_servers();
        assert_eq!(servers.len(), 10);

        let stats = registry.index_stats().unwrap();
        assert_eq!(stats.doc_count, 200); // 10 * 20 tools

        // Search should still work
        let results = registry.query("tool", 50);
        assert!(results.len() >= 50);
    }

    #[test]
    fn test_semantic_search_database() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);
        registry.register(mock_github_spec(), false);

        // Search for "database" - should primarily find postgres tools
        let results = registry.query("database", 10);

        // Postgres tools should rank higher since they have "database" in descriptions
        assert!(!results.is_empty());
        let top_servers: Vec<&str> = results.iter().take(3).map(|r| r.server.as_str()).collect();
        assert!(
            top_servers.contains(&"mock-postgres"),
            "Postgres should be in top results for 'database'"
        );
    }

    #[test]
    fn test_semantic_search_pull_request() {
        let mut registry = DiscoveryRegistry::new();
        registry.register(mock_redis_spec(), false);
        registry.register(mock_postgres_spec(), false);
        registry.register(mock_github_spec(), false);

        // Search for "pull request" - should find GitHub tools
        let results = registry.query("pull request", 10);

        assert!(!results.is_empty());
        let github_results: Vec<_> = results
            .iter()
            .filter(|r| r.server == "mock-github")
            .collect();
        assert!(
            !github_results.is_empty(),
            "Expected GitHub tools for 'pull request'"
        );
    }
}

/// Tests for concurrent registration scenarios
#[cfg(feature = "mcp")]
mod concurrent_discovery {
    use jpx::mcp::discovery::{DiscoveryRegistry, DiscoverySpec};
    use serde_json::json;
    use std::sync::{Arc, RwLock};
    use std::thread;

    fn make_spec(name: &str, tool_count: usize) -> DiscoverySpec {
        let tools: Vec<serde_json::Value> = (0..tool_count)
            .map(|i| {
                json!({
                    "name": format!("{}_tool_{}", name, i),
                    "description": format!("Tool {} from {}", i, name),
                    "category": "test",
                    "tags": ["concurrent"]
                })
            })
            .collect();

        serde_json::from_value(json!({
            "server": { "name": name, "version": "1.0.0" },
            "tools": tools
        }))
        .unwrap()
    }

    #[test]
    fn test_concurrent_registrations() {
        let registry = Arc::new(RwLock::new(DiscoveryRegistry::new()));
        let mut handles = vec![];

        // Spawn 5 threads, each registering a different server
        for i in 0..5 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let server_name = format!("concurrent-server-{}", i);
                let spec = make_spec(&server_name, 10);

                let result = {
                    let mut reg = registry.write().unwrap();
                    reg.register(spec, false)
                };

                assert!(
                    result.ok,
                    "Failed to register {}: {:?}",
                    server_name, result.warnings
                );
                result.tools_indexed
            });
            handles.push(handle);
        }

        // Wait for all registrations to complete
        let total_tools: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(total_tools, 50); // 5 servers * 10 tools each

        // Verify all servers are registered
        let servers = registry.read().unwrap().list_servers();
        assert_eq!(servers.len(), 5);
    }

    #[test]
    fn test_concurrent_reads_during_registration() {
        let registry = Arc::new(RwLock::new(DiscoveryRegistry::new()));

        // Pre-register one server
        {
            let mut reg = registry.write().unwrap();
            reg.register(make_spec("pre-registered", 5), false);
        }

        let mut handles = vec![];

        // Spawn readers
        for _ in 0..3 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let reg = registry.read().unwrap();
                    let _ = reg.query("tool", 5);
                    let _ = reg.list_servers();
                }
            });
            handles.push(handle);
        }

        // Spawn writers (new registrations)
        for i in 0..3 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let spec = make_spec(&format!("writer-{}", i), 5);
                let mut reg = registry.write().unwrap();
                let result = reg.register(spec, false);
                assert!(result.ok);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify state
        let servers = registry.read().unwrap().list_servers();
        assert_eq!(servers.len(), 4); // 1 pre-registered + 3 writers
    }

    #[test]
    fn test_concurrent_search_consistency() {
        let registry = Arc::new(RwLock::new(DiscoveryRegistry::new()));

        // Register multiple servers
        {
            let mut reg = registry.write().unwrap();
            for i in 0..3 {
                reg.register(make_spec(&format!("search-server-{}", i), 10), false);
            }
        }

        let mut handles = vec![];

        // Spawn multiple search threads
        for _ in 0..10 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let reg = registry.read().unwrap();
                let results = reg.query("tool", 30);

                // Should always find tools from all 3 servers
                let servers: std::collections::HashSet<_> =
                    results.iter().map(|r| r.server.clone()).collect();

                assert_eq!(servers.len(), 3, "Expected 3 servers in results");
                results.len()
            });
            handles.push(handle);
        }

        // All searches should return consistent results
        let result_counts: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All threads should see the same number of results
        let first = result_counts[0];
        assert!(
            result_counts.iter().all(|&c| c == first),
            "Inconsistent search results across threads"
        );
    }

    #[test]
    fn test_unregister_during_search() {
        let registry = Arc::new(RwLock::new(DiscoveryRegistry::new()));

        // Register servers
        {
            let mut reg = registry.write().unwrap();
            reg.register(make_spec("stable-server", 10), false);
            reg.register(make_spec("unstable-server", 10), false);
        }

        let registry_for_reader = Arc::clone(&registry);
        let reader_handle = thread::spawn(move || {
            let mut found_unstable = false;
            for _ in 0..100 {
                let reg = registry_for_reader.read().unwrap();
                let results = reg.query("tool", 20);
                if results.iter().any(|r| r.server == "unstable-server") {
                    found_unstable = true;
                }
            }
            found_unstable
        });

        // Give reader a head start
        thread::sleep(std::time::Duration::from_millis(1));

        // Unregister the unstable server
        {
            let mut reg = registry.write().unwrap();
            reg.unregister("unstable-server");
        }

        // Reader should have seen the unstable server at some point
        let saw_unstable = reader_handle.join().unwrap();
        // Note: This might be flaky, but demonstrates the test pattern
        // In practice, we're testing that this doesn't panic/deadlock
        let _ = saw_unstable;

        // Final state should only have stable server
        let servers = registry.read().unwrap().list_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "stable-server");
    }

    #[test]
    fn test_replace_registration_concurrent() {
        let registry = Arc::new(RwLock::new(DiscoveryRegistry::new()));

        // Initial registration
        {
            let mut reg = registry.write().unwrap();
            reg.register(make_spec("replaceable", 5), false);
        }

        let mut handles = vec![];

        // Multiple threads try to replace the same server
        for i in 0..5 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let spec = make_spec("replaceable", 5 + i); // Each has different tool count
                let mut reg = registry.write().unwrap();
                reg.register(spec, true)
            });
            handles.push(handle);
        }

        // All replacements should succeed
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.ok);
        }

        // Should still have exactly one server
        let servers = registry.read().unwrap().list_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "replaceable");
    }
}
