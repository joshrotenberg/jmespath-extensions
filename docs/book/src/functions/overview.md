# Function Overview

jpx provides over 320 functions organized into categories.

## Discovering Functions

### List All Functions

```bash
jpx --list-functions
```

### List by Category

```bash
jpx --list-category string
jpx --list-category math
jpx --list-category datetime
```

### Get Function Details

```bash
jpx --describe upper
```

Output:
```
STRING functions:

  upper - Convert string to uppercase
    Signature: string -> string
    Example: upper('hello') -> "HELLO"
```

## Categories

| Category | Description | Count |
|----------|-------------|-------|
| [Standard](./standard.md) | Built-in JMESPath functions | 26 |
| [String](./string.md) | String manipulation | 40+ |
| [Array](./array.md) | Array operations | 30+ |
| [Object](./object.md) | Object manipulation | 25+ |
| [Math](./math.md) | Math and statistics | 30+ |
| [DateTime](./datetime.md) | Date and time | 15+ |
| [Hash & Encoding](./hash-encoding.md) | Hashing and encoding | 20+ |
| [Validation](./validation.md) | Data validation | 10+ |
| [Expression](./expression.md) | Higher-order functions | 10+ |
| [Other](./other.md) | Geo, fuzzy, network, etc. | 100+ |

## Function Syntax

Functions are called with parentheses:

```bash
function_name(arg1, arg2, ...)
```

### Examples

```bash
# No arguments
echo '{}' | jpx 'now()'

# One argument
echo '{"name": "hello"}' | jpx 'upper(name)'

# Multiple arguments
echo '{"text": "hello world"}' | jpx 'split(text, ` `)'

# Literal arguments (use backticks)
echo '{}' | jpx 'range(`1`, `10`)'
```

## Standard vs Extension Functions

### Standard Functions (26)

These are part of the JMESPath specification and work in all implementations:

`abs`, `avg`, `ceil`, `contains`, `ends_with`, `floor`, `join`, `keys`, `length`, `map`, `max`, `max_by`, `merge`, `min`, `min_by`, `not_null`, `reverse`, `sort`, `sort_by`, `starts_with`, `sum`, `to_array`, `to_number`, `to_string`, `type`, `values`

### Extension Functions (300+)

These are jpx-specific and won't work in other JMESPath implementations:

`upper`, `lower`, `split`, `unique`, `chunk`, `median`, `now`, `uuid`, `md5`, and many more.

### Strict Mode

Use `--strict` to disable extension functions:

```bash
# This works
jpx --strict 'length(items)' -f data.json

# This fails (upper is an extension)
jpx --strict 'upper(name)' -f data.json
```
