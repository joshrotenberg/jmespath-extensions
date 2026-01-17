//! JMESPath compliance tests with extensions registered.
//!
//! This module runs the official JMESPath compliance test suite to ensure
//! that registering extension functions doesn't break standard JMESPath behavior.
//!
//! The compliance tests validate:
//! - All 26 standard JMESPath functions work correctly
//! - Extension functions don't shadow or interfere with standard functions
//! - Grammar and parsing behavior matches the specification

use jmespath::{Rcvar, Runtime, Variable};
use serde::Deserialize;
use serde_json::Value;
use std::rc::Rc;

/// A single test case from the compliance suite
#[derive(Debug, Deserialize)]
struct TestCase {
    expression: String,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    bench: Option<String>,
}

/// A test suite containing multiple cases with shared input data
#[derive(Debug, Deserialize)]
struct TestSuite {
    given: Value,
    cases: Vec<TestCase>,
}

/// Compare two JSON values with fuzzy numeric comparison.
/// This handles the case where jmespath returns 1.0 but the test expects 1.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(na), Value::Number(nb)) => {
            // Compare as f64 to handle 1.0 == 1
            let fa = na.as_f64().unwrap_or(f64::NAN);
            let fb = nb.as_f64().unwrap_or(f64::NAN);
            (fa - fb).abs() < 1e-10
        }
        (Value::Array(aa), Value::Array(ab)) => {
            aa.len() == ab.len() && aa.iter().zip(ab.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(oa), Value::Object(ob)) => {
            oa.len() == ob.len()
                && oa
                    .iter()
                    .all(|(k, v)| ob.get(k).map_or(false, |v2| values_equal(v, v2)))
        }
        _ => a == b,
    }
}

/// Create a runtime with all extensions registered
fn create_runtime() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    jmespath_extensions::register_all(&mut runtime);
    runtime
}

/// Run a single test case
fn run_test_case(
    runtime: &Runtime,
    given: &Value,
    case: &TestCase,
    suite_name: &str,
    case_idx: usize,
) {
    // Skip benchmark cases
    if case.bench.is_some() {
        return;
    }

    let given_var = serde_json::from_value::<Variable>(given.clone())
        .expect("Failed to convert given data to Variable");
    let given_rc: Rcvar = Rc::new(given_var);

    let compile_result = runtime.compile(&case.expression);

    match (&case.error, &case.result) {
        // Expect an error
        (Some(error_type), None) => {
            match error_type.as_str() {
                "syntax" => {
                    assert!(
                        compile_result.is_err(),
                        "[{}:{}] Expected syntax error for '{}', but it parsed successfully",
                        suite_name,
                        case_idx,
                        case.expression
                    );
                }
                "invalid-type" | "invalid-value" | "invalid-arity" | "unknown-function" => {
                    // These are runtime errors, so the expression should parse
                    match compile_result {
                        Err(e) => {
                            // Some implementations catch these at parse time
                            // Just verify we got an error
                            let _ = e;
                        }
                        Ok(expr) => {
                            let search_result = expr.search(given_rc);
                            assert!(
                                search_result.is_err(),
                                "[{}:{}] Expected {} error for '{}', but got result: {:?}",
                                suite_name,
                                case_idx,
                                error_type,
                                case.expression,
                                search_result
                            );
                        }
                    }
                }
                other => {
                    panic!(
                        "[{}:{}] Unknown error type: {}",
                        suite_name, case_idx, other
                    );
                }
            }
        }
        // Expect a valid result
        (None, Some(expected)) => {
            let expr = compile_result.unwrap_or_else(|e| {
                panic!(
                    "[{}:{}] Failed to compile '{}': {}",
                    suite_name, case_idx, case.expression, e
                )
            });

            let result = expr.search(given_rc).unwrap_or_else(|e| {
                panic!(
                    "[{}:{}] Failed to evaluate '{}': {}",
                    suite_name, case_idx, case.expression, e
                )
            });

            // Convert result back to serde_json::Value for comparison
            let result_json: Value =
                serde_json::to_value(&*result).expect("Failed to convert result to JSON");

            // Use fuzzy numeric comparison (1.0 == 1)
            assert!(
                values_equal(&result_json, expected),
                "[{}:{}] Expression '{}' returned {:?}, expected {:?}",
                suite_name,
                case_idx,
                case.expression,
                result_json,
                expected
            );
        }
        // Invalid test case
        (Some(_), Some(_)) => {
            panic!(
                "[{}:{}] Test case has both error and result",
                suite_name, case_idx
            );
        }
        (None, None) => {
            // No assertion - might be a comment or placeholder
        }
    }
}

