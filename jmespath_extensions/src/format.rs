//! CSV and TSV formatting functions.
//!
//! This module provides functions for formatting data as CSV/TSV strings,
//! similar to jq's `@csv` and `@tsv` formatters.
//!
//! Uses the [`csv`](https://docs.rs/csv) crate for RFC 4180 compliant output.
//!
//! # Example
//!
//! ```rust
//! use jmespath::{Runtime, Variable};
//! use jmespath_extensions::format;
//!
//! let mut runtime = Runtime::new();
//! runtime.register_builtin_functions();
//! format::register(&mut runtime);
//! ```

use std::rc::Rc;

use csv::WriterBuilder;

use crate::common::{ArgumentType, Context, Function, JmespathError, Rcvar, Runtime, Variable};
use crate::define_function;

/// Register all format functions with the runtime.
pub fn register(runtime: &mut Runtime) {
    runtime.register_function("to_csv", Box::new(ToCsvFn::new()));
    runtime.register_function("to_tsv", Box::new(ToTsvFn::new()));
    runtime.register_function("to_csv_rows", Box::new(ToCsvRowsFn::new()));
    runtime.register_function("to_csv_table", Box::new(ToCsvTableFn::new()));
}

/// Convert a JMESPath Variable to a string suitable for CSV field.
fn variable_to_csv_string(value: &Variable) -> String {
    match value {
        Variable::Null => String::new(),
        Variable::Bool(b) => b.to_string(),
        Variable::Number(n) => n.to_string(),
        Variable::String(s) => s.clone(),
        Variable::Array(_) | Variable::Object(_) => {
            // For complex types, serialize as JSON
            serde_json::to_string(value).unwrap_or_default()
        }
        Variable::Expref(_) => String::new(),
    }
}

/// Write a single row using the csv crate's writer.
fn write_csv_row(fields: &[String], delimiter: u8) -> Result<String, std::io::Error> {
    let mut wtr = WriterBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_writer(vec![]);

    wtr.write_record(fields)?;
    wtr.flush()?;

    let data = wtr
        .into_inner()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Remove trailing newline that csv crate adds
    let mut s = String::from_utf8(data).unwrap_or_default();
    if s.ends_with('\n') {
        s.pop();
    }
    if s.ends_with('\r') {
        s.pop();
    }
    Ok(s)
}

/// Write multiple rows using the csv crate's writer.
fn write_csv_rows(rows: &[Vec<String>], delimiter: u8) -> Result<String, std::io::Error> {
    let mut wtr = WriterBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .from_writer(vec![]);

    for row in rows {
        wtr.write_record(row)?;
    }
    wtr.flush()?;

    let data = wtr
        .into_inner()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Remove trailing newline
    let mut s = String::from_utf8(data).unwrap_or_default();
    if s.ends_with('\n') {
        s.pop();
    }
    if s.ends_with('\r') {
        s.pop();
    }
    Ok(s)
}

// =============================================================================
// to_csv(array) -> string
// =============================================================================

define_function!(ToCsvFn, vec![ArgumentType::Array], None);

impl Function for ToCsvFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        // Empty array returns empty string (matches jq @csv behavior)
        if arr.is_empty() {
            return Ok(Rc::new(Variable::String(String::new())));
        }

        let fields: Vec<String> = arr.iter().map(|v| variable_to_csv_string(v)).collect();

        match write_csv_row(&fields, b',') {
            Ok(s) => Ok(Rc::new(Variable::String(s))),
            Err(e) => Err(crate::common::custom_error(
                ctx,
                &format!("CSV write error: {}", e),
            )),
        }
    }
}

// =============================================================================
// to_tsv(array) -> string
// =============================================================================

define_function!(ToTsvFn, vec![ArgumentType::Array], None);

impl Function for ToTsvFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let arr = args[0].as_array().unwrap();

        // Empty array returns empty string (matches jq @tsv behavior)
        if arr.is_empty() {
            return Ok(Rc::new(Variable::String(String::new())));
        }

        let fields: Vec<String> = arr.iter().map(|v| variable_to_csv_string(v)).collect();

        match write_csv_row(&fields, b'\t') {
            Ok(s) => Ok(Rc::new(Variable::String(s))),
            Err(e) => Err(crate::common::custom_error(
                ctx,
                &format!("TSV write error: {}", e),
            )),
        }
    }
}

