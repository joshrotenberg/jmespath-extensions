//! Query library file loading and CLI support.
//!
//! This module provides file I/O and CLI-specific functionality for query libraries.
//! The core parsing logic is in `jmespath_extensions::query_library`.

use anyhow::{Context, Result, anyhow};

// Re-export core types for convenience
pub use jmespath_extensions::query_library::{
    NamedQuery, ParseError, QueryLibrary, is_query_library,
};

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

/// Result of loading a query file.
pub enum LoadResult {
    /// A single expression to evaluate
    Expression(String),
    /// A library to list (for --list-queries)
    List(QueryLibrary),
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
            .map_err(|e| anyhow!("{}", e))
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
