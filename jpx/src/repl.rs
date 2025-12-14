//! Interactive REPL for jpx
//!
//! Provides an interactive environment for exploring JSON data with JMESPath queries.

use anyhow::{Context, Result};
use jmespath::{Runtime, Variable};
use jmespath_extensions::register_all;
use jmespath_extensions::registry::{Category, FunctionRegistry};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};
use std::borrow::Cow;
use std::collections::HashSet;

// ANSI color codes
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";

    // JMESPath syntax
    pub const FUNCTION: &str = "\x1b[38;5;39m"; // Blue
    pub const STRING: &str = "\x1b[38;5;78m"; // Green
    pub const NUMBER: &str = "\x1b[38;5;180m"; // Orange/tan
    pub const LITERAL: &str = "\x1b[38;5;141m"; // Purple
    pub const OPERATOR: &str = "\x1b[38;5;204m"; // Pink
    pub const BRACKET: &str = "\x1b[38;5;248m"; // Gray
    pub const FIELD: &str = "\x1b[38;5;255m"; // White
    pub const AT: &str = "\x1b[38;5;220m"; // Yellow
    pub const AMPERSAND: &str = "\x1b[38;5;141m"; // Purple (expression ref)

    // JSON output
    pub const JSON_KEY: &str = "\x1b[38;5;39m"; // Blue
    pub const JSON_STRING: &str = "\x1b[38;5;78m"; // Green
    pub const JSON_NUMBER: &str = "\x1b[38;5;180m"; // Orange
    pub const JSON_BOOL: &str = "\x1b[38;5;220m"; // Yellow
    pub const JSON_NULL: &str = "\x1b[38;5;241m"; // Gray

    // UI
    pub const PROMPT: &str = "\x1b[38;5;39m"; // Blue
    pub const ERROR: &str = "\x1b[38;5;196m"; // Red
    pub const SUCCESS: &str = "\x1b[38;5;78m"; // Green
    pub const INFO: &str = "\x1b[38;5;248m"; // Gray
    pub const HINT: &str = "\x1b[38;5;241m"; // Dark gray
}

/// Demo themes with pre-loaded data and suggested queries
pub struct Demo {
    pub name: &'static str,
    pub description: &'static str,
    pub data: &'static str,
    pub queries: &'static [(&'static str, &'static str)], // (query, description)
}

