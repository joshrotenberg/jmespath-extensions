use anyhow::{Context, Result};
use clap::Parser;
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{Category, FunctionRegistry};
use std::io::{self, Read};

/// JMESPath CLI with extended functions
///
/// A command-line tool for querying JSON data using JMESPath expressions
/// with 150+ additional functions beyond the standard specification.
#[derive(Parser, Debug)]
#[command(name = "jpx", version, about, long_about = None)]
struct Args {
    /// JMESPath expression to evaluate
    expression: Option<String>,

    /// Input file (reads from stdin if not provided)
    #[arg(short, long)]
    file: Option<String>,

    /// Output raw strings without quotes
    #[arg(short = 'r', long)]
    raw: bool,

    /// Compact output (no pretty printing)
    #[arg(short, long)]
    compact: bool,

    /// List available extension functions
    #[arg(long)]
    list_functions: bool,

    /// List functions in a specific category
    #[arg(long, value_name = "CATEGORY")]
    list_category: Option<String>,

    /// Show detailed info for a specific function
    #[arg(long, value_name = "FUNCTION")]
    describe: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Create registry for introspection
    let mut registry = FunctionRegistry::new();
    registry.register_all();

    if args.list_functions {
        print_functions(&registry);
        return Ok(());
    }

    if let Some(category_name) = &args.list_category {
        print_category(&registry, category_name)?;
        return Ok(());
    }

    if let Some(func_name) = &args.describe {
        describe_function(&registry, func_name)?;
        return Ok(());
    }

    let expression = args
        .expression
        .ok_or_else(|| anyhow::anyhow!("Expression required. Use --help for usage."))?;

    // Read input JSON
    let input = match &args.file {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?,
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("Failed to read from stdin")?;
            buf
        }
    };

    // Parse JSON
    let data = Variable::from_json(&input)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON input: {}", e))?;

    // Create runtime with extensions
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);

    // Compile and execute expression
    let expr = runtime
        .compile(&expression)
        .with_context(|| format!("Failed to compile expression: {}", expression))?;

    let result = expr
        .search(&data)
        .context("Failed to evaluate expression")?;

    // Output result
    if result.is_null() {
        // Don't print anything for null results (like jq)
        return Ok(());
    }

    if args.raw {
        if let Some(s) = result.as_string() {
            println!("{}", s);
            return Ok(());
        }
    }

    let output = if args.compact {
        serde_json::to_string(&*result)?
    } else {
        serde_json::to_string_pretty(&*result)?
    };

    println!("{}", output);
    Ok(())
}

fn print_functions(registry: &FunctionRegistry) {
    println!("jpx - JMESPath with Extended Functions\n");

    // Count standard and extension functions
    let standard_count = registry.functions().filter(|f| f.is_standard).count();
    let extension_count = registry.functions().filter(|f| !f.is_standard).count();

    // Print standard functions
    let standard_funcs: Vec<_> = registry
        .functions_in_category(Category::Standard)
        .map(|f| f.name)
        .collect();
    println!("Standard JMESPath functions ({}):", standard_count);
    println!("  {}\n", standard_funcs.join(", "));

    println!("Extension functions ({} available):\n", extension_count);

    // Group by category (skip Standard)
    for category in Category::all() {
        if *category == Category::Standard || !category.is_available() {
            continue;
        }

        let funcs: Vec<_> = registry.functions_in_category(*category).collect();
        if funcs.is_empty() {
            continue;
        }

        let names: Vec<_> = funcs.iter().map(|f| f.name).collect();
        println!("{}: {}", category.name().to_uppercase(), names.join(", "));
        println!();
    }

    println!("Use --list-category <name> for details on a category");
    println!("Use --describe <function> for details on a specific function");
    println!("\nFor full documentation: https://docs.rs/jmespath_extensions");
}

fn print_category(registry: &FunctionRegistry, category_name: &str) -> Result<()> {
    let category = Category::all()
        .iter()
        .find(|c| c.name().eq_ignore_ascii_case(category_name))
        .ok_or_else(|| {
            let available: Vec<_> = Category::all()
                .iter()
                .filter(|c| c.is_available())
                .map(|c| c.name())
                .collect();
            anyhow::anyhow!(
                "Unknown category '{}'. Available: {}",
                category_name,
                available.join(", ")
            )
        })?;

    if !category.is_available() {
        return Err(anyhow::anyhow!(
            "Category '{}' is not available (not compiled in)",
            category_name
        ));
    }

    println!("{} functions:\n", category.name().to_uppercase());

    for func in registry.functions_in_category(*category) {
        println!("  {} - {}", func.name, func.description);
        println!("    Signature: {}", func.signature);
        println!("    Example: {}", func.example);
        println!();
    }

    Ok(())
}

fn describe_function(registry: &FunctionRegistry, func_name: &str) -> Result<()> {
    let func = registry.get_function(func_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown function '{}'. Use --list-functions to see available functions.",
            func_name
        )
    })?;

    println!("{}", func.name);
    println!("{}", "=".repeat(func.name.len()));
    println!();
    println!(
        "Type:        {}",
        if func.is_standard {
            "standard JMESPath"
        } else {
            "extension"
        }
    );
    println!("Category:    {}", func.category.name());
    if let Some(jep) = func.jep {
        println!("JEP:         {}", jep);
    }
    println!("Description: {}", func.description);
    println!("Signature:   {}", func.signature);
    println!();
    println!("Example:");
    println!("  {}", func.example);

    Ok(())
}
