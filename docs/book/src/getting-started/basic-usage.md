# Basic Usage

## Input Sources

jpx can read JSON from multiple sources:

### Standard Input (stdin)

```bash
echo '{"name": "Alice"}' | jpx 'name'
cat data.json | jpx 'users[*].name'
curl -s https://api.example.com/data | jpx 'results[0]'
```

### File Input

```bash
jpx 'users[*].name' -f data.json
jpx --file users.json 'length(@)'
```

### Null Input

Use `-n` to start with null (useful for functions that don't need input):

```bash
jpx -n 'now()'
jpx -n 'uuid()'
jpx -n 'range(`1`, `10`)'
```

## Output Options

### Pretty Printing (default)

By default, jpx pretty-prints JSON output with colors:

```bash
echo '{"a":1,"b":2}' | jpx '@'
```

Output:
```json
{
  "a": 1,
  "b": 2
}
```

### Compact Output

Use `-c` for single-line output:

```bash
echo '{"a":1,"b":2}' | jpx -c '@'
```

Output:
```json
{"a":1,"b":2}
```

### Raw Strings

Use `-r` to output strings without quotes:

```bash
echo '{"msg": "hello"}' | jpx -r 'msg'
```

Output:
```
hello
```

### Output to File

Write results to a file:

```bash
jpx 'users[*].email' -f data.json -o emails.json
```

## Slurp Mode

Read multiple JSON objects into an array:

```bash
echo '{"a":1}
{"b":2}
{"c":3}' | jpx -s 'length(@)'
```

Output:
```
3
```

## Expression as Positional Argument

The expression can be the first positional argument:

```bash
jpx 'users[*].name' -f data.json
```

Or use `-e` / `--expression`:

```bash
jpx -e 'users[*].name' -f data.json
```

## Chaining Expressions

Chain multiple expressions (applied sequentially):

```bash
jpx -e 'users' -e '[*].name' -e 'sort(@)' -f data.json
```

## Verbose Mode

See expression details and timing:

```bash
echo '{"x": 1}' | jpx -v 'x'
```

## Quiet Mode

Suppress errors and warnings:

```bash
jpx -q 'invalid[' -f data.json  # Won't show error output
```

## Color Control

Control colorized output:

```bash
jpx --color=always 'name' -f data.json  # Force colors
jpx --color=never 'name' -f data.json   # No colors
jpx --color=auto 'name' -f data.json    # Auto-detect (default)
```
