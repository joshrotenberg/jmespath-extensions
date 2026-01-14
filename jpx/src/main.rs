#[cfg(feature = "mcp")]
mod mcp;
mod repl;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum, builder::styling};
use clap_complete::{Shell, generate};
use colored::Colorize;
use jmespath::ast::Ast;
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{Category, FunctionRegistry};
use std::fs::File;
use std::io::{self, Read, Write};
use std::rc::Rc;
use std::time::Instant;

// Cargo-style help coloring
const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

/// Check if an environment variable is set to a "truthy" value
fn env_is_true(var: &str) -> bool {
    std::env::var(var)
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Apply environment variable defaults to args
/// CLI args take precedence over env vars (if CLI flag is set, don't override)
fn apply_env_defaults(args: &mut Args) {
    // Only apply env var if CLI flag wasn't explicitly set
    // Since clap sets bool flags to false by default, we check env vars
    // and set to true if the env var is truthy
    if !args.verbose && env_is_true("JPX_VERBOSE") {
        args.verbose = true;
    }
    if !args.quiet && env_is_true("JPX_QUIET") {
        args.quiet = true;
    }
    if !args.strict && env_is_true("JPX_STRICT") {
        args.strict = true;
    }
    if !args.raw && env_is_true("JPX_RAW") {
        args.raw = true;
    }
    if !args.compact && env_is_true("JPX_COMPACT") {
        args.compact = true;
    }
}

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

/// Subcommands for jpx
#[derive(Subcommand, Debug)]
enum Commands {
    /// Start MCP (Model Context Protocol) server for AI assistant integration
    #[cfg(feature = "mcp")]
    Mcp {
        /// Strict mode - only use standard JMESPath functions (no extensions)
        #[arg(long)]
        strict: bool,
    },
}

/// JMESPath CLI with extended functions
///
/// A command-line tool for querying JSON data using JMESPath expressions
/// with 150+ additional functions beyond the standard specification.
#[derive(Parser, Debug)]
#[command(name = "jpx")]
#[command(version, about, long_about = None)]
#[command(styles = STYLES)]
#[command(after_help = concat!(
    "Examples:\n",
    "  echo '{\"name\": \"alice\"}' | jpx 'name'\n",
    "  echo '[1, 2, 3]' | jpx 'sum(@)'\n",
    "  echo '{\"ts\": \"2024-01-15\"}' | jpx 'format_date(ts, \"%B %d, %Y\")'\n",
    "  jpx -n 'now()'\n",
    "  cat data.json | jpx -e 'items[*].name' -e 'sort(@)'\n",
    "\nVersion: ", env!("CARGO_PKG_VERSION"),
    "\nDocumentation: https://docs.rs/jmespath_extensions"
))]
struct Args {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,

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
    /// Can also be set with JPX_RAW=1
    #[arg(short = 'r', long)]
    raw: bool,

    /// Compact output (no pretty printing)
    /// Can also be set with JPX_COMPACT=1
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
    /// Can also be set with JPX_QUIET=1
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Verbose mode - show expression details and timing
    /// Can also be set with JPX_VERBOSE=1
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Strict mode - only use standard JMESPath functions (no extensions)
    /// Can also be set with JPX_STRICT=1
    #[arg(long)]
    strict: bool,

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

    /// Search functions by name, description, or category (fuzzy matching)
    #[arg(long, value_name = "QUERY")]
    search: Option<String>,

    /// Generate JSON Patch (RFC 6902) from two files: --diff <source> <target>
    #[arg(long, num_args = 2, value_names = ["SOURCE", "TARGET"])]
    diff: Option<Vec<String>>,

    /// Apply JSON Patch (RFC 6902): --patch <file> (reads patch from stdin, or use -f for document)
    #[arg(long, value_name = "PATCH_FILE")]
    patch: Option<String>,

    /// Apply JSON Merge Patch (RFC 7396): --merge <file> (reads merge patch from stdin, or use -f for document)
    #[arg(long, value_name = "MERGE_FILE")]
    merge: Option<String>,

    /// Explain how an expression is parsed (show AST)
    #[arg(long)]
    explain: bool,

    /// Start interactive REPL mode
    #[arg(long)]
    repl: bool,

    /// Load a demo dataset (use with --repl)
    #[arg(long, value_name = "NAME")]
    demo: Option<String>,
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    apply_env_defaults(&mut args);

    // Handle subcommands
    #[cfg(feature = "mcp")]
    if let Some(Commands::Mcp { strict }) = args.command {
        return tokio::runtime::Runtime::new()?.block_on(mcp::run(strict));
    }

