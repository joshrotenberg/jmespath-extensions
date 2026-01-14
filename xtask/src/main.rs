//! xtask for jmespath-extensions
//!
//! Usage:
//!   cargo xtask gen-docs    Generate mdbook documentation from functions.toml

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("gen-docs") => gen_docs(),
        Some(cmd) => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            std::process::exit(1);
        }
        None => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: cargo xtask <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  gen-docs    Generate mdbook documentation from functions.toml");
}

fn gen_docs() {
    let workspace_root = find_workspace_root();
    let toml_path = workspace_root.join("jmespath_extensions/functions.toml");
    let docs_dir = workspace_root.join("docs/book/src/functions");

    println!("Reading functions.toml...");
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

    // Sort functions within each category by name
    for funcs in by_category.values_mut() {
        funcs.sort_by(|a, b| a.name.cmp(&b.name));
    }

    println!(
        "Found {} categories with {} total functions",
        by_category.len(),
        data.functions.len()
    );

    // Ensure the functions directory exists
    fs::create_dir_all(&docs_dir).expect("Failed to create functions directory");

    // Generate per-category pages
    for (category, funcs) in &by_category {
        let filename = category_filename(category);
        let filepath = docs_dir.join(&filename);
        let content = generate_category_page(category, funcs);
        fs::write(&filepath, content).expect(&format!("Failed to write {}", filename));
        println!("  Generated {} ({} functions)", filename, funcs.len());
    }

    // Generate overview page with accurate counts
    let overview = generate_overview_page(&by_category);
    fs::write(docs_dir.join("overview.md"), overview).expect("Failed to write overview.md");
    println!("  Generated overview.md");

    // Generate SUMMARY.md entries
    println!();
    println!("Add the following to SUMMARY.md under '# Function Reference':");
    println!();
    println!("# Function Reference");
    println!();
    println!("- [Overview](./functions/overview.md)");

    // Group categories for SUMMARY
    let category_groups = get_category_groups();
    for (group_name, categories) in &category_groups {
        let group_funcs: Vec<_> = categories
            .iter()
            .filter_map(|c| by_category.get(*c))
            .flatten()
            .collect();
        if group_funcs.is_empty() {
            continue;
        }

        // If single category in group, just list it
        if categories.len() == 1 {
            let cat = categories[0];
            if let Some(funcs) = by_category.get(cat) {
                println!(
                    "- [{}](./functions/{}) ({} functions)",
                    group_name,
                    category_filename(cat),
                    funcs.len()
                );
            }
        } else {
            // Multiple categories grouped together
            let total: usize = categories
                .iter()
                .filter_map(|c| by_category.get(*c).map(|f| f.len()))
                .sum();
            println!(
                "- [{}](./functions/{}) ({} functions)",
                group_name,
                category_filename(categories[0]),
                total
            );
        }
    }

    println!();
    println!("Done!");
}

fn find_workspace_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().expect("Failed to get current directory");
    loop {
        if dir.join("Cargo.toml").exists() {
            let content = fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
            if content.contains("[workspace]") {
                return dir;
            }
        }
        if !dir.pop() {
            panic!("Could not find workspace root");
        }
    }
}

fn category_filename(category: &str) -> String {
    format!("{}.md", category.replace('-', "_"))
}

fn category_title(category: &str) -> String {
    match category {
        "standard" => "Standard JMESPath Functions".to_string(),
        "multimatch" => "Multi-Match Functions".to_string(),
        "jsonpatch" => "JSON Patch Functions".to_string(),
        "datetime" => "Date/Time Functions".to_string(),
        "uuid" => "UUID Functions".to_string(),
        "ids" => "ID Generation Functions".to_string(),
        "url" => "URL Functions".to_string(),
        "regex" => "Regular Expression Functions".to_string(),
        "semver" => "Semantic Versioning Functions".to_string(),
        "geo" => "Geolocation Functions".to_string(),
        "rand" => "Random Functions".to_string(),
        _ => {
            // Title case with "Functions" suffix
            let title: String = category
                .split(['_', '-'])
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("{} Functions", title)
        }
    }
}

