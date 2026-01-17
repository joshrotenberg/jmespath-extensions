//! Runtime query storage for sessions
//!
//! Provides in-memory storage for named JMESPath queries that can be
//! defined, retrieved, listed, deleted, and executed during a session.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A stored query with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredQuery {
    /// Query name (identifier)
    pub name: String,
    /// JMESPath expression
    pub expression: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// In-memory store for named queries
#[derive(Debug, Default)]
pub struct QueryStore {
    queries: HashMap<String, StoredQuery>,
}

impl QueryStore {
    /// Create a new empty query store
    pub fn new() -> Self {
        Self::default()
    }

    /// Define (store) a named query
    ///
    /// Returns the previous query if one existed with the same name
    pub fn define(&mut self, query: StoredQuery) -> Option<StoredQuery> {
        self.queries.insert(query.name.clone(), query)
    }

    /// Get a query by name
    pub fn get(&self, name: &str) -> Option<&StoredQuery> {
        self.queries.get(name)
    }

    /// Delete a query by name
    ///
    /// Returns the deleted query if it existed
    pub fn delete(&mut self, name: &str) -> Option<StoredQuery> {
        self.queries.remove(name)
    }

    /// List all stored queries
    pub fn list(&self) -> Vec<&StoredQuery> {
        let mut queries: Vec<_> = self.queries.values().collect();
        queries.sort_by(|a, b| a.name.cmp(&b.name));
        queries
    }

    /// Get the number of stored queries
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Clear all stored queries
    pub fn clear(&mut self) {
        self.queries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let mut store = QueryStore::new();

        let query = StoredQuery {
            name: "count".to_string(),
            expression: "length(@)".to_string(),
            description: Some("Count items".to_string()),
        };

        assert!(store.define(query.clone()).is_none());
        assert_eq!(store.len(), 1);

        let retrieved = store.get("count").unwrap();
        assert_eq!(retrieved.name, "count");
        assert_eq!(retrieved.expression, "length(@)");
        assert_eq!(retrieved.description, Some("Count items".to_string()));
    }

    #[test]
    fn test_define_overwrites() {
        let mut store = QueryStore::new();

        let query1 = StoredQuery {
            name: "test".to_string(),
            expression: "length(@)".to_string(),
            description: None,
        };

        let query2 = StoredQuery {
            name: "test".to_string(),
            expression: "keys(@)".to_string(),
            description: Some("Updated".to_string()),
        };

        assert!(store.define(query1).is_none());
        let old = store.define(query2).unwrap();
        assert_eq!(old.expression, "length(@)");

        let current = store.get("test").unwrap();
        assert_eq!(current.expression, "keys(@)");
    }

    #[test]
    fn test_delete() {
        let mut store = QueryStore::new();

        let query = StoredQuery {
            name: "to_delete".to_string(),
            expression: "`null`".to_string(),
            description: None,
        };

        store.define(query);
        assert_eq!(store.len(), 1);

        let deleted = store.delete("to_delete").unwrap();
        assert_eq!(deleted.name, "to_delete");
        assert_eq!(store.len(), 0);

        assert!(store.delete("nonexistent").is_none());
    }

    #[test]
    fn test_list() {
        let mut store = QueryStore::new();

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
            name: "beta".to_string(),
            expression: "`3`".to_string(),
            description: None,
        });

        let list = store.list();
        assert_eq!(list.len(), 3);
        // Should be sorted alphabetically
        assert_eq!(list[0].name, "alpha");
        assert_eq!(list[1].name, "beta");
        assert_eq!(list[2].name, "zebra");
    }

    #[test]
    fn test_clear() {
        let mut store = QueryStore::new();

        store.define(StoredQuery {
            name: "a".to_string(),
            expression: "`1`".to_string(),
            description: None,
        });
        store.define(StoredQuery {
            name: "b".to_string(),
            expression: "`2`".to_string(),
            description: None,
        });

        assert_eq!(store.len(), 2);
        store.clear();
        assert!(store.is_empty());
    }
}