/// Run all test cases in a suite
fn run_suite(json_content: &str, suite_name: &str) {
    let runtime = create_runtime();
    let suites: Vec<TestSuite> = serde_json::from_str(json_content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", suite_name, e));

    for (suite_idx, suite) in suites.iter().enumerate() {
        for (case_idx, case) in suite.cases.iter().enumerate() {
            run_test_case(
                &runtime,
                &suite.given,
                case,
                &format!("{}[{}]", suite_name, suite_idx),
                case_idx,
            );
        }
    }
}

// Include compliance test files at compile time
const BASIC_JSON: &str = include_str!("compliance/basic.json");
const BOOLEAN_JSON: &str = include_str!("compliance/boolean.json");
const CURRENT_JSON: &str = include_str!("compliance/current.json");
const ESCAPE_JSON: &str = include_str!("compliance/escape.json");
const FILTERS_JSON: &str = include_str!("compliance/filters.json");
const FUNCTIONS_JSON: &str = include_str!("compliance/functions.json");
const IDENTIFIERS_JSON: &str = include_str!("compliance/identifiers.json");
const INDICES_JSON: &str = include_str!("compliance/indices.json");
const LITERAL_JSON: &str = include_str!("compliance/literal.json");
const MULTISELECT_JSON: &str = include_str!("compliance/multiselect.json");
const PIPE_JSON: &str = include_str!("compliance/pipe.json");
const SLICE_JSON: &str = include_str!("compliance/slice.json");
const SYNTAX_JSON: &str = include_str!("compliance/syntax.json");
const UNICODE_JSON: &str = include_str!("compliance/unicode.json");
const WILDCARD_JSON: &str = include_str!("compliance/wildcard.json");

#[test]
fn compliance_basic() {
    run_suite(BASIC_JSON, "basic");
}

#[test]
fn compliance_boolean() {
    run_suite(BOOLEAN_JSON, "boolean");
}

#[test]
fn compliance_current() {
    run_suite(CURRENT_JSON, "current");
}

#[test]
fn compliance_escape() {
    run_suite(ESCAPE_JSON, "escape");
}

#[test]
fn compliance_filters() {
    run_suite(FILTERS_JSON, "filters");
}

#[test]
fn compliance_functions() {
    run_suite(FUNCTIONS_JSON, "functions");
}

#[test]
fn compliance_identifiers() {
    run_suite(IDENTIFIERS_JSON, "identifiers");
}

#[test]
fn compliance_indices() {
    run_suite(INDICES_JSON, "indices");
}

#[test]
fn compliance_literal() {
    run_suite(LITERAL_JSON, "literal");
}

#[test]
fn compliance_multiselect() {
    run_suite(MULTISELECT_JSON, "multiselect");
}

#[test]
fn compliance_pipe() {
    run_suite(PIPE_JSON, "pipe");
}

#[test]
fn compliance_slice() {
    run_suite(SLICE_JSON, "slice");
}

#[test]
fn compliance_syntax() {
    run_suite(SYNTAX_JSON, "syntax");
}

#[test]
fn compliance_unicode() {
    run_suite(UNICODE_JSON, "unicode");
}

#[test]
fn compliance_wildcard() {
    run_suite(WILDCARD_JSON, "wildcard");
}

/// Test that standard functions are not shadowed by extensions
#[test]
fn standard_functions_not_shadowed() {
    let runtime = create_runtime();

    // Test each of the 26 standard JMESPath functions
    let test_cases = [
        // abs
        (r#"abs(`-5`)"#, "5"),
        // avg
        (r#"avg(`[1, 2, 3, 4, 5]`)"#, "3.0"),
        // ceil
        (r#"ceil(`1.5`)"#, "2"),
        // contains
        (r#"contains(`["a", "b"]`, `"a"`)"#, "true"),
        // ends_with
        (r#"ends_with(`"hello"`, `"lo"`)"#, "true"),
        // floor
        (r#"floor(`1.9`)"#, "1"),
        // join
        (r#"join(`","`, `["a", "b"]`)"#, r#""a,b""#),
        // keys
        (r#"keys(`{"a": 1, "b": 2}`)"#, r#"["a","b"]"#),
        // length
        (r#"length(`[1, 2, 3]`)"#, "3"),
        // map - requires expression reference, skip for simplicity
        // max
        (r#"max(`[1, 2, 3]`)"#, "3"),
        // max_by - requires expression reference, skip
        // merge
        (r#"merge(`{"a": 1}`, `{"b": 2}`)"#, r#"{"a":1,"b":2}"#),
        // min
        (r#"min(`[1, 2, 3]`)"#, "1"),
        // min_by - requires expression reference, skip
        // not_null
        (r#"not_null(`null`, `1`, `2`)"#, "1"),
        // reverse
        (r#"reverse(`[1, 2, 3]`)"#, "[3,2,1]"),
        // sort
        (r#"sort(`[3, 1, 2]`)"#, "[1,2,3]"),
        // sort_by - requires expression reference, skip
        // starts_with
        (r#"starts_with(`"hello"`, `"he"`)"#, "true"),
        // sum
        (r#"sum(`[1, 2, 3]`)"#, "6"),
        // to_array
        (r#"to_array(`"hello"`)"#, r#"["hello"]"#),
        // to_number
        (r#"to_number(`"42"`)"#, "42"),
        // to_string
        (r#"to_string(`42`)"#, r#""42""#),
        // type
        (r#"type(`"hello"`)"#, r#""string""#),
        // values
        (r#"values(`{"a": 1, "b": 2}`)"#, "[1,2]"),
    ];

    for (expr_str, expected_str) in test_cases {
        let expr = runtime
            .compile(expr_str)
            .unwrap_or_else(|e| panic!("Failed to compile '{}': {}", expr_str, e));

        let result = expr
            .search(Rc::new(Variable::Null))
            .unwrap_or_else(|e| panic!("Failed to evaluate '{}': {}", expr_str, e));

        let result_json: Value = serde_json::to_value(&*result).unwrap();
        let expected_json: Value = serde_json::from_str(expected_str).unwrap();

        assert!(
            values_equal(&result_json, &expected_json),
            "Standard function test failed for '{}': got {:?}, expected {:?}",
            expr_str,
            result_json,
            expected_json
        );
    }
}