fn generate_category_page(category: &str, funcs: &[&Function]) -> String {
    let mut doc = String::new();
    let title = category_title(category);

    doc.push_str(&format!("# {}\n\n", title));

    // Add category description
    let description = category_description(category);
    doc.push_str(&format!("{}\n\n", description));

    // Summary table
    doc.push_str("## Summary\n\n");
    doc.push_str("| Function | Signature | Description |\n");
    doc.push_str("|----------|-----------|-------------|\n");

    for func in funcs {
        let sig = func.signature.replace('|', "\\|");
        let desc = func.description.replace('|', "\\|");
        doc.push_str(&format!(
            "| [`{}`](#{}) | `{}` | {} |\n",
            func.name,
            func.name.replace('_', "-"),
            sig,
            desc
        ));
    }
    doc.push('\n');

    // Detailed function documentation
    doc.push_str("## Functions\n\n");

    for func in funcs {
        doc.push_str(&format!("### {}\n\n", func.name));
        doc.push_str(&format!("{}\n\n", func.description));
        doc.push_str(&format!("**Signature:** `{}`\n\n", func.signature));

        if let Some(jep) = &func.jep {
            doc.push_str(&format!("**JEP:** {}\n\n", jep));
        }

        if let Some(aliases) = &func.aliases {
            if !aliases.is_empty() {
                doc.push_str(&format!("**Aliases:** `{}`\n\n", aliases.join("`, `")));
            }
        }

        let examples = func.all_examples();
        if !examples.is_empty() {
            doc.push_str("**Examples:**\n\n");
            doc.push_str("```text\n");
            for ex in &examples {
                if let Some(ref desc) = ex.description {
                    doc.push_str(&format!("# {}\n", desc));
                }
                doc.push_str(&format!("{}\n", ex.code));
            }
            doc.push_str("```\n\n");
        }

        // Add usage example with jpx
        let first_example = examples.first().map(|e| &e.code);
        if let Some(ex) = first_example {
            if let Some(expr) = ex.split(" -> ").next() {
                doc.push_str("**CLI Usage:**\n\n");
                doc.push_str("```bash\n");
                // Try to construct a sensible example
                if expr.contains("(") {
                    doc.push_str(&format!("echo '{{}}' | jpx '{}'\n", expr));
                }
                doc.push_str("```\n\n");
            }
        }
    }

    doc
}

fn category_description(category: &str) -> &'static str {
    match category {
        "standard" => "These are the standard JMESPath functions as defined in the specification. They work in all JMESPath implementations.",
        "array" => "Functions for working with arrays: chunking, filtering, transforming, and combining array data.",
        "string" => "Functions for string manipulation: case conversion, splitting, joining, padding, and text processing.",
        "object" => "Functions for working with JSON objects: merging, filtering keys/values, and transformations.",
        "math" => "Mathematical and statistical functions: arithmetic, rounding, statistics, and number formatting.",
        "datetime" => "Functions for working with dates and times: parsing, formatting, arithmetic, and timezone handling.",
        "hash" => "Cryptographic hash functions: MD5, SHA family, and other hash algorithms.",
        "encoding" => "Encoding and decoding functions: Base64, hex, URL encoding, and more.",
        "validation" => "Functions for validating data: email, URL, UUID, and format validation.",
        "regex" => "Regular expression functions: matching, replacing, splitting, and pattern extraction.",
        "url" => "Functions for parsing and manipulating URLs and their components.",
        "semver" => "Semantic versioning functions: parsing, comparing, and manipulating version strings.",
        "jsonpatch" => "JSON Patch (RFC 6902) functions: applying patches, generating diffs, and path operations.",
        "multimatch" => "Functions for matching multiple patterns or expressions in a single operation.",
        "expression" => "Higher-order functions that work with JMESPath expressions as arguments.",
        "type" => "Type conversion and checking functions.",
        "uuid" => "Functions for generating and working with UUIDs.",
        "ids" => "Functions for generating various types of unique identifiers.",
        "rand" => "Functions for generating random values: numbers, strings, and selections.",
        "geo" => "Geolocation functions: distance calculation, coordinate parsing, and geographic utilities.",
        "fuzzy" => "Fuzzy matching and string similarity functions.",
        "phonetic" => "Phonetic encoding functions for sound-based string matching.",
        "text" => "Text analysis and processing functions.",
        "language" => "Natural language processing functions.",
        "network" => "Network-related functions: IP addresses, CIDR notation, and network utilities.",
        "path" => "File path manipulation functions.",
        "format" => "Data formatting functions for numbers, currencies, and other values.",
        "color" => "Color manipulation and conversion functions.",
        "computing" => "Computing-related utility functions.",
        "duration" => "Functions for working with time durations.",
        "utility" => "General utility functions that don't fit other categories.",
        _ => "Extension functions for jmespath-extensions.",
    }
}