pub const DEMOS: &[Demo] = &[
    Demo {
        name: "users",
        description: "User profiles with nested data",
        data: r#"{
  "users": [
    {"id": 1, "name": "Alice", "age": 30, "email": "alice@example.com", "role": "admin", "active": true},
    {"id": 2, "name": "Bob", "age": 25, "email": "bob@example.com", "role": "user", "active": true},
    {"id": 3, "name": "Carol", "age": 35, "email": "carol@example.com", "role": "user", "active": false},
    {"id": 4, "name": "Dave", "age": 28, "email": "dave@example.com", "role": "moderator", "active": true}
  ],
  "meta": {"total": 4, "version": "1.0"}
}"#,
        queries: &[
            ("users[*].name", "Get all user names"),
            ("users[?active].name", "Get active user names"),
            ("users[?age > `30`]", "Users older than 30"),
            ("group_by_expr(users, &role)", "Group users by role"),
            (
                "users | sort_by_expr(@, &age) | [].{name: name, age: age}",
                "Sort by age",
            ),
            (
                "{oldest: max_by(users, &age).name, youngest: min_by(users, &age).name}",
                "Oldest and youngest",
            ),
        ],
    },
    Demo {
        name: "geo",
        description: "Geographic locations and distances",
        data: r#"{
  "cities": [
    {"name": "New York", "lat": 40.7128, "lon": -74.0060, "population": 8336817},
    {"name": "Los Angeles", "lat": 34.0522, "lon": -118.2437, "population": 3979576},
    {"name": "London", "lat": 51.5074, "lon": -0.1278, "population": 8982000},
    {"name": "Tokyo", "lat": 35.6762, "lon": 139.6503, "population": 13960000},
    {"name": "Sydney", "lat": -33.8688, "lon": 151.2093, "population": 5312000}
  ],
  "origin": {"name": "San Francisco", "lat": 37.7749, "lon": -122.4194}
}"#,
        queries: &[
            ("cities[*].name", "List all cities"),
            (
                "cities | sort_by(@, &population) | reverse(@) | [0:3].name",
                "Top 3 by population",
            ),
            (
                "geo_distance_km(origin.lat, origin.lon, cities[0].lat, cities[0].lon)",
                "Distance SF to NYC (km)",
            ),
            (
                "geo_bearing(origin.lat, origin.lon, cities[2].lat, cities[2].lon)",
                "Bearing SF to London",
            ),
            (
                "cities[*].{name: name, pop_millions: divide(population, `1000000`)}",
                "Population in millions",
            ),
        ],
    },
    Demo {
        name: "text",
        description: "Text analysis and NLP functions",
        data: r#"{
  "articles": [
    {"title": "Hello World", "body": "This is a simple example. It has two sentences.", "tags": ["intro", "basic"]},
    {"title": "Advanced Topics", "body": "The quick brown fox jumps over the lazy dog. This pangram contains every letter.", "tags": ["advanced", "complete"]}
  ],
  "words": ["hello", "hallo", "helo", "help", "world"]
}"#,
        queries: &[
            (
                "articles[0].body | word_count(@)",
                "Count words in first article",
            ),
            (
                "articles[*].body | [*].word_count(@)",
                "Word counts for all articles",
            ),
            ("articles[0].body | ngrams(@, `2`)", "Word bigrams"),
            (
                "ngrams(articles[1].body, `3`, 'char') | [0:5]",
                "First 5 character trigrams",
            ),
            ("words[*].{word: @, soundex: soundex(@)}", "Soundex codes"),
            (
                "levenshtein(words[0], words[1])",
                "Edit distance: hello vs hallo",
            ),
        ],
    },
    Demo {
        name: "datetime",
        description: "Date and time manipulation",
        data: r#"{
  "events": [
    {"name": "Launch", "timestamp": 1704067200, "duration": "2h30m"},
    {"name": "Meeting", "timestamp": 1704110400, "duration": "1h"},
    {"name": "Review", "timestamp": 1704153600, "duration": "45m"}
  ],
  "now": 1704200000
}"#,
        queries: &[
            (
                "events[*].{name: name, date: format_date(timestamp, '%Y-%m-%d')}",
                "Format dates",
            ),
            (
                "events[*].{name: name, duration_sec: parse_duration(duration)}",
                "Parse durations",
            ),
            (
                "events[*].{name: name, ago: time_ago(timestamp)}",
                "Time ago strings",
            ),
            (
                "events | filter_expr(@, &timestamp > `1704100000`)",
                "Events after timestamp",
            ),
            (
                "now() | format_date(@, '%A, %B %d, %Y')",
                "Current date formatted",
            ),
        ],
    },
    Demo {
        name: "ecommerce",
        description: "E-commerce orders and products",
        data: r#"{
  "orders": [
    {"id": "ORD-001", "customer": "alice", "items": [{"sku": "WIDGET-1", "qty": 2, "price": 9.99}, {"sku": "GADGET-1", "qty": 1, "price": 24.99}], "status": "shipped"},
    {"id": "ORD-002", "customer": "bob", "items": [{"sku": "WIDGET-1", "qty": 5, "price": 9.99}], "status": "pending"},
    {"id": "ORD-003", "customer": "alice", "items": [{"sku": "THING-1", "qty": 1, "price": 149.99}], "status": "delivered"}
  ],
  "customers": {"alice": {"tier": "gold", "discount": 0.1}, "bob": {"tier": "silver", "discount": 0.05}}
}"#,
        queries: &[
            (
                "orders[*].{id: id, total: sum(items[*].multiply(qty, price))}",
                "Order totals",
            ),
            (
                "orders[].items[] | unique_by(@, &sku)",
                "Unique products ordered",
            ),
            ("group_by_expr(orders, &customer)", "Orders by customer"),
            ("orders[?status == 'pending'].id", "Pending order IDs"),
            (
                "orders[*].items[*].price | flatten(@) | {min: min(@), max: max(@), avg: avg(@)}",
                "Price stats",
            ),
        ],
    },
];

