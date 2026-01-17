//! Tests for the FunctionRegistry, including disable_function enforcement.
//!
//! These tests verify that:
//! - Functions can be disabled via the registry
//! - Disabled functions are actually unavailable at runtime (not just hidden from introspection)
//! - The registry correctly tracks enabled/disabled state
//!
//! Note: These tests require specific feature flags to be enabled.

#[cfg(feature = "string")]
use jmespath::{Runtime, Variable};
#[cfg(feature = "string")]
use jmespath_extensions::registry::{Category, FunctionRegistry};
#[cfg(feature = "string")]
use std::rc::Rc;

/// Create a runtime with specific registry configuration
#[cfg(feature = "string")]
fn create_runtime_with_registry(registry: &FunctionRegistry) -> Runtime {
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    registry.apply(&mut runtime);
    runtime
}

/// Test that disable_function actually prevents function execution at runtime.
/// This is the key test for issue #358.
#[test]
#[cfg(feature = "string")]
fn test_disable_function_prevents_execution() {
    // Create a registry with string functions, but disable "upper"
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);
    registry.disable_function("upper");

    let runtime = create_runtime_with_registry(&registry);

    // The "upper" function should NOT be available
    let expr = runtime.compile("upper('hello')");
    match expr {
        Ok(compiled) => {
            // If it compiled, execution should fail
            let result = compiled.search(Rc::new(Variable::Null));
            assert!(
                result.is_err(),
                "Expected 'upper' to fail at runtime when disabled, but it succeeded"
            );
        }
        Err(_) => {
            // Compilation failed because function doesn't exist - this is also acceptable
        }
    }

    // The "lower" function should still work (not disabled)
    let expr = runtime
        .compile("lower('HELLO')")
        .expect("lower should compile");
    let result = expr
        .search(Rc::new(Variable::Null))
        .expect("lower should execute");
    let result_str = result.as_string().expect("lower should return string");
    assert_eq!(result_str, "hello");
}

/// Test that multiple functions can be disabled
#[test]
#[cfg(feature = "string")]
fn test_disable_multiple_functions() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);
    registry.disable_function("upper");
    registry.disable_function("lower");
    registry.disable_function("trim");

    let runtime = create_runtime_with_registry(&registry);

    // All three should be unavailable
    for func_name in &["upper", "lower", "trim"] {
        let expr_str = format!("{}('test')", func_name);
        let expr = runtime.compile(&expr_str);
        match expr {
            Ok(compiled) => {
                let result = compiled.search(Rc::new(Variable::Null));
                assert!(
                    result.is_err(),
                    "Expected '{}' to fail at runtime when disabled",
                    func_name
                );
            }
            Err(_) => {
                // Compilation failed - also acceptable
            }
        }
    }

    // But "pad_left" should still work
    let expr = runtime
        .compile("pad_left('hi', `5`, ' ')")
        .expect("pad_left should compile");
    let result = expr
        .search(Rc::new(Variable::Null))
        .expect("pad_left should execute");
    let result_str = result.as_string().expect("pad_left should return string");
    assert_eq!(result_str, "   hi");
}

/// Test that enable_function re-enables a disabled function
#[test]
#[cfg(feature = "string")]
fn test_enable_function_restores_access() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);
    registry.disable_function("upper");

    // Verify it's disabled in introspection
    assert!(!registry.is_enabled("upper"));

    // Re-enable it
    registry.enable_function("upper");
    assert!(registry.is_enabled("upper"));

    // Now create runtime - upper should work
    let runtime = create_runtime_with_registry(&registry);
    let expr = runtime
        .compile("upper('hello')")
        .expect("upper should compile after re-enabling");
    let result = expr
        .search(Rc::new(Variable::Null))
        .expect("upper should execute after re-enabling");
    let result_str = result.as_string().expect("upper should return string");
    assert_eq!(result_str, "HELLO");
}

/// Test that is_enabled correctly reflects disabled state
#[test]
#[cfg(feature = "string")]
fn test_is_enabled_reflects_state() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);

    // Initially all should be enabled
    assert!(registry.is_enabled("upper"));
    assert!(registry.is_enabled("lower"));

    // Disable one
    registry.disable_function("upper");
    assert!(!registry.is_enabled("upper"));
    assert!(registry.is_enabled("lower"));

    // Re-enable
    registry.enable_function("upper");
    assert!(registry.is_enabled("upper"));
}

/// Test that get_function returns None for disabled functions
#[test]
#[cfg(feature = "string")]
fn test_get_function_returns_none_for_disabled() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);

    // Should return Some before disabling
    assert!(registry.get_function("upper").is_some());

    // Should return None after disabling
    registry.disable_function("upper");
    assert!(registry.get_function("upper").is_none());
}

/// Test that functions() iterator excludes disabled functions
#[test]
#[cfg(feature = "string")]
fn test_functions_iterator_excludes_disabled() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);

    let initial_count = registry.functions().count();

    registry.disable_function("upper");
    registry.disable_function("lower");

    let after_disable_count = registry.functions().count();
    assert_eq!(after_disable_count, initial_count - 2);

    // Verify "upper" and "lower" are not in the iterator
    let names: Vec<_> = registry.functions().map(|f| f.name).collect();
    assert!(!names.contains(&"upper"));
    assert!(!names.contains(&"lower"));
}

/// Test disabling functions across different categories
#[test]
#[cfg(all(feature = "string", feature = "math"))]
fn test_disable_across_categories() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);
    registry.register_category(Category::Math);

    // Disable one from each category
    registry.disable_function("upper"); // String
    registry.disable_function("add"); // Math

    let runtime = create_runtime_with_registry(&registry);

    // Both should be unavailable
    for (func_name, expr_str) in &[("upper", "upper('test')"), ("add", "add(`1`, `2`)")] {
        let expr = runtime.compile(expr_str);
        match expr {
            Ok(compiled) => {
                let result = compiled.search(Rc::new(Variable::Null));
                assert!(
                    result.is_err(),
                    "Expected '{}' to fail at runtime when disabled",
                    func_name
                );
            }
            Err(_) => {
                // Compilation failed - also acceptable
            }
        }
    }

    // Others should still work
    let expr = runtime
        .compile("lower('TEST')")
        .expect("lower should compile");
    let result = expr
        .search(Rc::new(Variable::Null))
        .expect("lower should execute");
    assert_eq!(result.as_string().unwrap(), "test");

    let expr = runtime
        .compile("subtract(`5`, `3`)")
        .expect("subtract should compile");
    let result = expr
        .search(Rc::new(Variable::Null))
        .expect("subtract should execute");
    assert_eq!(result.as_number().unwrap(), 2.0);
}

/// Test that len() correctly accounts for disabled functions
#[test]
#[cfg(feature = "string")]
fn test_len_accounts_for_disabled() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);

    let initial_len = registry.len();

    registry.disable_function("upper");
    registry.disable_function("lower");

    assert_eq!(registry.len(), initial_len - 2);
}

/// Test disabling a function that doesn't exist (should be a no-op for introspection,
/// but will still add to disabled set - which is fine since it won't affect anything)
#[test]
#[cfg(feature = "string")]
fn test_disable_nonexistent_function() {
    let mut registry = FunctionRegistry::new();
    registry.register_category(Category::String);

    let initial_count = registry.functions().count();

    // Disabling a non-existent function shouldn't panic
    // The disabled set will contain it, but len() subtracts disabled from registered,
    // so if the function isn't registered, it won't affect the count
    registry.disable_function("nonexistent_function_xyz");

    // The functions() iterator count should be unchanged since the function wasn't registered
    assert_eq!(registry.functions().count(), initial_count);
}