fn generate_overview_page(by_category: &BTreeMap<String, Vec<&Function>>) -> String {
    let mut doc = String::new();
    let total: usize = by_category.values().map(|v| v.len()).sum();

    doc.push_str("# Function Overview\n\n");
    doc.push_str(&format!(
        "jpx provides {} functions organized into {} categories.\n\n",
        total,
        by_category.len()
    ));

    doc.push_str("## Discovering Functions\n\n");
    doc.push_str("### List All Functions\n\n");
    doc.push_str("```bash\n");
    doc.push_str("jpx --list-functions\n");
    doc.push_str("```\n\n");

    doc.push_str("### List by Category\n\n");
    doc.push_str("```bash\n");
    doc.push_str("jpx --list-category string\n");
    doc.push_str("jpx --list-category math\n");
    doc.push_str("jpx --list-category datetime\n");
    doc.push_str("```\n\n");

    doc.push_str("### Get Function Details\n\n");
    doc.push_str("```bash\n");
    doc.push_str("jpx --describe upper\n");
    doc.push_str("```\n\n");

    doc.push_str("## Categories\n\n");
    doc.push_str("| Category | Description | Count |\n");
    doc.push_str("|----------|-------------|-------|\n");

    for (category, funcs) in by_category {
        let title = category_title(category);
        let filename = category_filename(category);
        let desc = category_description(category);
        // Truncate description for table
        let short_desc: String = desc.chars().take(60).collect();
        let short_desc = if desc.len() > 60 {
            format!("{}...", short_desc)
        } else {
            short_desc
        };
        doc.push_str(&format!(
            "| [{}](./{}) | {} | {} |\n",
            title.trim_end_matches(" Functions"),
            filename,
            short_desc,
            funcs.len()
        ));
    }
    doc.push('\n');

    doc.push_str("## Function Syntax\n\n");
    doc.push_str("Functions are called with parentheses:\n\n");
    doc.push_str("```bash\n");
    doc.push_str("function_name(arg1, arg2, ...)\n");
    doc.push_str("```\n\n");

    doc.push_str("### Examples\n\n");
    doc.push_str("```bash\n");
    doc.push_str("# No arguments\n");
    doc.push_str("echo '{}' | jpx 'now()'\n\n");
    doc.push_str("# One argument\n");
    doc.push_str("echo '{\"name\": \"hello\"}' | jpx 'upper(name)'\n\n");
    doc.push_str("# Multiple arguments\n");
    doc.push_str("echo '{\"text\": \"hello world\"}' | jpx 'split(text, ` `)'\n\n");
    doc.push_str("# Literal arguments (use backticks)\n");
    doc.push_str("echo '{}' | jpx 'range(`1`, `10`)'\n");
    doc.push_str("```\n\n");

    doc.push_str("## Standard vs Extension Functions\n\n");

    if let Some(standard_funcs) = by_category.get("standard") {
        doc.push_str(&format!(
            "### Standard Functions ({})\n\n",
            standard_funcs.len()
        ));
        doc.push_str(
            "These are part of the JMESPath specification and work in all implementations:\n\n",
        );
        let names: Vec<_> = standard_funcs
            .iter()
            .map(|f| format!("`{}`", f.name))
            .collect();
        doc.push_str(&format!("{}\n\n", names.join(", ")));
    }

    let extension_count: usize = by_category
        .iter()
        .filter(|(cat, _)| *cat != "standard")
        .map(|(_, funcs)| funcs.len())
        .sum();

    doc.push_str(&format!(
        "### Extension Functions ({})\n\n",
        extension_count
    ));
    doc.push_str("These are jpx-specific and won't work in other JMESPath implementations.\n\n");

    doc.push_str("### Strict Mode\n\n");
    doc.push_str("Use `--strict` to disable extension functions:\n\n");
    doc.push_str("```bash\n");
    doc.push_str("# This works\n");
    doc.push_str("jpx --strict 'length(items)' -f data.json\n\n");
    doc.push_str("# This fails (upper is an extension)\n");
    doc.push_str("jpx --strict 'upper(name)' -f data.json\n");
    doc.push_str("```\n");

    doc
}

/// Get category groupings for SUMMARY.md organization
fn get_category_groups() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("Standard", vec!["standard"]),
        ("String", vec!["string"]),
        ("Array", vec!["array"]),
        ("Object", vec!["object"]),
        ("Math", vec!["math"]),
        ("Date/Time", vec!["datetime", "duration"]),
        ("Hash & Encoding", vec!["hash", "encoding"]),
        ("Validation", vec!["validation"]),
        ("Expression", vec!["expression"]),
        ("Type", vec!["type"]),
        ("Regex", vec!["regex"]),
        ("URL", vec!["url"]),
        ("Semver", vec!["semver"]),
        ("JSON Patch", vec!["jsonpatch"]),
        ("Multi-Match", vec!["multimatch"]),
        ("UUID & IDs", vec!["uuid", "ids"]),
        ("Random", vec!["rand"]),
        (
            "Text & Language",
            vec!["text", "language", "fuzzy", "phonetic"],
        ),
        ("Network & Geo", vec!["network", "geo"]),
        ("Format & Path", vec!["format", "path"]),
        ("Other", vec!["color", "computing", "utility"]),
    ]
}

#[derive(Debug, Deserialize)]
struct TomlData {
    functions: Vec<Function>,
}

#[derive(Debug, Deserialize)]
struct Function {
    name: String,
    category: String,
    description: String,
    signature: String,
    #[serde(default)]
    example: Option<String>,
    #[serde(default)]
    examples: Option<Vec<Example>>,
    #[serde(default)]
    #[allow(dead_code)]
    is_standard: Option<bool>,
    jep: Option<String>,
    aliases: Option<Vec<String>>,
    #[serde(default)]
    #[allow(dead_code)]
    features: Option<Vec<String>>,
}

impl Function {
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

#[derive(Debug, Clone, Deserialize)]
struct Example {
    code: String,
    #[serde(default)]
    description: Option<String>,
}