/// JMESPath syntax highlighter and completer
pub struct JmespathHelper {
    functions: HashSet<String>,
}

impl JmespathHelper {
    pub fn new() -> Self {
        let mut registry = FunctionRegistry::new();
        registry.register_all();

        let functions: HashSet<String> = registry.functions().map(|f| f.name.to_string()).collect();

        Self { functions }
    }

    /// Highlight JMESPath expression
    fn highlight_jmespath(&self, line: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            match c {
                // String literals
                '\'' => {
                    result.push_str(colors::STRING);
                    result.push(c);
                    i += 1;
                    while i < chars.len() && chars[i] != '\'' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            result.push(chars[i]);
                            i += 1;
                            if i < chars.len() {
                                result.push(chars[i]);
                                i += 1;
                            }
                        } else {
                            result.push(chars[i]);
                            i += 1;
                        }
                    }
                    if i < chars.len() {
                        result.push(chars[i]);
                        i += 1;
                    }
                    result.push_str(colors::RESET);
                }

                // Backtick literals
                '`' => {
                    result.push_str(colors::LITERAL);
                    result.push(c);
                    i += 1;
                    while i < chars.len() && chars[i] != '`' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            result.push(chars[i]);
                            i += 1;
                            if i < chars.len() {
                                result.push(chars[i]);
                                i += 1;
                            }
                        } else {
                            result.push(chars[i]);
                            i += 1;
                        }
                    }
                    if i < chars.len() {
                        result.push(chars[i]);
                        i += 1;
                    }
                    result.push_str(colors::RESET);
                }

                // Current node
                '@' => {
                    result.push_str(colors::AT);
                    result.push(c);
                    result.push_str(colors::RESET);
                    i += 1;
                }

                // Expression reference
                '&' => {
                    result.push_str(colors::AMPERSAND);
                    result.push(c);
                    result.push_str(colors::RESET);
                    i += 1;
                }

                // Brackets
                '[' | ']' | '{' | '}' | '(' | ')' => {
                    result.push_str(colors::BRACKET);
                    result.push(c);
                    result.push_str(colors::RESET);
                    i += 1;
                }

                // Operators
                '|' | '.' | ',' | ':' | '?' | '*' | '!' => {
                    result.push_str(colors::OPERATOR);
                    result.push(c);
                    result.push_str(colors::RESET);
                    i += 1;
                }

                // Comparison operators
                '=' | '<' | '>' => {
                    result.push_str(colors::OPERATOR);
                    result.push(c);
                    i += 1;
                    // Handle == != <= >=
                    if i < chars.len() && (chars[i] == '=' || (c == '!' && chars[i] == '=')) {
                        result.push(chars[i]);
                        i += 1;
                    }
                    result.push_str(colors::RESET);
                }

                // Identifiers (fields or functions)
                'a'..='z' | 'A'..='Z' | '_' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();

                    // Check if it's a function (followed by '(')
                    let is_function =
                        i < chars.len() && chars[i] == '(' && self.functions.contains(&word);

                    if is_function {
                        result.push_str(colors::FUNCTION);
                        result.push_str(&word);
                        result.push_str(colors::RESET);
                    } else {
                        result.push_str(colors::FIELD);
                        result.push_str(&word);
                        result.push_str(colors::RESET);
                    }
                }

                // Numbers
                '0'..='9' | '-'
                    if c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() =>
                {
                    result.push_str(colors::NUMBER);
                    result.push(c);
                    i += 1;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        result.push(chars[i]);
                        i += 1;
                    }
                    result.push_str(colors::RESET);
                }

                // Everything else
                _ => {
                    result.push(c);
                    i += 1;
                }
            }
        }

        result
    }
}

impl Completer for JmespathHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find the start of the current word
        let line_to_pos = &line[..pos];
        let word_start = line_to_pos
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = &line[word_start..pos];

        if prefix.is_empty() {
            return Ok((pos, vec![]));
        }

        let mut completions: Vec<Pair> = self
            .functions
            .iter()
            .filter(|f| f.starts_with(prefix))
            .map(|f| Pair {
                display: f.clone(),
                replacement: format!("{}(", f),
            })
            .collect();

        completions.sort_by(|a, b| a.display.cmp(&b.display));

        Ok((word_start, completions))
    }
}

