# CLI Reference

Complete reference for all jpx command-line options.

## Synopsis

```
jpx [OPTIONS] [EXPRESSION]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[EXPRESSION]` | JMESPath expression to evaluate (positional) |

## Options

### Input/Output

| Option | Description |
|--------|-------------|
| `-e, --expression <EXPR>` | Expression(s) to evaluate (can be repeated) |
| `-Q, --query-file <FILE>` | Read JMESPath expression from file |
| `-f, --file <FILE>` | Input JSON file (reads stdin if not provided) |
| `-o, --output <FILE>` | Output file (writes to stdout if not provided) |
| `-n, --null-input` | Don't read input, use null as input value |
| `-s, --slurp` | Read all inputs into an array |

### Output Format

| Option | Description |
|--------|-------------|
| `-r, --raw` | Output raw strings without quotes |
| `-c, --compact` | Compact output (no pretty printing) |
| `--color <MODE>` | Colorize output: `auto`, `always`, `never` |

### Modes

| Option | Description |
|--------|-------------|
| `--strict` | Strict mode - only standard JMESPath functions |
| `-v, --verbose` | Show expression details and timing |
| `-q, --quiet` | Suppress errors and warnings |

### Function Discovery

| Option | Description |
|--------|-------------|
| `--list-functions` | List all available functions |
| `--list-category <NAME>` | List functions in a specific category |
| `--describe <FUNCTION>` | Show detailed info for a function |

### Other

| Option | Description |
|--------|-------------|
| `--completions <SHELL>` | Generate shell completions |
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

```bash
# Basic query from stdin
echo '{"name": "Alice"}' | jpx 'name'

# Query a file
jpx 'users[*].email' -f data.json

# Raw output for scripting
jpx -r 'config.api_key' -f settings.json

# Chain multiple expressions
jpx -e 'users' -e '[?active]' -e '[*].name' -f data.json

# Use null input for functions that don't need data
jpx -n 'now()'

# Slurp multiple JSON objects
cat *.json | jpx -s 'length(@)'

# List string functions
jpx --list-category string

# Get function documentation
jpx --describe median

# Strict mode (standard JMESPath only)
jpx --strict 'length(items)' -f data.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid expression, file not found, etc.) |
