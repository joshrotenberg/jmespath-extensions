//! Named Query Library support for `.jpx` files.
//!
//! Allows defining multiple named, reusable queries in a single file:
//!
//! ```text
//! -- :name top-keywords
//! -- :desc Extract top keywords from text field
//! tokens(@) | remove_stopwords(@) | stems(@) | frequencies(@)
//!
//! -- :name clean-html
//! -- :desc Strip HTML tags and normalize whitespace
//! regex_replace(@, `<[^>]+>`, ` `) | collapse_whitespace(@)
//! ```

use anyhow::{Context, Result, anyhow};

/// A named query with optional description.
#[derive(Debug, Clone)]
pub struct NamedQuery {
    /// Query name (used for lookup)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// The JMESPath expression
    pub expression: String,
    /// Line number where the query starts (for error messages)
    pub line_number: usize,
}

/// A collection of named queries parsed from a `.jpx` file.
#[derive(Debug, Default)]
pub struct QueryLibrary {
    queries: Vec<NamedQuery>,
}

impl QueryLibrary {
    /// Parse a query library from file content.
    ///
    /// Format:
    /// - `-- :name <name>` starts a new query
    /// - `-- :desc <description>` adds a description to the current query
    /// - `-- ` other comment lines are ignored
    /// - Non-comment lines are appended to the current query's expression
    pub fn parse(content: &str) -> Result<Self> {
        let mut queries = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_desc: Option<String> = None;
        let mut current_expr = String::new();
        let mut current_line_number = 0usize;

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1; // 1-indexed for error messages
            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix("-- :name ").or_else(|| {
                // Handle "-- :name" without trailing space (empty name case)
                if trimmed == "-- :name" {
                    Some("")
                } else {
                    None
                }
            }) {
                // Save previous query if exists
                if let Some(name) = current_name.take() {
                    let expr = current_expr.trim().to_string();
                    if expr.is_empty() {
                        return Err(anyhow!(
                            "Query '{}' at line {} has no expression",
                            name,
                            current_line_number
                        ));
                    }
                    queries.push(NamedQuery {
                        name,
                        description: current_desc.take(),
                        expression: expr,
                        line_number: current_line_number,
                    });
                    current_expr.clear();
                }

                // Start new query
                let name = rest.trim().to_string();
                if name.is_empty() {
                    return Err(anyhow!("Empty query name at line {}", line_number));
                }

                // Check for duplicates
                if queries.iter().any(|q| q.name == name) {
                    return Err(anyhow!(
                        "Duplicate query name '{}' at line {}",
                        name,
                        line_number
                    ));
                }

                current_name = Some(name);
                current_line_number = line_number;
            } else if let Some(rest) = trimmed.strip_prefix("-- :desc ") {
                // Add description to current query
                if current_name.is_some() {
                    current_desc = Some(rest.trim().to_string());
                }
            } else if trimmed.starts_with("-- ") || trimmed == "--" {
                // Skip other comments
            } else if !trimmed.is_empty() {
                // Append to current expression
                if current_name.is_some() {
                    if !current_expr.is_empty() {
                        current_expr.push('\n');
                    }
                    current_expr.push_str(line);
                }
            }
        }

        // Save final query
        if let Some(name) = current_name {
            let expr = current_expr.trim().to_string();
            if expr.is_empty() {
                return Err(anyhow!(
                    "Query '{}' at line {} has no expression",
                    name,
                    current_line_number
                ));
            }
            queries.push(NamedQuery {
                name,
                description: current_desc,
                expression: expr,
                line_number: current_line_number,
            });
        }

        if queries.is_empty() {
            return Err(anyhow!(
                "No queries found. Use '-- :name <query-name>' to define queries."
            ));
        }

        Ok(QueryLibrary { queries })
    }

    /// Get a query by name.
    pub fn get(&self, name: &str) -> Option<&NamedQuery> {
        self.queries.iter().find(|q| q.name == name)
    }

    /// Get all queries.
    pub fn list(&self) -> &[NamedQuery] {
        &self.queries
    }

    /// Get query names.
    pub fn names(&self) -> Vec<&str> {
        self.queries.iter().map(|q| q.name.as_str()).collect()
    }
}

/// Check if content looks like a query library (starts with `-- :name`).
pub fn is_query_library(content: &str) -> bool {
    content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().starts_with("-- :name "))
        .unwrap_or(false)
}

/// Parse a query file path that may contain a colon-separated query name.
///
/// Examples:
/// - `queries.jpx:top-keywords` → `("queries.jpx", Some("top-keywords"))`
/// - `queries.jpx` → `("queries.jpx", None)`
/// - `C:\path\file.jpx:query` → `("C:\path\file.jpx", Some("query"))` (Windows)
pub fn parse_query_path(path: &str) -> (&str, Option<&str>) {
    // Handle Windows paths (e.g., C:\path\file.jpx:query)
    // Look for colon only after the path portion
    if let Some(last_colon) = path.rfind(':') {
        // Check if this looks like a Windows drive letter (single char before colon at position 1)
        let is_windows_drive = last_colon == 1
            && path
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false);

        // Also check if the colon is within the extension (e.g., .jpx:query vs C:\)
        if !is_windows_drive && last_colon > 0 {
            let (file_part, query_part) = path.split_at(last_colon);
            let query_name = &query_part[1..]; // Skip the colon
            if !query_name.is_empty() && !query_name.contains(['/', '\\']) {
                return (file_part, Some(query_name));
            }
        }
    }
    (path, None)
}

