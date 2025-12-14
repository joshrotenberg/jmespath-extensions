//! Build script that generates documentation and registry code from functions.toml
//!
//! This ensures that:
//! - Rustdoc is always in sync with function metadata
//! - Registry code is generated from a single source of truth
//! - jpx introspection uses the same data

// Allow nested if for compatibility with older Rust versions that don't support let chains
#![allow(clippy::collapsible_if)]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=functions.toml");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&manifest_dir).join("functions.toml");
    let out_dir = env::var("OUT_DIR").unwrap();

    // Read and parse the TOML file
    let toml_content = fs::read_to_string(&toml_path).expect("Failed to read functions.toml");
    let data: TomlData = toml::from_str(&toml_content).expect("Failed to parse functions.toml");

    // Group functions by category
    let mut by_category: BTreeMap<String, Vec<&Function>> = BTreeMap::new();
    for func in &data.functions {
        by_category
            .entry(func.category.clone())
            .or_default()
            .push(func);
    }

    // Validate all examples have correct format
    validate_examples(&data.functions);

    // Generate the module documentation file
    generate_module_docs(&out_dir, &by_category);

    // Generate the registry data file
    generate_registry_data(&out_dir, &data.functions);

    // Generate per-category documentation
    generate_category_docs(&out_dir, &by_category);

    // Generate quick reference table for crate root
    generate_quick_reference(&out_dir, &by_category);

    // Generate test data for runtime example validation
    generate_example_test_data(&out_dir, &data.functions);
}

fn generate_module_docs(out_dir: &str, by_category: &BTreeMap<String, Vec<&Function>>) {
    let mut doc = String::new();

    // Generate pure markdown for #[doc = include_str!(...)]
    doc.push_str("# Complete Function Reference\n\n");
    doc.push_str("This documentation is auto-generated from `functions.toml`.\n\n");

    // Table of contents
    doc.push_str("## Categories\n\n");
    for (category, funcs) in by_category {
        let count = funcs.len();
        let cat_title = category_title(category);
        doc.push_str(&format!(
            "- [{}](#{}) ({} functions)\n",
            cat_title,
            category.to_lowercase().replace('-', "_"),
            count
        ));
    }
    doc.push('\n');

    // Each category section
    for (category, funcs) in by_category {
        let cat_title = category_title(category);
        doc.push_str(&format!("## {}\n\n", cat_title));
        doc.push_str("| Function | Signature | Description |\n");
        doc.push_str("|----------|-----------|-------------|\n");

        for func in funcs {
            let sig = func.signature.replace('|', "\\|");
            let desc = func
                .description
                .replace('|', "\\|")
                .replace('[', r"\[")
                .replace(']', r"\]");
            doc.push_str(&format!("| `{}` | `{}` | {} |\n", func.name, sig, desc));
        }
        doc.push('\n');

        // Detailed function docs
        for func in funcs {
            doc.push_str(&format!("### `{}`\n\n", func.name));
            // Escape brackets in description to avoid rustdoc link warnings
            let desc = func.description.replace('[', r"\[").replace(']', r"\]");
            doc.push_str(&format!("{}\n\n", desc));
            doc.push_str(&format!("**Signature:** `{}`\n\n", func.signature));

            if let Some(jep) = &func.jep {
                doc.push_str(&format!("**JEP:** {}\n\n", jep));
            }

            if let Some(aliases) = &func.aliases {
                if !aliases.is_empty() {
                    doc.push_str(&format!("**Aliases:** {}\n\n", aliases.join(", ")));
                }
            }

            let examples = func.all_examples();
            if !examples.is_empty() {
                if examples.len() == 1 {
                    doc.push_str("**Example:**\n");
                } else {
                    doc.push_str("**Examples:**\n");
                }
                doc.push_str("```text\n");
                for ex in &examples {
                    if let Some(ref desc) = ex.description {
                        doc.push_str(&format!("// {}\n", desc));
                    }
                    doc.push_str(&format!("{}\n", ex.code));
                }
                doc.push_str("```\n\n");
            }
        }
    }

    let doc_path = Path::new(out_dir).join("function_docs.md");
    fs::write(doc_path, doc).expect("Failed to write function_docs.md");
}

