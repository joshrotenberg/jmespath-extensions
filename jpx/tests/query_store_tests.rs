//! Integration tests for MCP query store tools

#[cfg(feature = "mcp")]
mod query_store_integration {
    use jpx::mcp::query_store::{QueryStore, StoredQuery, query_store};

    /// Helper to reset the query store between tests
    fn with_fresh_store<F, R>(f: F) -> R
    where
        F: FnOnce(&mut QueryStore) -> R,
    {
        let store = query_store();
        let mut guard = store.write().unwrap();
        guard.clear();
        f(&mut guard)
    }

    #[test]
    fn test_define_and_retrieve_query() {
        with_fresh_store(|store| {
            let query = StoredQuery {
                name: "count_items".to_string(),
                expression: "length(@)".to_string(),
                description: Some("Count array items".to_string()),
            };

            assert!(store.define(query).is_none());

            let retrieved = store.get("count_items").unwrap();
            assert_eq!(retrieved.name, "count_items");
            assert_eq!(retrieved.expression, "length(@)");
            assert_eq!(retrieved.description, Some("Count array items".to_string()));
        });
    }

    #[test]
    fn test_define_overwrites_existing() {
        with_fresh_store(|store| {
            let query1 = StoredQuery {
                name: "my_query".to_string(),
                expression: "length(@)".to_string(),
                description: None,
            };

            let query2 = StoredQuery {
                name: "my_query".to_string(),
                expression: "keys(@)".to_string(),
                description: Some("Updated".to_string()),
            };

            store.define(query1);
            let old = store.define(query2).unwrap();

            assert_eq!(old.expression, "length(@)");

            let current = store.get("my_query").unwrap();
            assert_eq!(current.expression, "keys(@)");
            assert_eq!(current.description, Some("Updated".to_string()));
        });
    }

    #[test]
    fn test_list_queries_sorted() {
        with_fresh_store(|store| {
            store.define(StoredQuery {
                name: "zebra".to_string(),
                expression: "`1`".to_string(),
                description: None,
            });
            store.define(StoredQuery {
                name: "alpha".to_string(),
                expression: "`2`".to_string(),
                description: None,
            });
            store.define(StoredQuery {
                name: "middle".to_string(),
                expression: "`3`".to_string(),
                description: None,
            });

            let list = store.list();
            assert_eq!(list.len(), 3);
            assert_eq!(list[0].name, "alpha");
            assert_eq!(list[1].name, "middle");
            assert_eq!(list[2].name, "zebra");
        });
    }

    #[test]
    fn test_delete_query() {
        with_fresh_store(|store| {
            store.define(StoredQuery {
                name: "to_delete".to_string(),
                expression: "length(@)".to_string(),
                description: None,
            });

            assert_eq!(store.len(), 1);

            let deleted = store.delete("to_delete").unwrap();
            assert_eq!(deleted.name, "to_delete");
            assert_eq!(store.len(), 0);

            // Deleting non-existent returns None
            assert!(store.delete("nonexistent").is_none());
        });
    }

    #[test]
    fn test_get_nonexistent_query() {
        with_fresh_store(|store| {
            assert!(store.get("does_not_exist").is_none());
        });
    }

    #[test]
    fn test_query_store_workflow() {
        // Simulates an agent's workflow:
        // 1. Define a query
        // 2. Run it (via the expression)
        // 3. Refine it
        // 4. Delete old version
        with_fresh_store(|store| {
            // Initial query
            store.define(StoredQuery {
                name: "extract_names".to_string(),
                expression: "[*].name".to_string(),
                description: Some("Extract names from array".to_string()),
            });

            let q = store.get("extract_names").unwrap();
            assert_eq!(q.expression, "[*].name");

            // Refine the query (overwrite)
            store.define(StoredQuery {
                name: "extract_names".to_string(),
                expression: "[?active].name".to_string(),
                description: Some("Extract names from active items".to_string()),
            });

            let q = store.get("extract_names").unwrap();
            assert_eq!(q.expression, "[?active].name");
            assert_eq!(
                q.description,
                Some("Extract names from active items".to_string())
            );

            // Clean up
            store.delete("extract_names");
            assert!(store.is_empty());
        });
    }

    #[test]
    fn test_multiple_queries() {
        with_fresh_store(|store| {
            // Define several queries like an agent building a library
            store.define(StoredQuery {
                name: "count".to_string(),
                expression: "length(@)".to_string(),
                description: Some("Count items".to_string()),
            });

            store.define(StoredQuery {
                name: "keys".to_string(),
                expression: "keys(@)".to_string(),
                description: Some("Get object keys".to_string()),
            });

            store.define(StoredQuery {
                name: "first".to_string(),
                expression: "@[0]".to_string(),
                description: Some("Get first item".to_string()),
            });

            store.define(StoredQuery {
                name: "last".to_string(),
                expression: "@[-1]".to_string(),
                description: Some("Get last item".to_string()),
            });

            assert_eq!(store.len(), 4);

            let list = store.list();
            let names: Vec<&str> = list.iter().map(|q| q.name.as_str()).collect();
            assert_eq!(names, vec!["count", "first", "keys", "last"]);
        });
    }
}
