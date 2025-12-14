//! Integration tests for jpx CLI

use std::io::Write;
use std::process::Command;

fn jpx_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jpx"))
}

fn run_query(json: &str, query: &str) -> String {
    let mut child = jpx_cmd()
        .arg(query)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn jpx");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(json.as_bytes())
        .expect("Failed to write to stdin");

    let output = child.wait_with_output().expect("Failed to wait on jpx");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_query_with_file(file: &str, query: &str) -> String {
    let testdata = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/");
    let path = format!("{}{}", testdata, file);

    let output = jpx_cmd()
        .arg("-f")
        .arg(&path)
        .arg(query)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn jpx")
        .wait_with_output()
        .expect("Failed to wait on jpx");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

mod basic_queries {
    use super::*;

    #[test]
    fn test_simple_field_access() {
        let result = run_query(r#"{"name": "Alice", "age": 30}"#, "name");
        assert_eq!(result, r#""Alice""#);
    }

    #[test]
    fn test_nested_field_access() {
        let result = run_query(r#"{"user": {"name": "Bob"}}"#, "user.name");
        assert_eq!(result, r#""Bob""#);
    }

    #[test]
    fn test_array_index() {
        let result = run_query(r#"[1, 2, 3]"#, "[1]");
        assert_eq!(result, "2");
    }

    #[test]
    fn test_array_projection() {
        let result = run_query(r#"[{"a": 1}, {"a": 2}]"#, "[*].a");
        assert_eq!(result, "[\n  1,\n  2\n]");
    }

    #[test]
    fn test_filter_expression() {
        let result = run_query(r#"[{"age": 20}, {"age": 30}]"#, "[?age > `25`]");
        assert_eq!(result, "[\n  {\n    \"age\": 30\n  }\n]");
    }
}

mod extension_functions {
    use super::*;

    #[test]
    fn test_unique() {
        let result = run_query(r#"[1, 2, 2, 3, 3, 3]"#, "unique(@)");
        assert_eq!(result, "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_flatten_deep() {
        let result = run_query(r#"[[1, [2, [3]]]]"#, "flatten_deep(@)");
        assert_eq!(result, "[\n  1,\n  2,\n  3\n]");
    }

    #[test]
    fn test_split() {
        // split(string, delimiter) - splits the string by delimiter
        let result = run_query(r#""a,b,c""#, "split(@, ',')");
        assert_eq!(result, "[\n  \"a\",\n  \"b\",\n  \"c\"\n]");
    }

    #[test]
    fn test_trim() {
        let result = run_query(r#""  hello  ""#, "trim(@)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_now() {
        // now() returns a number (Unix timestamp)
        let result = run_query(r#"null"#, "type(now())");
        assert_eq!(result, r#""number""#);
    }

    #[test]
    fn test_from_items() {
        // from_items converts [[key, value], ...] pairs to object
        let result = run_query(r#"[["a", 1]]"#, "from_items(@)");
        assert_eq!(result, "{\n  \"a\": 1\n}");
    }

    #[test]
    fn test_items() {
        // items converts object to [[key, value], ...] pairs
        let result = run_query(r#"{"a": 1}"#, "items(@)");
        assert!(result.contains("\"a\""));
        assert!(result.contains("1"));
    }

    #[test]
    fn test_group_by_expr() {
        // group_by_expr('expression', array)
        let result = run_query(
            r#"[{"role": "admin", "name": "Alice"}, {"role": "user", "name": "Bob"}, {"role": "admin", "name": "Carol"}]"#,
            "group_by_expr('role', @)",
        );
        assert!(result.contains("\"admin\""));
        assert!(result.contains("\"user\""));
    }

    #[test]
    fn test_map_values() {
        let result = run_query(r#"{"a": 1, "b": 2}"#, "map_values('multiply(@, `2`)', @)");
        assert!(result.contains("\"a\": 2"));
        assert!(result.contains("\"b\": 4"));
    }

    #[test]
    fn test_reduce_expr() {
        // reduce_expr(expression, array, initial)
        let result = run_query(
            r#"[1, 2, 3, 4, 5]"#,
            "reduce_expr('add(accumulator, current)', @, `0`)",
        );
        assert_eq!(result, "15.0");
    }

    #[test]
    fn test_filter_expr() {
        // filter_expr('expression', array)
        let result = run_query(r#"[1, 2, 3, 4, 5]"#, "filter_expr('@ > `3`', @)");
        assert_eq!(result, "[\n  4,\n  5\n]");
    }
}

mod file_operations {
    use super::*;

    #[test]
    fn test_load_users_file() {
        let result = run_query_with_file("users.json", "length(@)");
        assert_eq!(result, "5");
    }

    #[test]
    fn test_users_filter() {
        let result =
            run_query_with_file("users.json", "[?department == 'Engineering'] | length(@)");
        assert_eq!(result, "3");
    }

    #[test]
    fn test_users_projection() {
        let result = run_query_with_file("users.json", "[*].name | sort(@)");
        assert!(result.contains("Alice Johnson"));
        assert!(result.contains("Eva Martinez"));
    }

    #[test]
    fn test_users_aggregation() {
        let result = run_query_with_file("users.json", "avg([*].salary)");
        assert_eq!(result, "95000.0");
    }

    #[test]
    fn test_users_group_by() {
        let result = run_query_with_file(
            "users.json",
            "group_by_expr('department', @) | map_values('length(@)', @)",
        );
        assert!(result.contains("\"Engineering\": 3"));
        assert!(result.contains("\"Marketing\": 1"));
        assert!(result.contains("\"Sales\": 1"));
    }
}

mod cli_options {
    use super::*;

    #[test]
    fn test_compact_output() {
        let mut child = jpx_cmd()
            .arg("-c")
            .arg("[*].a")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"[{\"a\": 1}, {\"a\": 2}]")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(result, "[1,2]");
    }

    #[test]
    fn test_raw_output() {
        let mut child = jpx_cmd()
            .arg("-r")
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"\"hello world\"")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_list_functions() {
        // Use --list-category instead of --list
        let output = jpx_cmd()
            .arg("--list-category")
            .arg("array")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let result = String::from_utf8_lossy(&output.stdout);
        assert!(result.contains("unique"));
        assert!(result.contains("flatten"));
    }

    #[test]
    fn test_describe_function() {
        let output = jpx_cmd()
            .arg("--describe")
            .arg("unique")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let result = String::from_utf8_lossy(&output.stdout);
        assert!(result.contains("unique"));
        assert!(result.contains("array"));
    }

    #[test]
    fn test_version() {
        let output = jpx_cmd()
            .arg("--version")
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx")
            .wait_with_output()
            .expect("Failed to wait");

        let result = String::from_utf8_lossy(&output.stdout);
        assert!(result.contains("jpx"));
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_invalid_json() {
        let mut child = jpx_cmd()
            .arg("@")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"not valid json")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(!output.status.success());
    }

    #[test]
    fn test_invalid_query() {
        let mut child = jpx_cmd()
            .arg("[[[invalid")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn jpx");

        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"{}")
            .expect("Failed to write");

        let output = child.wait_with_output().expect("Failed to wait");
        assert!(!output.status.success());
    }
}