fn generate_registry_data(out_dir: &str, functions: &[Function]) {
    let mut code = String::new();

    code.push_str("// Auto-generated from functions.toml - DO NOT EDIT\n\n");
    code.push_str("use super::{Category, Feature, FunctionInfo};\n\n");
    code.push_str("pub const FUNCTIONS: &[FunctionInfo] = &[\n");

    for func in functions {
        code.push_str("    FunctionInfo {\n");
        code.push_str(&format!("        name: \"{}\",\n", func.name));
        code.push_str(&format!(
            "        category: Category::{},\n",
            category_variant(&func.category)
        ));
        code.push_str(&format!(
            "        description: r##\"{}\"##,\n",
            func.description
        ));
        code.push_str(&format!(
            "        signature: r##\"{}\"##,\n",
            func.signature
        ));
        // Get the first example for the registry (backward compatibility)
        let examples = func.all_examples();
        let example = examples
            .first()
            .map(|e| e.code.replace("\\\"", "\""))
            .unwrap_or_default();
        code.push_str(&format!("        example: r##\"{}\"##,\n", example));
        code.push_str(&format!(
            "        is_standard: {},\n",
            func.is_standard.unwrap_or(false)
        ));

        match &func.jep {
            Some(jep) => code.push_str(&format!("        jep: Some(\"{}\"),\n", jep)),
            None => code.push_str("        jep: None,\n"),
        }

        match &func.aliases {
            Some(aliases) if !aliases.is_empty() => {
                let aliases_str: Vec<String> =
                    aliases.iter().map(|a| format!("\"{}\"", a)).collect();
                code.push_str(&format!(
                    "        aliases: &[{}],\n",
                    aliases_str.join(", ")
                ));
            }
            _ => code.push_str("        aliases: &[],\n"),
        }

        match &func.features {
            Some(features) if !features.is_empty() => {
                let features_str: Vec<String> = features
                    .iter()
                    .map(|f| format!("Feature::{}", feature_variant(f)))
                    .collect();
                code.push_str(&format!(
                    "        features: &[{}],\n",
                    features_str.join(", ")
                ));
            }
            _ => code.push_str("        features: &[],\n"),
        }

        code.push_str("    },\n");
    }

    code.push_str("];\n");

    let data_path = Path::new(out_dir).join("registry_data.rs");
    fs::write(data_path, code).expect("Failed to write registry_data.rs");
}

fn generate_category_docs(out_dir: &str, by_category: &BTreeMap<String, Vec<&Function>>) {
    for (category, funcs) in by_category {
        let mut doc = String::new();

        doc.push_str(&format!("//! # {} Functions\n", category_title(category)));
        doc.push_str("//!\n");
        doc.push_str("//! | Function | Signature | Description |\n");
        doc.push_str("//! |----------|-----------|-------------|\n");

        for func in funcs {
            let sig = func.signature.replace('|', "\\|");
            let desc = func.description.replace('|', "\\|");
            doc.push_str(&format!(
                "//! | [`{}`](#{}) | `{}` | {} |\n",
                func.name,
                func.name.replace('_', "-"),
                sig,
                desc
            ));
        }
        doc.push_str("//!\n");

        let filename = format!("{}_docs.rs", category.replace('-', "_"));
        let doc_path = Path::new(out_dir).join(filename);
        fs::write(doc_path, doc).expect("Failed to write category docs");
    }
}

