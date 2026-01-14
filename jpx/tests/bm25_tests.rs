//! BM25 search indexing tests

#[cfg(feature = "mcp")]
mod bm25_tests {
    use jpx::mcp::bm25::{Bm25Index, IndexOptions};
    use serde_json::json;

    #[test]
    fn test_build_index_simple() {
        let docs = vec![
            json!("hello world"),
            json!("hello there"),
            json!("goodbye world"),
        ];

        let index = Bm25Index::build(&docs, IndexOptions::default());

        assert_eq!(index.doc_count, 3);
        assert!(index.terms.contains_key("hello"));
        assert!(index.terms.contains_key("world"));
        assert_eq!(index.terms.get("hello").unwrap().df, 2);
        assert_eq!(index.terms.get("world").unwrap().df, 2);
    }

    #[test]
    fn test_build_index_with_fields() {
        let docs = vec![
            json!({"name": "create_cluster", "description": "Create a new cluster"}),
            json!({"name": "delete_cluster", "description": "Delete an existing cluster"}),
            json!({"name": "list_backups", "description": "List all backups"}),
        ];

        let options = IndexOptions {
            fields: vec!["name".to_string(), "description".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);

        assert_eq!(index.doc_count, 3);
        assert!(index.docs.contains_key("create_cluster"));
        assert!(index.docs.contains_key("delete_cluster"));
        assert!(index.terms.contains_key("cluster"));
        assert_eq!(index.terms.get("cluster").unwrap().df, 2);
    }

    #[test]
    fn test_search_basic() {
        let docs = vec![
            json!({"name": "create_cluster", "description": "Create a new Redis cluster"}),
            json!({"name": "delete_cluster", "description": "Delete an existing cluster"}),
            json!({"name": "create_backup", "description": "Create a backup of data"}),
        ];

        let options = IndexOptions {
            fields: vec!["name".to_string(), "description".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);
        let results = index.search("cluster", 10);

        assert_eq!(results.len(), 2);
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"create_cluster"));
        assert!(ids.contains(&"delete_cluster"));
    }

    #[test]
    fn test_search_ranking() {
        let docs = vec![
            json!({"name": "cluster_manager", "description": "Manage cluster operations"}),
            json!({"name": "backup_tool", "description": "Backup tool for cluster data"}),
            json!({"name": "monitor", "description": "Monitor system health"}),
        ];

        let options = IndexOptions {
            fields: vec!["name".to_string(), "description".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);
        let results = index.search("cluster", 10);

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "cluster_manager");
    }

    #[test]
    fn test_search_multi_term() {
        let docs = vec![
            json!({"name": "create_backup", "description": "Create a backup in a region"}),
            json!({"name": "restore_backup", "description": "Restore from backup"}),
            json!({"name": "list_regions", "description": "List available regions"}),
        ];

        let options = IndexOptions {
            fields: vec!["name".to_string(), "description".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);
        let results = index.search("backup region", 10);

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "create_backup");
    }

    #[test]
    fn test_explain() {
        let docs = vec![json!({"name": "test", "description": "test document with terms"})];

        let options = IndexOptions {
            fields: vec!["name".to_string(), "description".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);
        let explanation = index.explain("test", "test").unwrap();

        assert_eq!(explanation.id, "test");
        assert!(explanation.total_score > 0.0);
        assert!(!explanation.term_scores.is_empty());
    }

    #[test]
    fn test_similar() {
        let docs = vec![
            json!({"name": "create_cluster", "description": "Create a new cluster"}),
            json!({"name": "delete_cluster", "description": "Delete a cluster"}),
            json!({"name": "list_backups", "description": "List all backups"}),
        ];

        let options = IndexOptions {
            fields: vec!["name".to_string(), "description".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);
        let similar = index.similar("create_cluster", 10);

        assert!(!similar.is_empty());
        // delete_cluster shares "cluster" term, should be most similar
        assert_eq!(similar[0].id, "delete_cluster");
    }

    #[test]
    fn test_stopwords() {
        let docs = vec![json!("the quick brown fox"), json!("the lazy dog")];

        let options = IndexOptions {
            stopwords: vec!["the".to_string()],
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);

        assert!(!index.terms.contains_key("the"));
        assert!(index.terms.contains_key("quick"));
    }

    #[test]
    fn test_case_insensitive() {
        let docs = vec![json!("Hello World"), json!("HELLO THERE")];

        let index = Bm25Index::build(&docs, IndexOptions::default());
        let results = index.search("hello", 10);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_json_serialization() {
        let docs = vec![json!({"name": "test", "description": "test doc"})];

        let options = IndexOptions {
            fields: vec!["name".to_string()],
            id_field: Some("name".to_string()),
            ..Default::default()
        };

        let index = Bm25Index::build(&docs, options);

        let json_str = serde_json::to_string(&index).unwrap();
        assert!(json_str.contains("jpx:bm25_index"));

        let restored: Bm25Index = serde_json::from_str(&json_str).unwrap();
        assert_eq!(restored.doc_count, 1);
    }

    #[test]
    fn test_terms_list() {
        let docs = vec![
            json!("hello hello world"),
            json!("hello there"),
            json!("goodbye world"),
        ];

        let index = Bm25Index::build(&docs, IndexOptions::default());
        let terms = index.terms();

        assert!(!terms.is_empty());
        assert!(terms[0].1 >= terms.last().unwrap().1);
    }
}
