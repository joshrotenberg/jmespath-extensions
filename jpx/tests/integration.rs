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

mod path_functions {
    use super::*;

    #[test]
    fn test_get_path_dot_notation() {
        let result = run_query(r#"{"a": {"b": {"c": 42}}}"#, "get_path(@, `\"a.b.c\"`)");
        assert_eq!(result, "42");
    }

    #[test]
    fn test_get_path_with_default() {
        let result = run_query(r#"{"a": 1}"#, "get_path(@, `\"a.b.c\"`, `\"missing\"`)");
        assert_eq!(result, r#""missing""#);
    }

    #[test]
    fn test_get_path_array_index() {
        let result = run_query(
            r#"{"users": [{"name": "alice"}, {"name": "bob"}]}"#,
            "get_path(@, `\"users.0.name\"`)",
        );
        assert_eq!(result, r#""alice""#);
    }

    #[test]
    fn test_has_path_exists() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "has_path(@, `\"a.b\"`)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_has_path_missing() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "has_path(@, `\"a.c\"`)");
        assert_eq!(result, "false");
    }

    #[test]
    fn test_has_path_array_index() {
        let result = run_query(r#"{"items": [1, 2, 3]}"#, "has_path(@, `\"items.1\"`)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_set_path_dot_notation() {
        let result = run_query(r#"{"a": {}}"#, "set_path(@, `\"a.b\"`, `99`)");
        assert!(result.contains("\"b\": 99"));
    }

    #[test]
    fn test_set_path_creates_nested() {
        let result = run_query(r#"{}"#, "set_path(@, `\"a.b.c\"`, `\"deep\"`)");
        assert!(result.contains("\"c\": \"deep\""));
    }

    #[test]
    fn test_set_path_array_index() {
        let result = run_query(
            r#"{"items": [1, 2, 3]}"#,
            "set_path(@, `\"items.1\"`, `99`)",
        );
        assert!(result.contains("99"));
    }

    #[test]
    fn test_delete_path_dot_notation() {
        let result = run_query(r#"{"a": {"b": 1, "c": 2}}"#, "delete_path(@, `\"a.b\"`)");
        assert!(!result.contains("\"b\":"));
        assert!(result.contains("\"c\": 2"));
    }

    #[test]
    fn test_delete_path_array_index() {
        let result = run_query(r#"{"items": [1, 2, 3]}"#, "delete_path(@, `\"items.1\"`)");
        // After deleting index 1 (value 2), should have [1, 3]
        assert!(result.contains("1"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_get_alias_works() {
        // get and get_path should be equivalent
        let result1 = run_query(r#"{"a": {"b": 1}}"#, "get(@, `\"a.b\"`)");
        let result2 = run_query(r#"{"a": {"b": 1}}"#, "get_path(@, `\"a.b\"`)");
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_has_alias_works() {
        // has and has_path should be equivalent
        let result1 = run_query(r#"{"a": {"b": 1}}"#, "has(@, `\"a.b\"`)");
        let result2 = run_query(r#"{"a": {"b": 1}}"#, "has_path(@, `\"a.b\"`)");
        assert_eq!(result1, result2);
    }
}

mod object_functions {
    use super::*;

    #[test]
    fn test_pick() {
        let result = run_query(r#"{"a": 1, "b": 2, "c": 3}"#, "pick(@, `[\"a\", \"c\"]`)");
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 3"));
        assert!(!result.contains("\"b\""));
    }

    #[test]
    fn test_omit() {
        let result = run_query(r#"{"a": 1, "b": 2, "c": 3}"#, "omit(@, `[\"a\", \"c\"]`)");
        assert!(!result.contains("\"a\""));
        assert!(result.contains("\"b\": 2"));
        assert!(!result.contains("\"c\""));
    }

    #[test]
    fn test_deep_merge() {
        let result = run_query(
            r#"[{"a": {"b": 1}}, {"a": {"c": 2}}]"#,
            "deep_merge([0], [1])",
        );
        assert!(result.contains("\"b\": 1"));
        assert!(result.contains("\"c\": 2"));
    }

    #[test]
    fn test_defaults() {
        let result = run_query(r#"{"a": 1}"#, "defaults(@, `{\"a\": 99, \"b\": 2}`)");
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"b\": 2"));
    }

    #[test]
    fn test_rename_keys() {
        let result = run_query(
            r#"{"old_name": 1}"#,
            "rename_keys(@, `{\"old_name\": \"new_name\"}`)",
        );
        assert!(result.contains("\"new_name\": 1"));
        assert!(!result.contains("\"old_name\""));
    }

    #[test]
    fn test_invert() {
        let result = run_query(r#"{"a": "x", "b": "y"}"#, "invert(@)");
        assert!(result.contains("\"x\": \"a\""));
        assert!(result.contains("\"y\": \"b\""));
    }
}

mod type_functions {
    use super::*;

    #[test]
    fn test_to_boolean_string_true() {
        let result = run_query(r#""true""#, "to_boolean(@)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_to_boolean_string_yes() {
        let result = run_query(r#""yes""#, "to_boolean(@)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_to_boolean_number() {
        let result = run_query(r#"1"#, "to_boolean(@)");
        assert_eq!(result, "true");
    }

    #[test]
    fn test_to_boolean_zero() {
        let result = run_query(r#"0"#, "to_boolean(@)");
        assert_eq!(result, "false");
    }

    #[test]
    fn test_parse_numbers() {
        let result = run_query(r#"{"count": "42", "name": "alice"}"#, "parse_numbers(@)");
        assert!(result.contains("\"count\": 42"));
        assert!(result.contains("\"name\": \"alice\""));
    }

    #[test]
    fn test_parse_booleans() {
        let result = run_query(
            r#"{"active": "true", "name": "alice"}"#,
            "parse_booleans(@)",
        );
        assert!(result.contains("\"active\": true"));
        assert!(result.contains("\"name\": \"alice\""));
    }

    #[test]
    fn test_auto_parse() {
        let result = run_query(
            r#"{"num": "42", "bool": "true", "nil": "null", "str": "hello"}"#,
            "auto_parse(@)",
        );
        assert!(result.contains("\"num\": 42"));
        assert!(result.contains("\"bool\": true"));
        assert!(result.contains("\"nil\": null"));
        assert!(result.contains("\"str\": \"hello\""));
    }

    #[test]
    fn test_type_of() {
        assert_eq!(run_query(r#""hello""#, "type_of(@)"), r#""string""#);
        assert_eq!(run_query(r#"42"#, "type_of(@)"), r#""number""#);
        assert_eq!(run_query(r#"true"#, "type_of(@)"), r#""boolean""#);
        assert_eq!(run_query(r#"[1,2]"#, "type_of(@)"), r#""array""#);
        assert_eq!(run_query(r#"{"a":1}"#, "type_of(@)"), r#""object""#);
    }

    #[test]
    fn test_is_string() {
        assert_eq!(run_query(r#""hello""#, "is_string(@)"), "true");
        assert_eq!(run_query(r#"42"#, "is_string(@)"), "false");
    }

    #[test]
    fn test_is_number() {
        assert_eq!(run_query(r#"42"#, "is_number(@)"), "true");
        assert_eq!(run_query(r#""42""#, "is_number(@)"), "false");
    }

    #[test]
    fn test_is_empty() {
        assert_eq!(run_query(r#"[]"#, "is_empty(@)"), "true");
        assert_eq!(run_query(r#"{}"#, "is_empty(@)"), "true");
        assert_eq!(run_query(r#""""#, "is_empty(@)"), "true");
        assert_eq!(run_query(r#"[1]"#, "is_empty(@)"), "false");
    }
}

mod cleanup_functions {
    use super::*;

    #[test]
    fn test_remove_nulls() {
        let result = run_query(r#"{"a": 1, "b": null, "c": 2}"#, "remove_nulls(@)");
        assert!(result.contains("\"a\": 1"));
        assert!(result.contains("\"c\": 2"));
        assert!(!result.contains("null"));
    }

    #[test]
    fn test_remove_empty() {
        let result = run_query(
            r#"{"a": "", "b": "hello", "c": [], "d": [1]}"#,
            "remove_empty(@)",
        );
        assert!(result.contains("\"b\": \"hello\""));
        assert!(result.contains("\"d\""));
    }

    #[test]
    fn test_remove_empty_strings() {
        let result = run_query(
            r#"{"name": "alice", "bio": "", "city": "nyc"}"#,
            "remove_empty_strings(@)",
        );
        assert!(result.contains("\"name\": \"alice\""));
        assert!(result.contains("\"city\": \"nyc\""));
        assert!(!result.contains("\"bio\""));
    }
}

mod array_functions {
    use super::*;

    #[test]
    fn test_index_by_simple() {
        let result = run_query(
            r#"[{"id": "a", "value": 1}, {"id": "b", "value": 2}]"#,
            "index_by(@, 'id')",
        );
        assert!(result.contains("\"a\":"));
        assert!(result.contains("\"b\":"));
    }

    #[test]
    fn test_index_at() {
        assert_eq!(run_query(r#"[1, 2, 3]"#, "index_at(@, `0`)"), "1");
        assert_eq!(run_query(r#"[1, 2, 3]"#, "index_at(@, `-1`)"), "3");
    }

    #[test]
    fn test_find_index() {
        assert_eq!(run_query(r#"[1, 2, 3]"#, "find_index(@, `2`)"), "1");
    }

    #[test]
    fn test_zip() {
        let result = run_query(r#"{"a": [1, 2], "b": ["x", "y"]}"#, "zip(a, b)");
        assert!(result.contains("1"));
        assert!(result.contains("\"x\""));
    }

    #[test]
    fn test_intersection() {
        let result = run_query(r#"[[1, 2, 3], [2, 3, 4]]"#, "intersection([0], [1])");
        assert!(result.contains("2"));
        assert!(result.contains("3"));
        assert!(!result.contains("1"));
        assert!(!result.contains("4"));
    }

    #[test]
    fn test_difference() {
        let result = run_query(r#"[[1, 2, 3], [2, 3, 4]]"#, "difference([0], [1])");
        assert!(result.contains("1"));
        assert!(!result.contains("2"));
    }
}

mod flatten_unflatten {
    use super::*;

    #[test]
    fn test_flatten_keys() {
        let result = run_query(r#"{"a": {"b": {"c": 1}}}"#, "flatten_keys(@)");
        assert!(result.contains("\"a.b.c\": 1"));
    }

    #[test]
    fn test_flatten_alias() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "flatten(@)");
        assert!(result.contains("\"a.b\": 1"));
    }

    #[test]
    fn test_unflatten_keys() {
        let result = run_query(r#"{"a.b.c": 1, "a.b.d": 2}"#, "unflatten_keys(@)");
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\""));
        assert!(result.contains("\"c\": 1"));
        assert!(result.contains("\"d\": 2"));
    }

    #[test]
    fn test_unflatten_alias() {
        let result = run_query(r#"{"a.b": 1}"#, "unflatten(@)");
        assert!(result.contains("\"a\""));
        assert!(result.contains("\"b\": 1"));
    }

    #[test]
    fn test_flatten_array() {
        let result = run_query(r#"{"a": [1, 2]}"#, "flatten_array(@)");
        assert!(result.contains("\"a.0\": 1"));
        assert!(result.contains("\"a.1\": 2"));
    }
}

mod key_functions {
    use super::*;

    #[test]
    fn test_pluck_deep() {
        let result = run_query(
            r#"{"users": [{"id": 1}, {"id": 2}], "meta": {"id": 99}}"#,
            "pluck_deep(@, 'id')",
        );
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("99"));
    }

    #[test]
    fn test_paths_to() {
        let result = run_query(r#"{"a": {"id": 1}, "b": {"id": 2}}"#, "paths_to(@, 'id')");
        assert!(result.contains("a.id"));
        assert!(result.contains("b.id"));
    }

    #[test]
    fn test_camel_keys() {
        let result = run_query(r#"{"hello_world": 1, "foo_bar": 2}"#, "camel_keys(@)");
        assert!(result.contains("helloWorld"));
        assert!(result.contains("fooBar"));
    }

    #[test]
    fn test_snake_keys() {
        let result = run_query(r#"{"helloWorld": 1, "fooBar": 2}"#, "snake_keys(@)");
        assert!(result.contains("hello_world"));
        assert!(result.contains("foo_bar"));
    }

    #[test]
    fn test_kebab_keys() {
        let result = run_query(r#"{"helloWorld": 1}"#, "kebab_keys(@)");
        assert!(result.contains("hello-world"));
    }

    #[test]
    fn test_leaves() {
        let result = run_query(r#"{"a": 1, "b": {"c": 2, "d": 3}}"#, "leaves(@)");
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_paths() {
        let result = run_query(r#"{"a": {"b": 1}}"#, "paths(@)");
        assert!(result.contains("/a"));
        assert!(result.contains("/a/b"));
    }
}

mod redaction {
    use super::*;

    #[test]
    fn test_mask_default() {
        let result = run_query(r#""4111111111111111""#, "mask(@)");
        assert!(result.contains("*"));
        assert!(result.contains("1111")); // shows last 4 by default
    }

    #[test]
    fn test_mask_custom_visible() {
        let result = run_query(r#""secret123""#, "mask(@, `3`)");
        // Should show last 3 characters
        assert!(result.contains("123"));
        assert!(result.contains("*"));
    }

    #[test]
    fn test_redact() {
        let result = run_query(
            r#"{"password": "secret", "name": "alice"}"#,
            "redact(@, `[\"password\"]`)",
        );
        assert!(result.contains("\"name\": \"alice\""));
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("secret"));
    }

    #[test]
    fn test_redact_keys_pattern() {
        let result = run_query(
            r#"{"password": "secret", "api_key": "xyz", "name": "alice"}"#,
            "redact_keys(@, 'password|api_key')",
        );
        assert!(result.contains("\"name\": \"alice\""));
        assert!(!result.contains("secret"));
        assert!(!result.contains("xyz"));
    }
}

mod math_stats {
    use super::*;

    #[test]
    fn test_quartiles() {
        let result = run_query(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#, "quartiles(@)");
        assert!(result.contains("q1"));
        assert!(result.contains("q2"));
        assert!(result.contains("q3"));
    }

    #[test]
    fn test_outliers_iqr() {
        let result = run_query(r#"[1, 2, 3, 4, 5, 100]"#, "outliers_iqr(@)");
        // 100 should be detected as an outlier
        assert!(result.contains("100"));
    }

    #[test]
    fn test_outliers_zscore() {
        let result = run_query(r#"[10, 10, 10, 10, 10, 100]"#, "outliers_zscore(@)");
        // 100 should be detected as an outlier
        assert!(result.contains("100"));
    }

    #[test]
    fn test_percentile() {
        let result = run_query(r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"#, "percentile(@, `50`)");
        // 50th percentile (median) should be around 5.5
        assert!(result.contains("5"));
    }

    #[test]
    fn test_median() {
        let result = run_query(r#"[1, 2, 3, 4, 5]"#, "median(@)");
        assert!(result.starts_with("3")); // Could be 3 or 3.0
    }

    #[test]
    fn test_stddev() {
        let result = run_query(r#"[2, 4, 4, 4, 5, 5, 7, 9]"#, "stddev(@)");
        assert!(result.contains("2")); // stddev is 2
    }

    #[test]
    fn test_variance() {
        let result = run_query(r#"[2, 4, 4, 4, 5, 5, 7, 9]"#, "variance(@)");
        assert!(result.contains("4")); // variance is 4
    }
}

mod string_functions {
    use super::*;

    #[test]
    fn test_upper() {
        assert_eq!(run_query(r#""hello""#, "upper(@)"), r#""HELLO""#);
    }

    #[test]
    fn test_lower() {
        assert_eq!(run_query(r#""HELLO""#, "lower(@)"), r#""hello""#);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(
            run_query(r#""hello world""#, "capitalize(@)"),
            r#""Hello world""#
        );
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            run_query(r#""hello world""#, "replace(@, 'world', 'there')"),
            r#""hello there""#
        );
    }

    #[test]
    fn test_pad_left() {
        let result = run_query(r#""42""#, "pad_left(@, `5`, '0')");
        assert_eq!(result, r#""00042""#);
    }

    #[test]
    fn test_pad_right() {
        let result = run_query(r#""hi""#, "pad_right(@, `5`, '.')");
        assert_eq!(result, r#""hi...""#);
    }

    #[test]
    fn test_repeat() {
        assert_eq!(run_query(r#""ab""#, "repeat(@, `3`)"), r#""ababab""#);
    }

    #[test]
    fn test_substr() {
        let result = run_query(r#""hello world""#, "substr(@, `0`, `5`)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_slice() {
        let result = run_query(r#""hello world""#, "slice(@, `0`, `5`)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_is_blank() {
        assert_eq!(run_query(r#""   ""#, "is_blank(@)"), "true");
        assert_eq!(run_query(r#""hello""#, "is_blank(@)"), "false");
    }

    #[test]
    fn test_ltrimstr() {
        assert_eq!(run_query(r#""foobar""#, "ltrimstr(@, 'foo')"), r#""bar""#);
    }

    #[test]
    fn test_rtrimstr() {
        assert_eq!(run_query(r#""foobar""#, "rtrimstr(@, 'bar')"), r#""foo""#);
    }
}

mod encoding_functions {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let result = run_query(r#""hello""#, "base64_encode(@)");
        assert_eq!(result, r#""aGVsbG8=""#);
    }

    #[test]
    fn test_base64_decode() {
        let result = run_query(r#""aGVsbG8=""#, "base64_decode(@)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_hex_encode() {
        let result = run_query(r#""hello""#, "hex_encode(@)");
        assert_eq!(result, r#""68656c6c6f""#);
    }

    #[test]
    fn test_hex_decode() {
        let result = run_query(r#""68656c6c6f""#, "hex_decode(@)");
        assert_eq!(result, r#""hello""#);
    }

    #[test]
    fn test_url_encode() {
        let result = run_query(r#""hello world""#, "url_encode(@)");
        assert!(result.contains("hello%20world") || result.contains("hello+world"));
    }

    #[test]
    fn test_url_decode() {
        let result = run_query(r#""hello%20world""#, "url_decode(@)");
        assert_eq!(result, r#""hello world""#);
    }
}

mod hash_functions {
    use super::*;

    #[test]
    fn test_md5() {
        let result = run_query(r#""hello""#, "md5(@)");
        assert_eq!(result, r#""5d41402abc4b2a76b9719d911017c592""#);
    }

    #[test]
    fn test_sha256() {
        let result = run_query(r#""hello""#, "sha256(@)");
        // SHA256 of "hello"
        assert!(result.len() > 60); // SHA256 hex is 64 chars + quotes
    }
}

mod datetime_functions {
    use super::*;

    #[test]
    fn test_now() {
        let result = run_query(r#"null"#, "type(now())");
        assert_eq!(result, r#""number""#);
    }

    #[test]
    fn test_format_date() {
        let result = run_query(r#"0"#, "format_date(@, '%Y')");
        assert_eq!(result, r#""1970""#);
    }

    #[test]
    fn test_date_add() {
        // Add 1 day (86400 seconds) to epoch
        let result = run_query(r#"0"#, "date_add(@, `1`, 'day')");
        assert!(result.starts_with("86400")); // Could be 86400 or 86400.0
    }
}

mod regex_functions {
    use super::*;

    #[test]
    fn test_regex_match() {
        assert_eq!(run_query(r#""hello123""#, "regex_match(@, '\\d+')"), "true");
        assert_eq!(run_query(r#""hello""#, "regex_match(@, '\\d+')"), "false");
    }

    #[test]
    fn test_regex_replace() {
        let result = run_query(r#""hello123world456""#, "regex_replace(@, '\\d+', 'X')");
        assert_eq!(result, r#""helloXworldX""#);
    }

    #[test]
    fn test_regex_extract() {
        let result = run_query(r#""hello123""#, "regex_extract(@, '(\\d+)')");
        assert!(result.contains("123"));
    }
}

mod validation_functions {
    use super::*;

    #[test]
    fn test_is_email() {
        assert_eq!(run_query(r#""test@example.com""#, "is_email(@)"), "true");
        assert_eq!(run_query(r#""not-an-email""#, "is_email(@)"), "false");
    }

    #[test]
    fn test_is_url() {
        assert_eq!(run_query(r#""https://example.com""#, "is_url(@)"), "true");
        assert_eq!(run_query(r#""not a url""#, "is_url(@)"), "false");
    }

    #[test]
    fn test_is_ipv4() {
        assert_eq!(run_query(r#""192.168.1.1""#, "is_ipv4(@)"), "true");
        assert_eq!(run_query(r#""999.999.999.999""#, "is_ipv4(@)"), "false");
    }

    #[test]
    fn test_is_uuid() {
        assert_eq!(
            run_query(r#""550e8400-e29b-41d4-a716-446655440000""#, "is_uuid(@)"),
            "true"
        );
        assert_eq!(run_query(r#""not-a-uuid""#, "is_uuid(@)"), "false");
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