// =============================================================================
// to_csv_rows(array_of_arrays) -> string
// =============================================================================

define_function!(ToCsvRowsFn, vec![ArgumentType::Array], None);

impl Function for ToCsvRowsFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let rows_var = args[0].as_array().unwrap();

        if rows_var.is_empty() {
            return Ok(Rc::new(Variable::String(String::new())));
        }

        let rows: Vec<Vec<String>> = rows_var
            .iter()
            .map(|row| {
                if let Some(arr) = row.as_array() {
                    arr.iter().map(|v| variable_to_csv_string(v)).collect()
                } else {
                    // Single value becomes a single-column row
                    vec![variable_to_csv_string(row)]
                }
            })
            .collect();

        match write_csv_rows(&rows, b',') {
            Ok(s) => Ok(Rc::new(Variable::String(s))),
            Err(e) => Err(crate::common::custom_error(
                ctx,
                &format!("CSV write error: {}", e),
            )),
        }
    }
}

// =============================================================================
// to_csv_table(array_of_objects, columns?) -> string
// =============================================================================

define_function!(
    ToCsvTableFn,
    vec![ArgumentType::Array],
    Some(ArgumentType::Array)
);

impl Function for ToCsvTableFn {
    fn evaluate(&self, args: &[Rcvar], ctx: &mut Context<'_>) -> Result<Rcvar, JmespathError> {
        self.signature.validate(args, ctx)?;

        let rows = args[0].as_array().unwrap();

        if rows.is_empty() {
            return Ok(Rc::new(Variable::String(String::new())));
        }

        // Determine columns: from second argument or from first object's keys
        let columns: Vec<String> = if args.len() > 1 {
            // Explicit columns provided
            args[1]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_string().map(|s| s.to_string()))
                .collect()
        } else {
            // Infer from first object
            if let Some(obj) = rows[0].as_object() {
                let mut keys: Vec<String> = obj.keys().cloned().collect();
                keys.sort(); // Consistent ordering
                keys
            } else {
                return Ok(Rc::new(Variable::String(String::new())));
            }
        };

        if columns.is_empty() {
            return Ok(Rc::new(Variable::String(String::new())));
        }