fn generate_quick_reference(out_dir: &str, by_category: &BTreeMap<String, Vec<&Function>>) {
    let mut doc = String::new();

    // Count total functions
    let total: usize = by_category.values().map(|v| v.len()).sum();

    doc.push_str(&format!("## Quick Reference ({} functions)\n\n", total));
    doc.push_str("Click any function name to jump to its detailed documentation in the [`functions`] module.\n\n");

    // Generate a table for each category
    for (category, funcs) in by_category {
        let cat_title = category_title(category);
        let module_link = category_to_module(category);

        doc.push_str(&format!("### {} ([`{}`])\n\n", cat_title, module_link));
        doc.push_str("| Function | Signature | Description |\n");
        doc.push_str("|----------|-----------|-------------|\n");

        for func in funcs {
            let sig = func.signature.replace('|', "\\|");
            let desc = func
                .description
                .replace('|', "\\|")
                .replace('[', r"\[")
                .replace(']', r"\]");
            // Link to the function in the functions module
            doc.push_str(&format!(
                "| [`{}`](functions#{}---{}) | `{}` | {} |\n",
                func.name,
                category.to_lowercase().replace('-', "_"),
                func.name,
                sig,
                desc
            ));
        }
        doc.push('\n');
    }

    let doc_path = Path::new(out_dir).join("quick_reference.md");
    fs::write(doc_path, doc).expect("Failed to write quick_reference.md");
}

fn category_to_module(category: &str) -> String {
    match category {
        "standard" => "functions".to_string(),
        "multi-match" | "multimatch" => "mod@multi_match".to_string(),
        "jsonpatch" => "mod@jsonpatch".to_string(),
        "datetime" => "mod@datetime".to_string(),
        "regex" => "mod@regex_fns".to_string(),
        "url" => "mod@url_fns".to_string(),
        "semver" => "mod@semver_fns".to_string(),
        "type" => "mod@type_conv".to_string(),
        // Use mod@ prefix to disambiguate from primitive types
        "array" => "mod@array".to_string(),
        "string" => "mod@string".to_string(),
        _ => format!("mod@{}", category.replace('-', "_")),
    }
}