impl Hinter for JmespathHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }

        // Find partial function name
        let word_start = line
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = &line[word_start..];

        if prefix.len() < 2 {
            return None;
        }

        // Find first matching function
        self.functions
            .iter()
            .filter(|f| f.starts_with(prefix))
            .min()
            .map(|f| {
                let suffix = &f[prefix.len()..];
                format!("{}{}({})", colors::HINT, suffix, colors::RESET)
            })
    }
}

impl Highlighter for JmespathHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(self.highlight_jmespath(line))
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Owned(format!("{}{}{}", colors::PROMPT, prompt, colors::RESET))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint) // Already colored in hint()
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}

impl Validator for JmespathHelper {}

impl Helper for JmespathHelper {}

/// Colorize JSON output
pub fn colorize_json(value: &serde_json::Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);

    match value {
        serde_json::Value::Null => {
            format!("{}null{}", colors::JSON_NULL, colors::RESET)
        }
        serde_json::Value::Bool(b) => {
            format!("{}{}{}", colors::JSON_BOOL, b, colors::RESET)
        }
        serde_json::Value::Number(n) => {
            format!("{}{}{}", colors::JSON_NUMBER, n, colors::RESET)
        }
        serde_json::Value::String(s) => {
            format!(
                "{}\"{}\"{}",
                colors::JSON_STRING,
                escape_string(s),
                colors::RESET
            )
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else if arr.len() <= 3
                && arr.iter().all(|v| {
                    matches!(
                        v,
                        serde_json::Value::Number(_)
                            | serde_json::Value::Bool(_)
                            | serde_json::Value::Null
                    )
                })
            {
                // Compact format for short simple arrays
                let items: Vec<String> = arr.iter().map(|v| colorize_json(v, 0)).collect();
                format!("[{}]", items.join(", "))
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| format!("{}  {}", prefix, colorize_json(v, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), prefix)
            }
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}  {}\"{}\"{}: {}",
                            prefix,
                            colors::JSON_KEY,
                            k,
                            colors::RESET,
                            colorize_json(v, indent + 1)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), prefix)
            }
        }
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Describe a loaded value
fn describe_value(value: &Variable) -> String {
    match value {
        Variable::Null => "null".to_string(),
        Variable::Bool(_) => "boolean".to_string(),
        Variable::Number(_) => "number".to_string(),
        Variable::String(s) => format!("string ({} chars)", s.len()),
        Variable::Array(arr) => format!("array ({} items)", arr.len()),
        Variable::Object(obj) => format!("object ({} keys)", obj.len()),
        Variable::Expref(_) => "expression reference".to_string(),
    }
}