    // Handle shell completions
    if let Some(shell) = args.completions {
        let mut cmd = Args::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return Ok(());
    }

    // Handle REPL mode
    if args.repl || args.demo.is_some() {
        return repl::run(args.demo.as_deref());
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

    if let Some(query) = &args.search {
        search_functions(&registry, query);
        return Ok(());
    }

    // Handle --diff: generate JSON Patch from two files
    if let Some(files) = &args.diff {
        return diff_files(&files[0], &files[1], args.compact, &args.color);
    }

    // Handle --patch: apply JSON Patch to document
    if let Some(patch_file) = &args.patch {
        return apply_patch(&args.file, patch_file, args.compact, &args.color);
    }

    // Handle --merge: apply JSON Merge Patch to document
    if let Some(merge_file) = &args.merge {
        return apply_merge(&args.file, merge_file, args.compact, &args.color);
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
        std::mem::take(&mut args.expressions)
    } else if let Some(expr) = args.expression.take() {
        vec![expr]
    } else {
        return Err(anyhow::anyhow!(
            "Expression required. Use --help for usage."
        ));
    };

    // Handle --explain: parse and show AST without evaluating
    if args.explain {
        for (i, expression) in expressions.iter().enumerate() {
            if expressions.len() > 1 {
                println!("Expression {}: {}", i + 1, expression);
                println!("{}", "=".repeat(expression.len() + 14));
            } else {
                println!("Expression: {}", expression);
                println!("{}", "=".repeat(expression.len() + 12));
            }
            println!();

            let ast = jmespath::parse(expression)
                .with_context(|| format!("Failed to parse expression: {}", expression))?;

            print_ast(&ast, 0);
            println!();
        }
        return Ok(());
    }

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

    // Create runtime with extensions (unless strict mode)
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    if !args.strict {
        register_all(&mut runtime);
    }

    // Verbose mode: show input info
    if args.verbose {
        if args.strict {
            eprintln!("Mode: strict (standard JMESPath only)");
        }
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
        result = match expr.search(&result) {
            Ok(r) => r,
            Err(e) => {
                let err_msg = e.to_string();
                if args.strict && err_msg.contains("undefined function") {
                    return Err(anyhow::anyhow!(
                        "{}\n\nHint: You are using --strict mode which only allows standard JMESPath functions.\nRemove --strict or unset JPX_STRICT to use extension functions.",
                        err_msg
                    ));
                }
                return Err(anyhow::anyhow!("Failed to evaluate expression: {}", e));
            }
        };
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

    #[allow(clippy::collapsible_if)]
    if args.raw {
        if let Some(s) = result.as_string() {
            println!("{}", s);
            return Ok(());
        }
    }

    // Convert to serde_json::Value for output formatting
    let json_value: serde_json::Value = serde_json::to_value(&*result)?;

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

    let mut values: Vec<serde_json::Value> = Vec::new();
    let stream = Deserializer::from_str(input).into_iter::<serde_json::Value>();

    for result in stream {
        let value = result.context("Failed to parse JSON in slurp mode")?;
        values.push(value);
    }

    // Convert the collected values directly to Variable
    let array_value = serde_json::Value::Array(values);
    Variable::from_json(&array_value.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to create array: {}", e))
}

fn print_functions(registry: &FunctionRegistry) {
    println!(
        "{}\n",
        "jpx - JMESPath with Extended Functions".bold().green()
    );

    // Count standard and extension functions
    let standard_count = registry.functions().filter(|f| f.is_standard).count();
    let extension_count = registry.functions().filter(|f| !f.is_standard).count();

    // Print standard functions header
    println!(
        "  {} {}:",
        "▸".dimmed(),
        format!("STANDARD ({})", standard_count).dimmed()
    );
    for func in registry.functions_in_category(Category::Standard) {
        println!(
            "    {} [{}] - {}",
            func.name.cyan().bold(),
            "std".dimmed(),
            func.description
        );
    }
    println!();

    // Print extension functions by category
    println!(
        "{} ({} available):\n",
        "Extension functions".bold(),
        extension_count.to_string().yellow()
    );

    for category in Category::all() {
        if *category == Category::Standard || !category.is_available() {
            continue;
        }

        let funcs: Vec<_> = registry.functions_in_category(*category).collect();
        if funcs.is_empty() {
            continue;
        }

        println!(
            "  {} {}:",
            "▸".dimmed(),
            format!("{} ({})", category.name().to_uppercase(), funcs.len())
                .green()
                .bold()
        );
        for func in funcs {
            println!("    {} - {}", func.name.cyan().bold(), func.description);
        }
        println!();
    }

    println!(
        "Use {} for details on a category",
        "--list-category <name>".cyan()
    );
    println!(
        "Use {} for details on a specific function",
        "--describe <function>".cyan()
    );
    println!("Use {} to find functions", "--search <query>".cyan());
    println!(
        "\nFor full documentation: {}",
        "https://docs.rs/jmespath_extensions".blue().underline()
    );
}

/// Search result with relevance score
struct SearchResult<'a> {
    func: &'a jmespath_extensions::registry::FunctionInfo,
    score: i32,
    match_type: MatchType,
}

#[derive(Debug, Clone, Copy)]
enum MatchType {
    ExactName,
    NamePrefix,
    NameContains,
    AliasMatch,
    CategoryMatch,
    DescriptionMatch,
    SignatureMatch,
}

impl MatchType {
    fn label(&self) -> &'static str {
        match self {
            MatchType::ExactName => "exact",
            MatchType::NamePrefix => "prefix",
            MatchType::NameContains => "name",
            MatchType::AliasMatch => "alias",
            MatchType::CategoryMatch => "category",
            MatchType::DescriptionMatch => "description",
            MatchType::SignatureMatch => "signature",
        }
    }
}

fn search_functions(registry: &FunctionRegistry, query: &str) {
    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    for func in registry.functions() {
        let name_lower = func.name.to_lowercase();
        let desc_lower = func.description.to_lowercase();
        let sig_lower = func.signature.to_lowercase();
        let cat_lower = func.category.name().to_lowercase();

        // Scoring: higher = better match
        let (score, match_type) = if name_lower == query_lower {
            (1000, MatchType::ExactName)
        } else if name_lower.starts_with(&query_lower) {
            (800, MatchType::NamePrefix)
        } else if name_lower.contains(&query_lower) {
            (600, MatchType::NameContains)
        } else if func
            .aliases
            .iter()
            .any(|a| a.to_lowercase().contains(&query_lower))
        {
            (500, MatchType::AliasMatch)
        } else if cat_lower.contains(&query_lower) {
            (400, MatchType::CategoryMatch)
        } else if desc_lower.contains(&query_lower) {
            // Boost if query appears early in description
            let pos = desc_lower.find(&query_lower).unwrap_or(100);
            (300 - pos.min(100) as i32, MatchType::DescriptionMatch)
        } else if sig_lower.contains(&query_lower) {
            (100, MatchType::SignatureMatch)
        } else {
            continue; // No match
        };

        results.push(SearchResult {
            func,
            score,
            match_type,
        });
    }

    // Sort by score descending
    results.sort_by(|a, b| b.score.cmp(&a.score));

    if results.is_empty() {
        println!(
            "{} No functions found matching '{}'\n",
            "✗".red(),
            query.yellow()
        );
        println!("Try searching for:");
        println!("  • Function names: {}", "median, split, json".cyan());
        println!("  • Categories: {}", "math, string, array".cyan());
        println!("  • Concepts: {}", "hash, encode, date".cyan());
        return;
    }

    println!(
        "{} Found {} functions matching '{}':\n",
        "✓".green(),
        results.len().to_string().yellow(),
        query.cyan()
    );

    // Group by match type for nicer display
    let mut current_group: Option<&str> = None;

    for result in results.iter().take(25) {
        let group = result.match_type.label();
        if current_group != Some(group) {
            if current_group.is_some() {
                println!();
            }
            println!("  {} {}:", "▸".dimmed(), group.to_uppercase().dimmed());
            current_group = Some(group);
        }

        let type_badge = if result.func.is_standard {
            "std".dimmed()
        } else {
            result.func.category.name().green()
        };

        println!(
            "    {} [{}] - {}",
            result.func.name.cyan().bold(),
            type_badge,
            result.func.description
        );
    }

    if results.len() > 25 {
        println!(
            "\n  {} and {} more... (refine your search)",
            "...".dimmed(),
            (results.len() - 25).to_string().yellow()
        );
    }

    println!(
        "\nUse {} for details on a specific function",
        "--describe <function>".cyan()
    );
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

    println!(
        "{} functions:\n",
        category.name().to_uppercase().green().bold()
    );

    for func in registry.functions_in_category(*category) {
        println!("  {} - {}", func.name.cyan().bold(), func.description);
        println!("    {}: {}", "Signature".dimmed(), func.signature);
        println!("    {}: {}", "Example".dimmed(), func.example.yellow());
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

    println!("{}", func.name.cyan().bold());
    println!("{}", "=".repeat(func.name.len()).dimmed());
    println!();
    println!(
        "{}: {}",
        "Type".dimmed(),
        if func.is_standard {
            "standard JMESPath".normal()
        } else {
            "extension".green()
        }
    );
    println!("{}: {}", "Category".dimmed(), func.category.name().green());
    if let Some(jep) = func.jep {
        println!("{}: {}", "JEP".dimmed(), jep.yellow());
    }
    println!("{}: {}", "Description".dimmed(), func.description);
    println!("{}: {}", "Signature".dimmed(), func.signature.white());
    println!();
    println!("{}:", "Example".bold());
    println!("  {}", func.example.yellow());

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

/// Print AST in a human-readable tree format
fn print_ast(node: &Ast, indent: usize) {
    let prefix = "  ".repeat(indent);
    let connector = if indent > 0 { "├─ " } else { "" };

    match node {
        Ast::Identity { .. } => {
            println!("{}{}@ (current node)", prefix, connector);
        }
        Ast::Field { name, .. } => {
            println!("{}{}Field: {}", prefix, connector, name);
        }
        Ast::Index { idx, .. } => {
            println!("{}{}Index: [{}]", prefix, connector, idx);
        }
        Ast::Slice {
            start, stop, step, ..
        } => {
            let start_str = start.map_or("".to_string(), |s| s.to_string());
            let stop_str = stop.map_or("".to_string(), |s| s.to_string());
            if *step == 1 {
                println!("{}{}Slice: [{}:{}]", prefix, connector, start_str, stop_str);
            } else {
                println!(
                    "{}{}Slice: [{}:{}:{}]",
                    prefix, connector, start_str, stop_str, step
                );
            }
        }
        Ast::Subexpr { lhs, rhs, .. } => {
            println!("{}{}Subexpression (a.b):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Projection { lhs, rhs, .. } => {
            println!("{}{}Projection (map over array):", prefix, connector);
            println!("{}  source:", prefix);
            print_ast(lhs, indent + 2);
            println!("{}  project:", prefix);
            print_ast(rhs, indent + 2);
        }
        Ast::Function { name, args, .. } => {
            if args.is_empty() {
                println!("{}{}Function: {}()", prefix, connector, name);
            } else {
                println!("{}{}Function: {}", prefix, connector, name);
                for (i, arg) in args.iter().enumerate() {
                    println!("{}  arg {}:", prefix, i + 1);
                    print_ast(arg, indent + 2);
                }
            }
        }
        Ast::Literal { value, .. } => {
            let json = serde_json::to_string(&**value).unwrap_or_else(|_| "?".to_string());
            println!("{}{}Literal: `{}`", prefix, connector, json);
        }
        Ast::Comparison {
            comparator,
            lhs,
            rhs,
            ..
        } => {
            let op = match comparator {
                jmespath::ast::Comparator::Equal => "==",
                jmespath::ast::Comparator::NotEqual => "!=",
                jmespath::ast::Comparator::LessThan => "<",
                jmespath::ast::Comparator::LessThanEqual => "<=",
                jmespath::ast::Comparator::GreaterThan => ">",
                jmespath::ast::Comparator::GreaterThanEqual => ">=",
            };
            println!("{}{}Comparison: {}", prefix, connector, op);
            println!("{}  left:", prefix);
            print_ast(lhs, indent + 2);
            println!("{}  right:", prefix);
            print_ast(rhs, indent + 2);
        }
        Ast::And { lhs, rhs, .. } => {
            println!("{}{}And (&&):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Or { lhs, rhs, .. } => {
            println!("{}{}Or (||):", prefix, connector);
            print_ast(lhs, indent + 1);
            print_ast(rhs, indent + 1);
        }
        Ast::Not { node, .. } => {
            println!("{}{}Not (!):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::Condition {
            predicate, then, ..
        } => {
            println!("{}{}Filter condition ([?...]):", prefix, connector);
            println!("{}  predicate:", prefix);
            print_ast(predicate, indent + 2);
            println!("{}  then:", prefix);
            print_ast(then, indent + 2);
        }
        Ast::Flatten { node, .. } => {
            println!("{}{}Flatten ([]):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::ObjectValues { node, .. } => {
            println!("{}{}Object values (*):", prefix, connector);
            print_ast(node, indent + 1);
        }
        Ast::MultiList { elements, .. } => {
            println!(
                "{}{}Multi-select list ({} elements):",
                prefix,
                connector,
                elements.len()
            );
            for (i, elem) in elements.iter().enumerate() {
                println!("{}  [{}]:", prefix, i);
                print_ast(elem, indent + 2);
            }
        }
        Ast::MultiHash { elements, .. } => {
            println!(
                "{}{}Multi-select hash ({} keys):",
                prefix,
                connector,
                elements.len()
            );
            for kvp in elements {
                println!("{}  {}:", prefix, kvp.key);
                print_ast(&kvp.value, indent + 2);
            }
        }
        Ast::Expref { ast, .. } => {
            println!("{}{}Expression reference (&):", prefix, connector);
            print_ast(ast, indent + 1);
        }
    }
}

// =============================================================================
// JSON Patch / Merge operations
// =============================================================================

/// Read JSON from a file or stdin
fn read_json_from(path: Option<&str>) -> Result<serde_json::Value> {
    let content = if let Some(p) = path {
        std::fs::read_to_string(p).with_context(|| format!("Failed to read file: {}", p))?
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        buf
    };
    serde_json::from_str(&content).context("Failed to parse JSON")
}

/// Output JSON with optional coloring
fn output_json(value: &serde_json::Value, compact: bool, color_mode: &ColorMode) -> Result<()> {
    let use_color = match color_mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => atty::is(atty::Stream::Stdout),
    };

    if compact {
        println!("{}", serde_json::to_string(value)?);
    } else if use_color {
        use colored_json::{ColoredFormatter, PrettyFormatter};
        let formatter = ColoredFormatter::new(PrettyFormatter::new());
        println!("{}", formatter.to_colored_json_auto(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

/// Generate JSON Patch (RFC 6902) from two files
fn diff_files(
    source_path: &str,
    target_path: &str,
    compact: bool,
    color_mode: &ColorMode,
) -> Result<()> {
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(source_path)
            .with_context(|| format!("Failed to read source file: {}", source_path))?,
    )
    .context("Failed to parse source JSON")?;

    let target: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(target_path)
            .with_context(|| format!("Failed to read target file: {}", target_path))?,
    )
    .context("Failed to parse target JSON")?;

    let patch = json_patch::diff(&source, &target);

    if patch.0.is_empty() {
        eprintln!("{} No differences found", "✓".green());
        println!("[]");
        return Ok(());
    }

    eprintln!(
        "{} Generated {} patch operation(s)",
        "✓".green(),
        patch.0.len().to_string().yellow()
    );

    let patch_value = serde_json::to_value(&patch)?;
    output_json(&patch_value, compact, color_mode)
}

/// Apply JSON Patch (RFC 6902) to a document
fn apply_patch(
    doc_path: &Option<String>,
    patch_path: &str,
    compact: bool,
    color_mode: &ColorMode,
) -> Result<()> {
    // Read the document (from -f or stdin)
    let mut doc = read_json_from(doc_path.as_deref())?;

    // Read the patch from file
    let patch_content = std::fs::read_to_string(patch_path)
        .with_context(|| format!("Failed to read patch file: {}", patch_path))?;
    let patch_value: serde_json::Value =
        serde_json::from_str(&patch_content).context("Failed to parse patch JSON")?;

    // Convert to json_patch::Patch
    let patch: json_patch::Patch =
        serde_json::from_value(patch_value).context("Invalid JSON Patch format")?;

    // Apply the patch
    json_patch::patch(&mut doc, &patch).context("Failed to apply patch")?;

    eprintln!(
        "{} Applied {} patch operation(s)",
        "✓".green(),
        patch.0.len().to_string().yellow()
    );

    output_json(&doc, compact, color_mode)
}

/// Apply JSON Merge Patch (RFC 7396) to a document
fn apply_merge(
    doc_path: &Option<String>,
    merge_path: &str,
    compact: bool,
    color_mode: &ColorMode,
) -> Result<()> {
    // Read the document (from -f or stdin)
    let mut doc = read_json_from(doc_path.as_deref())?;

    // Read the merge patch from file
    let merge_content = std::fs::read_to_string(merge_path)
        .with_context(|| format!("Failed to read merge patch file: {}", merge_path))?;
    let merge_patch: serde_json::Value =
        serde_json::from_str(&merge_content).context("Failed to parse merge patch JSON")?;

    // Apply the merge patch
    json_patch::merge(&mut doc, &merge_patch);

    eprintln!("{} Applied merge patch", "✓".green());

    output_json(&doc, compact, color_mode)
}
