//! Build script that generates documentation and registry code from functions.toml
//!
//! This ensures that:
//! - Rustdoc is always in sync with function metadata
//! - Registry code is generated from a single source of truth
//! - jpx introspection uses the same data

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

    // Generate the module documentation file
    generate_module_docs(&out_dir, &by_category);

    // Generate the registry data file
    generate_registry_data(&out_dir, &data.functions);

    // Generate per-category documentation
    generate_category_docs(&out_dir, &by_category);
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

            if let Some(aliases) = &func.aliases
                && !aliases.is_empty()
            {
                doc.push_str(&format!("**Aliases:** {}\n\n", aliases.join(", ")));
            }

            if !func.example.is_empty() {
                doc.push_str("**Example:**\n");
                doc.push_str("```text\n");
                doc.push_str(&format!("{}\n", func.example));
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
        // Remove backslash escapes from example since we use raw strings
        let example = func.example.replace("\\\"", "\"");
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
    example: String,
    #[serde(default)]
    is_standard: Option<bool>,
    jep: Option<String>,
    aliases: Option<Vec<String>>,
    features: Option<Vec<String>>,
}