/// Run the REPL
pub fn run(demo_name: Option<&str>) -> Result<()> {
    let helper = JmespathHelper::new();
    let mut rl: Editor<JmespathHelper, DefaultHistory> = Editor::new()?;
    rl.set_helper(Some(helper));

    // Try to load history
    let history_path = dirs::data_local_dir().map(|p| p.join("jpx").join("history.txt"));

    if let Some(ref path) = history_path {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let _ = rl.load_history(path);
    }

    // Create runtime
    let mut runtime = Runtime::new();
    runtime.register_builtin_functions();
    register_all(&mut runtime);

    // Create registry for introspection
    let mut registry = FunctionRegistry::new();
    registry.register_all();

    // Current data
    let mut data: Option<Variable> = None;

    // Print banner
    println!(
        "{}{}jpx{} - JMESPath Extended REPL",
        colors::BOLD,
        colors::PROMPT,
        colors::RESET
    );
    println!(
        "{}Type .help for commands, .exit to quit{}\n",
        colors::INFO,
        colors::RESET
    );

    // Load demo if specified
    if let Some(name) = demo_name {
        if let Some(demo) = DEMOS.iter().find(|d| d.name == name) {
            data = Some(Variable::from_json(demo.data).unwrap());
            println!(
                "{}Loaded demo:{} {} - {}",
                colors::SUCCESS,
                colors::RESET,
                demo.name,
                demo.description
            );
            println!(
                "{}Data:{} {}\n",
                colors::INFO,
                colors::RESET,
                describe_value(data.as_ref().unwrap())
            );
            println!("{}Try these queries:{}", colors::INFO, colors::RESET);
            for (query, desc) in demo.queries {
                println!("  {}# {}{}", colors::HINT, desc, colors::RESET);
                println!("  {}", query);
            }
            println!();
        } else {
            println!(
                "{}Unknown demo '{}'. Available: {}{}",
                colors::ERROR,
                name,
                DEMOS.iter().map(|d| d.name).collect::<Vec<_>>().join(", "),
                colors::RESET
            );
        }
    }

    loop {
        let prompt = if data.is_some() {
            "jpx> "
        } else {
            "jpx (no data)> "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                // Handle commands
                if line.starts_with('.') {
                    if let Err(e) = handle_command(line, &mut data, &registry, &mut rl) {
                        println!("{}Error: {}{}", colors::ERROR, e, colors::RESET);
                    }
                    continue;
                }

                // Execute JMESPath expression
                if let Some(ref d) = data {
                    match runtime.compile(line) {
                        Ok(expr) => match expr.search(d) {
                            Ok(result) => {
                                if !result.is_null() {
                                    let json_value: serde_json::Value =
                                        serde_json::to_value(&*result).unwrap();
                                    println!("{}", colorize_json(&json_value, 0));
                                } else {
                                    println!("{}null{}", colors::JSON_NULL, colors::RESET);
                                }
                            }
                            Err(e) => {
                                println!("{}Runtime error: {}{}", colors::ERROR, e, colors::RESET);
                            }
                        },
                        Err(e) => {
                            println!("{}Parse error: {}{}", colors::ERROR, e, colors::RESET);
                        }
                    }
                } else {
                    println!(
                        "{}No data loaded. Use .load <file> or .demo <name>{}",
                        colors::ERROR,
                        colors::RESET
                    );
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}Use .exit to quit{}", colors::INFO, colors::RESET);
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("{}Error: {}{}", colors::ERROR, err, colors::RESET);
                break;
            }
        }
    }

    // Save history
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    println!("Goodbye!");
    Ok(())
}