fn category_title(category: &str) -> String {
    match category {
        "standard" => "Standard JMESPath".to_string(),
        "multi-match" => "Multi-Match".to_string(),
        "jsonpatch" => "JSON Patch".to_string(),
        _ => {
            // Title case
            let mut chars = category.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

fn category_variant(category: &str) -> String {
    match category {
        "standard" => "Standard".to_string(),
        "multi-match" | "multimatch" => "MultiMatch".to_string(),
        "jsonpatch" => "Jsonpatch".to_string(),
        _ => {
            // PascalCase
            category
                .split(['_', '-'])
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect()
        }
    }
}

fn feature_variant(feature: &str) -> String {
    match feature {
        "spec" => "Spec".to_string(),
        "core" => "Core".to_string(),
        "fp" => "Fp".to_string(),
        "jep" => "Jep".to_string(),
        _ => feature.to_string(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct TomlData {
    functions: Vec<Function>,
}

#[derive(Debug, serde::Deserialize)]
struct Function {
    name: String,
    category: String,
    description: String,
    signature: String,
    /// Single example (backward compatibility)
    #[serde(default)]
    example: Option<String>,
    /// Multiple examples with descriptions
    #[serde(default)]
    examples: Option<Vec<Example>>,
    #[serde(default)]
    is_standard: Option<bool>,
    jep: Option<String>,
    aliases: Option<Vec<String>>,
    features: Option<Vec<String>>,
}

impl Function {
    /// Get all examples, combining single example and examples array
    fn all_examples(&self) -> Vec<Example> {
        let mut result = Vec::new();
        if let Some(ref ex) = self.example {
            result.push(Example {
                code: ex.clone(),
                description: None,
            });
        }
        if let Some(ref exs) = self.examples {
            result.extend(exs.iter().cloned());
        }
        result
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Example {
    code: String,
    #[serde(default)]
    description: Option<String>,
}

/// Validate that all examples follow the expected format
fn validate_examples(functions: &[Function]) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for func in functions {
        let examples = func.all_examples();

        if examples.is_empty() {
            warnings.push(format!("Function '{}' has no examples", func.name));
            continue;
        }

        for (i, ex) in examples.iter().enumerate() {
            // Check that example contains " -> " separator
            if !ex.code.contains(" -> ") {
                errors.push(format!(
                    "Function '{}' example {} missing ' -> ' separator: {}",
                    func.name,
                    i + 1,
                    ex.code
                ));
                continue;
            }

            // Extract the expression part (before " -> ")
            let expression = ex.code.split(" -> ").next().unwrap_or("");

            // Check that the function name appears in the expression
            // (but not for aliases or special cases)
            if !expression.contains(&func.name) {
                // Check aliases too
                let has_alias = func.aliases.as_ref().map_or(false, |aliases| {
                    aliases.iter().any(|a| expression.contains(a))
                });

                if !has_alias {
                    warnings.push(format!(
                        "Function '{}' example {} may not use the function: {}",
                        func.name,
                        i + 1,
                        ex.code
                    ));
                }
            }

            // Check for balanced parentheses in expression
            let open_parens = expression.matches('(').count();
            let close_parens = expression.matches(')').count();
            if open_parens != close_parens {
                errors.push(format!(
                    "Function '{}' example {} has unbalanced parentheses: {}",
                    func.name,
                    i + 1,
                    expression
                ));
            }

            // Check for balanced brackets
            let open_brackets = expression.matches('[').count();
            let close_brackets = expression.matches(']').count();
            if open_brackets != close_brackets {
                errors.push(format!(
                    "Function '{}' example {} has unbalanced brackets: {}",
                    func.name,
                    i + 1,
                    expression
                ));
            }
        }
    }

    // Print warnings (don't fail build)
    for warning in &warnings {
        println!("cargo:warning={}", warning);
    }

    // Fail build on errors
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("ERROR: {}", error);
        }
        panic!(
            "Example validation failed with {} error(s). See messages above.",
            errors.len()
        );
    }
}

/// Generate test data for runtime example validation
fn generate_example_test_data(out_dir: &str, functions: &[Function]) {
    let mut code = String::new();

    code.push_str("// Auto-generated from functions.toml - DO NOT EDIT\n");
    code.push_str("// This file contains example expressions for runtime validation\n\n");
    code.push_str("pub struct ExampleTest {\n");
    code.push_str("    pub function_name: &'static str,\n");
    code.push_str("    pub expression: &'static str,\n");
    code.push_str("    pub expected: &'static str,\n");
    code.push_str("    pub description: Option<&'static str>,\n");
    code.push_str("}\n\n");
    code.push_str("pub const EXAMPLE_TESTS: &[ExampleTest] = &[\n");

    for func in functions {
        for ex in func.all_examples() {
            // Parse "expression -> expected" format
            let parts: Vec<&str> = ex.code.splitn(2, " -> ").collect();
            if parts.len() != 2 {
                continue; // Skip malformed examples (caught by validation)
            }

            let expression = parts[0].trim();
            let expected = parts[1].trim();
            let description = ex.description.as_deref();

            code.push_str("    ExampleTest {\n");
            // Use {:?} for proper escaping of special characters
            code.push_str(&format!("        function_name: {:?},\n", func.name));
            code.push_str(&format!("        expression: {:?},\n", expression));
            code.push_str(&format!("        expected: {:?},\n", expected));
            match description {
                Some(desc) => code.push_str(&format!("        description: Some({:?}),\n", desc)),
                None => code.push_str("        description: None,\n"),
            }
            code.push_str("    },\n");
        }
    }

    code.push_str("];\n");

    let test_data_path = Path::new(out_dir).join("example_tests.rs");
    fs::write(test_data_path, code).expect("Failed to write example_tests.rs");
}