        // Build all rows (header + data)
        let mut all_rows: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);

        // Header row
        all_rows.push(columns.clone());

        // Data rows
        for row in rows.iter() {
            if let Some(obj) = row.as_object() {
                let data_row: Vec<String> = columns
                    .iter()
                    .map(|col| {
                        obj.get(col)
                            .map(|v| variable_to_csv_string(v))
                            .unwrap_or_default()
                    })
                    .collect();
                all_rows.push(data_row);
            } else {
                // Non-object row - empty values
                all_rows.push(columns.iter().map(|_| String::new()).collect());
            }
        }

        match write_csv_rows(&all_rows, b',') {
            Ok(s) => Ok(Rc::new(Variable::String(s))),
            Err(e) => Err(crate::common::custom_error(
                ctx,
                &format!("CSV write error: {}", e),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jmespath::Runtime;

    fn setup_runtime() -> Runtime {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        register(&mut runtime);
        runtime
    }

    // =========================================================================
    // to_csv tests
    // =========================================================================

    #[test]
    fn test_to_csv_simple() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"["a", "b", "c"]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "a,b,c");
    }

    #[test]
    fn test_to_csv_mixed_types() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"["hello", 42, true, null]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello,42,true,");
    }

    #[test]
    fn test_to_csv_with_comma() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"["hello, world", "test"]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "\"hello, world\",test");
    }

    #[test]
    fn test_to_csv_with_quotes() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"["say \"hello\"", "test"]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "\"say \"\"hello\"\"\",test");
    }

    #[test]
    fn test_to_csv_with_newline() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"["line1\nline2", "test"]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "\"line1\nline2\",test");
    }

    #[test]
    fn test_to_csv_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "");
    }

    #[test]
    fn test_to_csv_with_leading_trailing_space() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv(@)").unwrap();
        let data = Variable::from_json(r#"["  hello  ", "test"]"#).unwrap();
        let result = expr.search(&data).unwrap();
        // csv crate quotes fields with leading/trailing whitespace to preserve them
        assert!(result.as_string().unwrap().contains("hello"));
    }

    // =========================================================================
    // to_tsv tests
    // =========================================================================

    #[test]
    fn test_to_tsv_simple() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_tsv(@)").unwrap();
        let data = Variable::from_json(r#"["a", "b", "c"]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "a\tb\tc");
    }

    #[test]
    fn test_to_tsv_mixed_types() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_tsv(@)").unwrap();
        let data = Variable::from_json(r#"["hello", 42, true, null]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello\t42\ttrue\t");
    }

    // =========================================================================
    // to_csv_rows tests
    // =========================================================================

    #[test]
    fn test_to_csv_rows_simple() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_rows(@)").unwrap();
        let data = Variable::from_json(r#"[[1, 2, 3], [4, 5, 6]]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "1,2,3\n4,5,6");
    }

    #[test]
    fn test_to_csv_rows_with_strings() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_rows(@)").unwrap();
        let data = Variable::from_json(r#"[["a", "b"], ["c", "d"]]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "a,b\nc,d");
    }

    #[test]
    fn test_to_csv_rows_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_rows(@)").unwrap();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "");
    }

    #[test]
    fn test_to_csv_rows_with_special_chars() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_rows(@)").unwrap();
        let data = Variable::from_json(r#"[["hello, world", "test"], ["a\"b", "c"]]"#).unwrap();
        let result = expr.search(&data).unwrap();
        // Should properly escape commas and quotes
        assert!(result.as_string().unwrap().contains("\"hello, world\""));
        assert!(result.as_string().unwrap().contains("\"a\"\"b\""));
    }

    // =========================================================================
    // to_csv_table tests
    // =========================================================================

    #[test]
    fn test_to_csv_table_simple() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_table(@)").unwrap();
        let data =
            Variable::from_json(r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]"#)
                .unwrap();
        let result = expr.search(&data).unwrap();
        // Keys are sorted alphabetically
        assert_eq!(result.as_string().unwrap(), "age,name\n30,alice\n25,bob");
    }

    #[test]
    fn test_to_csv_table_with_columns() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("to_csv_table(@, `[\"name\", \"age\"]`)")
            .unwrap();
        let data =
            Variable::from_json(r#"[{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]"#)
                .unwrap();
        let result = expr.search(&data).unwrap();
        // Columns in specified order
        assert_eq!(result.as_string().unwrap(), "name,age\nalice,30\nbob,25");
    }

    #[test]
    fn test_to_csv_table_missing_field() {
        let runtime = setup_runtime();
        let expr = runtime
            .compile("to_csv_table(@, `[\"name\", \"age\", \"email\"]`)")
            .unwrap();
        let data =
            Variable::from_json(r#"[{"name": "alice", "age": 30}, {"name": "bob"}]"#).unwrap();
        let result = expr.search(&data).unwrap();
        // Missing fields are empty
        assert_eq!(
            result.as_string().unwrap(),
            "name,age,email\nalice,30,\nbob,,"
        );
    }

    #[test]
    fn test_to_csv_table_empty() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_table(@)").unwrap();
        let data = Variable::from_json(r#"[]"#).unwrap();
        let result = expr.search(&data).unwrap();
        assert_eq!(result.as_string().unwrap(), "");
    }

    #[test]
    fn test_to_csv_table_special_chars() {
        let runtime = setup_runtime();
        let expr = runtime.compile("to_csv_table(@)").unwrap();
        let data =
            Variable::from_json(r#"[{"name": "O'Brien, Jr.", "note": "said \"hi\""}]"#).unwrap();
        let result = expr.search(&data).unwrap();
        // Should properly escape commas and quotes
        assert!(result.as_string().unwrap().contains("\"O'Brien, Jr.\""));
        assert!(result.as_string().unwrap().contains("\"said \"\"hi\"\"\""));
    }
}
