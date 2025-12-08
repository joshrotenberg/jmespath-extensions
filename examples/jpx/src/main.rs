use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{Shell, generate};
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{Category, FunctionRegistry};
use std::fs::File;
use std::io::{self, Read, Write};
use std::rc::Rc;
use std::time::Instant;

/// Color output mode
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ColorMode {
    /// Automatically detect if output is a terminal
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// JMESPath CLI with extended functions
///
/// A command-line tool for querying JSON data using JMESPath expressions
/// with 150+ additional functions beyond the standard specification.
#[derive(Parser, Debug)]
#[command(name = "jpx", version, about, long_about = None)]
struct Args {
    /// JMESPath expression(s) to evaluate (multiple expressions are chained)
    #[arg(short = 'e', long = "expression", conflicts_with = "query_file")]
    expressions: Vec<String>,

    /// JMESPath expression as positional argument
    #[arg(conflicts_with_all = ["query_file", "expressions"])]
    expression: Option<String>,

    /// Read JMESPath expression from file
    #[arg(short = 'Q', long = "query-file", conflicts_with_all = ["expression", "expressions"])]
    query_file: Option<String>,

    /// Input file (reads from stdin if not provided)
    #[arg(short, long)]
    file: Option<String>,

    /// Output raw strings without quotes
    #[arg(short = 'r', long)]
    raw: bool,

    /// Compact output (no pretty printing)
    #[arg(short, long)]
    compact: bool,

    /// Null input - don't read input, use null as the input value
    #[arg(short = 'n', long)]
    null_input: bool,

    /// Slurp - read all inputs into an array
    #[arg(short = 's', long)]
    slurp: bool,

    /// Colorize output (auto, always, never)
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,

    /// Output file (writes to stdout if not provided)
    #[arg(short = 'o', long)]
    output: Option<String>,

    /// Quiet mode - suppress errors and warnings
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Verbose mode - show expression details and timing
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL")]
    completions: Option<Shell>,

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

    // Handle shell completions
    if let Some(shell) = args.completions {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return Ok(());
    }

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

    // Get expressions from positional arg, -e flags, or file
    let expressions: Vec<String> = if let Some(query_path) = &args.query_file {
        vec![
            std::fs::read_to_string(query_path)
                .with_context(|| format!("Failed to read query file: {}", query_path))?
                .trim()
                .to_string(),
        ]
    } else if !args.expressions.is_empty() {
        args.expressions.clone()
    } else if let Some(expr) = &args.expression {
        vec![expr.clone()]
    } else {
        return Err(anyhow::anyhow!(
            "Expression required. Use --help for usage."
        ));
    };

    // Get input data
    let data = if args.null_input {
        // Null input mode - don't read anything
        Variable::Null
    } else {
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

        if args.slurp {
            // Slurp mode - parse multiple JSON values into an array
            parse_slurp(&input)?
        } else {
            // Normal mode - parse single JSON value
            Variable::from_json(&input)
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON input: {}", e))?
        }
    };

    // Create runtime with extensions
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);

    // Verbose mode: show input info
    if args.verbose {
        eprintln!("Input: {}", describe_value(&Rc::new(data.clone())));
        if expressions.len() > 1 {
            eprintln!("Expressions: {} (chained)", expressions.len());
        }
        eprintln!();
    }

    // Compile and execute expression(s)
    let start = Instant::now();
    let mut result: Rc<Variable> = Rc::new(data.clone());

    for (i, expression) in expressions.iter().enumerate() {
        if args.verbose {
            eprintln!("[{}] Expression: {}", i + 1, expression);
        }

        let expr = runtime
            .compile(expression)
            .with_context(|| format!("Failed to compile expression: {}", expression))?;

        let step_start = Instant::now();
        result = expr
            .search(&result)
            .context("Failed to evaluate expression")?;
        let step_elapsed = step_start.elapsed();

        if args.verbose {
            eprintln!("[{}] Result: {}", i + 1, describe_value(&result));
            eprintln!(
                "[{}] Time: {:.3}ms",
                i + 1,
                step_elapsed.as_secs_f64() * 1000.0
            );
            eprintln!();
        }
    }

    let total_elapsed = start.elapsed();
    if args.verbose {
        eprintln!("Total time: {:.3}ms", total_elapsed.as_secs_f64() * 1000.0);
        eprintln!();
    }

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

    // Convert to serde_json::Value for output formatting
    let json_value: serde_json::Value = serde_json::from_str(&serde_json::to_string(&*result)?)?;

    // When writing to file, don't colorize unless explicitly requested
    let should_colorize = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => args.output.is_none() && atty::is(atty::Stream::Stdout),
    };

    let output = if should_colorize && !args.compact {
        // Colored pretty output with custom color scheme
        use colored_json::{ColoredFormatter, PrettyFormatter, Style, Styler};

        let styler = Styler {
            key: Style::new().blue().bold(),
            string_value: Style::new().green(),
            integer_value: Style::new().cyan(),
            float_value: Style::new().cyan(),
            bool_value: Style::new().yellow(),
            nil_value: Style::new().red().dim(),
            ..Default::default()
        };

        let formatter = ColoredFormatter::with_styler(PrettyFormatter::new(), styler);
        let mut writer = Vec::new();
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        use serde::Serialize;
        json_value.serialize(&mut serializer)?;
        String::from_utf8(writer)?
    } else if args.compact {
        serde_json::to_string(&json_value)?
    } else {
        serde_json::to_string_pretty(&json_value)?
    };

    // Write output to file or stdout
    if let Some(output_path) = &args.output {
        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path))?;
        writeln!(file, "{}", output)
            .with_context(|| format!("Failed to write to output file: {}", output_path))?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Parse multiple JSON values from input into an array
fn parse_slurp(input: &str) -> Result<Variable> {
    use serde_json::Deserializer;

    let mut values = Vec::new();
    let stream = Deserializer::from_str(input).into_iter::<serde_json::Value>();

    for result in stream {
        let value = result.context("Failed to parse JSON in slurp mode")?;
        let var = Variable::from_json(&value.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to convert JSON value: {}", e))?;
        values.push(var);
    }

    // Convert Vec<Variable> to a Variable array
    Ok(Variable::from_json(&serde_json::to_string(&values)?)
        .map_err(|e| anyhow::anyhow!("Failed to create array: {}", e))?)
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

/// Describe a Variable value for verbose output
fn describe_value(value: &Rc<Variable>) -> String {
    match value.as_ref() {
        Variable::Null => "null".to_string(),
        Variable::Bool(b) => format!("bool ({})", b),
        Variable::Number(n) => format!("number ({})", n),
        Variable::String(s) => {
            if s.len() > 50 {
                format!("string ({} chars)", s.len())
            } else {
                format!("string \"{}\"", s)
            }
        }
        Variable::Array(arr) => format!("array ({} items)", arr.len()),
        Variable::Object(obj) => format!("object ({} keys)", obj.len()),
        Variable::Expref(_) => "expression reference".to_string(),
    }
}