/// Load an expression from a query file.
///
/// Handles both single-query files and `.jpx` query libraries.
pub fn load_query_expression(
    path: &str,
    query_name: Option<&str>,
    list_mode: bool,
) -> Result<LoadResult> {
    // Parse colon syntax
    let (file_path, colon_query) = parse_query_path(path);
    let query_name = query_name.or(colon_query);

    // Read file
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read query file: {}", file_path))?;

    // Determine if this is a query library
    let is_library = file_path.ends_with(".jpx") || is_query_library(&content);

    if is_library {
        let library = QueryLibrary::parse(&content)
            .with_context(|| format!("Failed to parse query library: {}", file_path))?;

        if list_mode {
            return Ok(LoadResult::List(library));
        }

        match query_name {
            Some(name) => {
                let query = library.get(name).ok_or_else(|| {
                    let available = library.names().join(", ");
                    anyhow!(
                        "Query '{}' not found in {}. Available queries: {}",
                        name,
                        file_path,
                        available
                    )
                })?;
                Ok(LoadResult::Expression(query.expression.clone()))
            }
            None => {
                let available = library.names().join(", ");
                Err(anyhow!(
                    "Query library requires --query <name> or use colon syntax ({}:query_name). Available queries: {}",
                    file_path,
                    available
                ))
            }
        }
    } else {
        // Plain single-query file (backwards compatible)
        if query_name.is_some() {
            return Err(anyhow!(
                "Cannot use --query with a plain query file. Use a .jpx file for named queries."
            ));
        }
        Ok(LoadResult::Expression(content.trim().to_string()))
    }
}

/// Result of loading a query file.
pub enum LoadResult {
    /// A single expression to evaluate
    Expression(String),
    /// A library to list (for --list-queries)
    List(QueryLibrary),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_library() {
        let content = r#"
-- :name greet
-- :desc Simple greeting
`"hello"`

-- :name count
length(@)
"#;
        let lib = QueryLibrary::parse(content).unwrap();
        assert_eq!(lib.queries.len(), 2);

        let greet = lib.get("greet").unwrap();
        assert_eq!(greet.name, "greet");
        assert_eq!(greet.description, Some("Simple greeting".to_string()));
        assert_eq!(greet.expression, "`\"hello\"`");

        let count = lib.get("count").unwrap();
        assert_eq!(count.name, "count");
        assert_eq!(count.description, None);
        assert_eq!(count.expression, "length(@)");
    }

    #[test]
    fn test_parse_multiline_expression() {
        let content = r#"
-- :name complex
-- :desc Multi-line query
{
  total: length(@),
  first: @[0]
}
"#;
        let lib = QueryLibrary::parse(content).unwrap();
        let query = lib.get("complex").unwrap();
        assert!(query.expression.contains("total: length(@)"));
        assert!(query.expression.contains("first: @[0]"));
    }

    #[test]
    fn test_parse_empty_name_error() {
        let content = "-- :name \nlength(@)";
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty query name"));
    }

    #[test]
    fn test_parse_duplicate_name_error() {
        let content = r#"
-- :name foo
length(@)

-- :name foo
keys(@)
"#;
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Duplicate query name")
        );
    }

    #[test]
    fn test_parse_no_expression_error() {
        let content = "-- :name empty\n-- :name another\nlength(@)";
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("has no expression")
        );
    }

    #[test]
    fn test_parse_no_queries_error() {
        let content = "-- just a comment\nlength(@)";
        let result = QueryLibrary::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No queries found"));
    }

    #[test]
    fn test_is_query_library() {
        assert!(is_query_library("-- :name foo\nlength(@)"));
        assert!(is_query_library("  -- :name foo\nlength(@)"));
        assert!(is_query_library("\n-- :name foo\nlength(@)"));
        assert!(!is_query_library("length(@)"));
        assert!(!is_query_library("-- comment\nlength(@)"));
    }

    #[test]
    fn test_parse_query_path() {
        assert_eq!(
            parse_query_path("file.jpx:query"),
            ("file.jpx", Some("query"))
        );
        assert_eq!(parse_query_path("file.jpx"), ("file.jpx", None));
        assert_eq!(
            parse_query_path("path/to/file.jpx:query"),
            ("path/to/file.jpx", Some("query"))
        );
        assert_eq!(parse_query_path("query"), ("query", None));
    }

    #[test]
    fn test_comments_ignored() {
        let content = r#"
-- :name test
-- :desc Description
-- This is a regular comment
-- Another comment
length(@)
-- Trailing comment
"#;
        let lib = QueryLibrary::parse(content).unwrap();
        let query = lib.get("test").unwrap();
        assert_eq!(query.expression, "length(@)");
    }
}
