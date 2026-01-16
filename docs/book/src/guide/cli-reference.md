# CLI Reference

Complete reference for all jpx command-line options.

## Synopsis

```
jpx [OPTIONS] [EXPRESSIONS]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[EXPRESSIONS]...` | JMESPath expression(s) to evaluate (multiple are chained as a pipeline) |

## Options

### Input/Output

| Option | Description |
|--------|-------------|
| `-e, --expression <EXPR>` | Expression(s) to evaluate (can be repeated) |
| `-Q, --query-file <FILE>` | Read JMESPath expression from file (supports `.jpx` libraries with colon syntax: `file.jpx:query-name`) |
| `--query <NAME>` | Select a named query from a `.jpx` library |
| `--list-queries` | List all queries in a `.jpx` library file |
| `--check` | Validate all queries in a `.jpx` library without running |
| `-f, --file <FILE>` | Input JSON file (reads stdin if not provided) |
| `-o, --output <FILE>` | Output file (writes to stdout if not provided) |
| `-n, --null-input` | Don't read input, use null as input value |
| `-s, --slurp` | Read all inputs into an array |
| `--stream` | Stream mode - process input line by line (for NDJSON/JSON Lines) |

### Output Format

| Option | Description |
|--------|-------------|
| `-r, --raw` | Output raw strings without quotes |
| `-c, --compact` | Compact output (no pretty printing) |
| `-y, --yaml` | Output as YAML |
| `--toml` | Output as TOML |
| `--csv` | Output as CSV (comma-separated values) |
| `--tsv` | Output as TSV (tab-separated values) |
| `-l, --lines` | Output one JSON value per line |
| `-t, --table` | Output as a formatted table (for arrays of objects) |
| `--table-style <STYLE>` | Table style: `unicode` (default), `ascii`, `markdown`, `plain` |
| `--color <MODE>` | Colorize output: `auto`, `always`, `never` |

See [Output Formats](./output-formats.md) for detailed examples.

### Modes

| Option | Description |
|--------|-------------|
| `--strict` | Strict mode - only standard JMESPath functions |
| `-v, --verbose` | Show expression details and timing |
| `-q, --quiet` | Suppress errors and warnings |

### JSON Patch Operations

| Option | Description |
|--------|-------------|
| `--diff <SOURCE> <TARGET>` | Generate JSON Patch (RFC 6902) from two files |
| `--patch <PATCH_FILE>` | Apply JSON Patch (RFC 6902) to input |
| `--merge <MERGE_FILE>` | Apply JSON Merge Patch (RFC 7396) to input |

### Data Analysis

| Option | Description |
|--------|-------------|
| `--stats` | Show statistics about the input data |
| `--paths` | List all paths in the input JSON |
| `--types` | Show types alongside paths (use with `--paths`) |
| `--values` | Show values alongside paths (use with `--paths`) |

### Benchmarking

| Option | Description |
|--------|-------------|
| `--bench [N]` | Benchmark expression performance (default: 100 iterations) |
| `--warmup <N>` | Number of warmup iterations before benchmarking (default: 5) |

### Function Discovery

| Option | Description |
|--------|-------------|
| `--list-functions` | List all available functions |
| `--list-category <NAME>` | List functions in a specific category |
| `--describe <FUNCTION>` | Show detailed info for a function |
| `--search <QUERY>` | Search functions by name, description, or category (fuzzy matching) |
| `--similar <FUNCTION>` | Find functions similar to the specified function |

### Debugging

| Option | Description |
|--------|-------------|
| `--explain` | Show how an expression is parsed (AST) |
| `--debug` | Show diagnostic information for troubleshooting |

### Interactive Mode

| Option | Description |
|--------|-------------|
| `--repl` | Start interactive REPL mode |
| `--demo <NAME>` | Load a demo dataset (use with `--repl`) |

### Other

| Option | Description |
|--------|-------------|
| `--completions <SHELL>` | Generate shell completions (`bash`, `zsh`, `fish`, `powershell`, `elvish`) |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `JPX_VERBOSE=1` | Enable verbose mode |
| `JPX_QUIET=1` | Enable quiet mode |
| `JPX_STRICT=1` | Enable strict mode |
| `JPX_RAW=1` | Output raw strings |
| `JPX_COMPACT=1` | Compact output |

Environment variables are overridden by command-line flags.

## Examples

### Basic Usage

```bash
# Basic query from stdin
echo '{"name": "Alice"}' | jpx 'name'

# Query a file
jpx 'users[*].email' -f data.json

# Raw output for scripting
jpx -r 'config.api_key' -f settings.json

# Use null input for functions that don't need data
jpx -n 'now()'
```

### Expression Pipelines

```bash
# Chain multiple expressions (output of each feeds the next)
jpx 'users' '[?active]' '[*].name' -f data.json

# Same thing with -e flags
jpx -e 'users' -e '[?active]' -e '[*].name' -f data.json
```

### Streaming and Slurping

```bash
# Slurp multiple JSON objects into an array
cat *.json | jpx -s 'length(@)'

# Stream NDJSON (process line by line with constant memory)
cat logs.ndjson | jpx --stream '[?level == `error`]'
```

### Output Formats

```bash
# Table output for arrays of objects
jpx -t '[*].{name, age, city}' -f users.json

# Markdown table for documentation
jpx -t --table-style markdown '[*].{name, email}' -f users.json

# YAML output
jpx -y 'config' -f settings.json

# CSV for spreadsheets
jpx --csv 'records[*]' -f data.json
```

### Function Discovery

```bash
# List string functions
jpx --list-category string

# Get function documentation
jpx --describe median

# Search for functions by keyword
jpx --search "date format"

# Find similar functions
jpx --similar upper
```

### JSON Patch Operations

```bash
# Generate a patch between two files
jpx --diff original.json modified.json > changes.patch

# Apply a JSON Patch
jpx --patch changes.patch -f document.json

# Apply a JSON Merge Patch
jpx --merge updates.json -f document.json
```

### Data Analysis

```bash
# Show statistics about JSON structure
jpx --stats -f data.json

# List all paths in the JSON
jpx --paths -f data.json

# Show paths with types
jpx --paths --types -f data.json

# Show paths with values
jpx --paths --values -f data.json
```

### Debugging and Development

```bash
# Explain how an expression is parsed
jpx --explain 'users[?active].name'

# Show diagnostic information
jpx --debug -f data.json

# Benchmark expression performance
jpx --bench 'users[?active]' -f data.json

# Benchmark with custom iterations and warmup
jpx --bench 500 --warmup 10 'sort_by(items, &price)' -f data.json
```

### Interactive Mode

```bash
# Start the REPL
jpx --repl

# Start REPL with a demo dataset
jpx --repl --demo users

# Start REPL with your own file
jpx --repl -f data.json
```

### Strict Mode

```bash
# Use only standard JMESPath functions (no extensions)
jpx --strict 'length(items)' -f data.json
```

### Query Libraries

```bash
# List queries in a .jpx library
jpx -Q queries.jpx --list-queries

# Run a named query (colon syntax)
jpx -Q queries.jpx:active-users data.json

# Run a named query (separate flag)
jpx -Q queries.jpx --query active-users data.json

# Validate all queries in a library
jpx -Q queries.jpx --check

# Simple query file (backwards compatible)
echo 'users[?active]' > query.txt
jpx -Q query.txt data.json
```

See [Query Files](./query-files.md) for detailed documentation on query libraries.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid expression, file not found, etc.) |