fn handle_command(
    line: &str,
    data: &mut Option<Variable>,
    registry: &FunctionRegistry,
    rl: &mut Editor<JmespathHelper, DefaultHistory>,
) -> Result<()> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let cmd = parts[0];
    let arg = parts.get(1).map(|s| s.trim());

    match cmd {
        ".exit" | ".quit" | ".q" => {
            std::process::exit(0);
        }

        ".help" | ".h" | ".?" => {
            println!("{}Commands:{}", colors::BOLD, colors::RESET);
            println!(
                "  {}.load <file>{}     Load JSON from file",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.json [json]{}     Load JSON (inline or multiline mode)",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.data{}            Show current data",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.demo [name]{}     Load demo dataset (users, geo, text, datetime, ecommerce)",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.demos{}           List available demos",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.functions{}       List all functions",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.describe <fn>{}   Describe a function",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.clear{}           Clear screen",
                colors::FUNCTION,
                colors::RESET
            );
            println!(
                "  {}.exit{}            Exit REPL",
                colors::FUNCTION,
                colors::RESET
            );
            println!();
            println!("{}Tips:{}", colors::BOLD, colors::RESET);
            println!("  - Tab completion for function names");
            println!("  - Up/Down arrows for history");
            println!("  - Ctrl+R to search history");
        }

        ".load" => {
            let path = arg.ok_or_else(|| anyhow::anyhow!("Usage: .load <file>"))?;
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path))?;
            let value = Variable::from_json(&content)
                .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
            println!(
                "{}Loaded:{} {}",
                colors::SUCCESS,
                colors::RESET,
                describe_value(&value)
            );
            *data = Some(value);
        }

        ".json" => {
            let json_str = if let Some(inline) = arg {
                // Inline JSON provided
                inline.to_string()
            } else {
                // Multiline input mode
                println!(
                    "{}Enter JSON (empty line to finish, Ctrl+C to cancel):{}",
                    colors::INFO,
                    colors::RESET
                );
                let mut lines = Vec::new();
                loop {
                    match rl.readline("... ") {
                        Ok(line) => {
                            if line.trim().is_empty() && !lines.is_empty() {
                                // Check if we have valid JSON so far
                                let current = lines.join("\n");
                                if serde_json::from_str::<serde_json::Value>(&current).is_ok() {
                                    break;
                                }
                            }
                            lines.push(line);

                            // Try to parse - if valid, we're done
                            let current = lines.join("\n");
                            if serde_json::from_str::<serde_json::Value>(&current).is_ok() {
                                break;
                            }
                        }
                        Err(rustyline::error::ReadlineError::Interrupted) => {
                            println!("{}Cancelled{}", colors::INFO, colors::RESET);
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Read error: {}", e));
                        }
                    }
                }
                lines.join("\n")
            };

            let value = Variable::from_json(&json_str)
                .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
            println!(
                "{}Loaded:{} {}",
                colors::SUCCESS,
                colors::RESET,
                describe_value(&value)
            );
            *data = Some(value);
        }

        ".data" => {
            if let Some(d) = data {
                let json_value: serde_json::Value = serde_json::to_value(&*d).unwrap();
                println!("{}", colorize_json(&json_value, 0));
            } else {
                println!("{}No data loaded{}", colors::INFO, colors::RESET);
            }
        }

        ".demo" => {
            let name = arg.unwrap_or("users");
            if let Some(demo) = DEMOS.iter().find(|d| d.name == name) {
                *data = Some(Variable::from_json(demo.data).unwrap());
                println!(
                    "{}Loaded demo:{} {} - {}",
                    colors::SUCCESS,
                    colors::RESET,
                    demo.name,
                    demo.description
                );
                println!(
                    "{}Data:{} {}\n",
                    colors::INFO,
                    colors::RESET,
                    describe_value(data.as_ref().unwrap())
                );
                println!("{}Try these queries:{}", colors::INFO, colors::RESET);
                for (query, desc) in demo.queries {
                    println!("  {}# {}{}", colors::HINT, desc, colors::RESET);
                    println!("  {}", query);
                }
            } else {
                println!(
                    "{}Unknown demo '{}'. Available: {}{}",
                    colors::ERROR,
                    name,
                    DEMOS.iter().map(|d| d.name).collect::<Vec<_>>().join(", "),
                    colors::RESET
                );
            }
        }

        ".demos" => {
            println!("{}Available demos:{}", colors::BOLD, colors::RESET);
            for demo in DEMOS {
                println!(
                    "  {}{:<12}{} - {}",
                    colors::FUNCTION,
                    demo.name,
                    colors::RESET,
                    demo.description
                );
            }
            println!(
                "\nUse {}.demo <name>{} to load",
                colors::FUNCTION,
                colors::RESET
            );
        }

        ".functions" | ".funcs" => {
            // Group by category
            for category in Category::all() {
                if !category.is_available() {
                    continue;
                }

                let funcs: Vec<_> = registry.functions_in_category(*category).collect();
                if funcs.is_empty() {
                    continue;
                }

                let names: Vec<_> = funcs.iter().map(|f| f.name).collect();
                println!(
                    "{}{}{}: {}",
                    colors::BOLD,
                    category.name().to_uppercase(),
                    colors::RESET,
                    names.join(", ")
                );
            }
        }

        ".describe" | ".desc" => {
            let name = arg.ok_or_else(|| anyhow::anyhow!("Usage: .describe <function>"))?;
            if let Some(func) = registry.get_function(name) {
                println!("{}{}{}", colors::BOLD, func.name, colors::RESET);
                println!("  Category:    {}", func.category.name());
                println!("  Description: {}", func.description);
                println!("  Signature:   {}", func.signature);
                println!("  Example:     {}", func.example);
                if let Some(jep) = func.jep {
                    println!("  JEP:         {}", jep);
                }
            } else {
                println!(
                    "{}Unknown function '{}'{}",
                    colors::ERROR,
                    name,
                    colors::RESET
                );
            }
        }

        ".clear" | ".cls" => {
            print!("\x1b[2J\x1b[H");
        }

        _ => {
            println!(
                "{}Unknown command '{}'. Type .help for commands{}",
                colors::ERROR,
                cmd,
                colors::RESET
            );
        }
    }

    Ok(())
}
