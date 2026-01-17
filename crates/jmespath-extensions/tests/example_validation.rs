//! Runtime validation of function examples from functions.toml
//!
//! This test validates that example expressions have correct JMESPath syntax.
//! Note: Examples use shorthand syntax like `func([1, 2], \`3\`)` for documentation,
//! but JMESPath doesn't support inline array literals. We extract and validate
//! just the function call structure.

use jmespath::Runtime;
use jmespath_extensions::register_all;

// Include the generated example test data
include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));

/// Extract the function name from an expression
fn extract_function_name(expr: &str) -> Option<&str> {
    // Find the opening paren
    let paren_pos = expr.find('(')?;
    let name = expr[..paren_pos].trim();
    if name.is_empty() { None } else { Some(name) }
}

/// Test that all function names in examples are registered
#[test]
fn all_example_functions_registered() {
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);

    let mut missing_functions = Vec::new();

    for test in EXAMPLE_TESTS {
        if let Some(func_name) = extract_function_name(test.expression) {
            // Try to compile a simple expression using this function
            // Use @ as the argument to avoid literal parsing issues
            let simple_expr = format!("{}(@)", func_name);

            // We just want to check if the function is registered, not if
            // the arity is correct. A "too few arguments" error is fine.
            match runtime.compile(&simple_expr) {
                Ok(_) => {}
                Err(e) => {
                    let err_str = e.to_string();
                    // "Unknown function" means the function isn't registered
                    if err_str.contains("Unknown function") {
                        missing_functions.push(format!(
                            "Function '{}' used in example is not registered: {}",
                            func_name, test.expression
                        ));
                    }
                    // Other errors (like arity) are expected and fine
                }
            }
        }
    }

    if !missing_functions.is_empty() {
        panic!(
            "\n{} function(s) used in examples are not registered:\n\n{}",
            missing_functions.len(),
            missing_functions.join("\n")
        );
    }
}

/// Test that example count is reasonable (catches missing examples)
#[test]
fn has_sufficient_examples() {
    assert!(
        EXAMPLE_TESTS.len() > 100,
        "Expected at least 100 examples, found {}. \
         Are examples being generated correctly?",
        EXAMPLE_TESTS.len()
    );
}

/// Print summary of example coverage
#[test]
fn example_coverage_summary() {
    use std::collections::HashMap;

    let mut by_function: HashMap<&str, usize> = HashMap::new();
    for test in EXAMPLE_TESTS {
        *by_function.entry(test.function_name).or_insert(0) += 1;
    }

    let total_functions = by_function.len();
    let total_examples = EXAMPLE_TESTS.len();
    let with_multiple = by_function.values().filter(|&&c| c > 1).count();

    println!("\n=== Example Coverage Summary ===");
    println!("Total functions with examples: {}", total_functions);
    println!("Total examples: {}", total_examples);
    println!(
        "Functions with multiple examples: {} ({:.1}%)",
        with_multiple,
        (with_multiple as f64 / total_functions as f64) * 100.0
    );
    println!(
        "Average examples per function: {:.1}",
        total_examples as f64 / total_functions as f64
    );
}

/// Validate that all examples follow the expected format
#[test]
fn examples_have_valid_format() {
    let mut issues = Vec::new();

    for test in EXAMPLE_TESTS {
        // Check that expression is not empty
        if test.expression.trim().is_empty() {
            issues.push(format!(
                "Function '{}' has empty expression",
                test.function_name
            ));
            continue;
        }

        // Check that expected is not empty
        if test.expected.trim().is_empty() {
            issues.push(format!(
                "Function '{}' has empty expected value for: {}",
                test.function_name, test.expression
            ));
        }

        // Check for function call syntax (should have parentheses)
        if !test.expression.contains('(') || !test.expression.contains(')') {
            issues.push(format!(
                "Function '{}' example doesn't look like a function call: {}",
                test.function_name, test.expression
            ));
        }
    }

    if !issues.is_empty() {
        panic!(
            "\n{} format issue(s) found:\n\n{}",
            issues.len(),
            issues.join("\n")
        );
    }
}
