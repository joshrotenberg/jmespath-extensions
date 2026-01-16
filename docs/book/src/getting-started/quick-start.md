# Quick Start

This guide will get you up and running with jpx in just a few minutes.

## Your First Query

Let's start with a simple JSON object:

```bash
echo '{"name": "Alice", "age": 30}' | jpx 'name'
```

Output:
```
"Alice"
```

## Querying Arrays

Access array elements and properties:

```bash
echo '{"users": [{"name": "Alice"}, {"name": "Bob"}]}' | jpx 'users[0].name'
```

Output:
```
"Alice"
```

Get all names:

```bash
echo '{"users": [{"name": "Alice"}, {"name": "Bob"}]}' | jpx 'users[*].name'
```

Output:
```
["Alice", "Bob"]
```

## Using Extension Functions

Here's where jpx shines. Use any of the 400+ extension functions:

```bash
# String manipulation
echo '{"name": "hello world"}' | jpx 'upper(name)'
# "HELLO WORLD"

# Array operations
echo '{"nums": [3, 1, 4, 1, 5, 9, 2, 6]}' | jpx 'unique(nums) | sort(@)'
# [1, 2, 3, 4, 5, 6, 9]

# Math functions
echo '{"values": [10, 20, 30, 40, 50]}' | jpx 'avg(values)'
# 30

# Current timestamp
echo '{}' | jpx 'now()'
# 1705312200
```

## Raw Output

Use `-r` to output strings without quotes:

```bash
echo '{"greeting": "Hello, World!"}' | jpx -r 'greeting'
```

Output:
```
Hello, World!
```

## Reading from Files

Query a JSON file directly:

```bash
jpx 'users[*].email' -f data.json
```

## Piping and Chaining

Chain multiple expressions with the pipe operator:

```bash
echo '{"items": ["apple", "banana", "cherry"]}' | jpx 'items | [0]'
# "apple"

echo '{"name": "john doe"}' | jpx 'name | upper(@) | split(@, ` `)'
# ["JOHN", "DOE"]
```

## Function Discovery

Find functions by category:

```bash
jpx --list-category string
```

Get details about a specific function:

```bash
jpx --describe upper
```

## Next Steps

- [Basic Usage](./basic-usage.md) - Learn more CLI options
- [CLI Reference](../guide/cli-reference.md) - Complete CLI documentation
- [Function Reference](../functions/overview.md) - Browse all 400+ functions
