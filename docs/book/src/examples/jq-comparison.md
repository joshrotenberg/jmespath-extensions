# jq vs jpx Comparison

If you're coming from jq, this guide will help you understand the differences and similarities between jq and jpx (JMESPath with extensions).

## Philosophy

| Aspect | jq | jpx |
|--------|-----|-----|
| **Query Language** | Custom DSL | JMESPath (standardized) |
| **Focus** | Turing-complete transformation | Query-focused with functions |
| **Learning Curve** | Steeper (unique syntax) | Gentler (more declarative) |
| **Ecosystem** | Standalone | JMESPath implementations in many languages |

## Syntax Comparison

### Basic Field Access

```bash
# jq
echo '{"name": "alice"}' | jq '.name'

# jpx
echo '{"name": "alice"}' | jpx 'name'
```

### Array Indexing

```bash
# jq
echo '[1, 2, 3]' | jq '.[0]'

# jpx
echo '[1, 2, 3]' | jpx '[0]'
```

### Array Projection (Map)

```bash
# jq - extract field from each object
echo '[{"name": "a"}, {"name": "b"}]' | jq '.[].name'
echo '[{"name": "a"}, {"name": "b"}]' | jq '[.[] | .name]'

# jpx - cleaner syntax
echo '[{"name": "a"}, {"name": "b"}]' | jpx '[*].name'
```

### Filtering

```bash
# jq
echo '[{"age": 25}, {"age": 35}]' | jq '[.[] | select(.age > 30)]'

# jpx - filter expression built into syntax
echo '[{"age": 25}, {"age": 35}]' | jpx '[?age > `30`]'
```

### Pipelines

```bash
# jq - pipe operator
echo '[3, 1, 2]' | jq 'sort | first'

# jpx - multiple expressions as pipeline
echo '[3, 1, 2]' | jpx 'sort(@)' 'first(@)'
# or with -e flags
echo '[3, 1, 2]' | jpx -e 'sort(@)' -e 'first(@)'
```

### Object Construction

```bash
# jq
echo '{"first": "John", "last": "Doe"}' | jq '{fullName: (.first + " " + .last)}'

# jpx - multi-select hash
echo '{"first": "John", "last": "Doe"}' | jpx '{fullName: join(` `, [first, last])}'
```

## Function Comparison

### String Functions

| Operation | jq | jpx |
|-----------|-----|-----|
| Uppercase | `ascii_upcase` | `upper(@)` |
| Lowercase | `ascii_downcase` | `lower(@)` |
| Split | `split(",")` | `split(@, ',')` |
| Join | `join(",")` | `join(',', @)` |
| Trim | `ltrimstr`, `rtrimstr` | `trim(@)` |
| Substring | `.[0:5]` | `substr(@, 0, 5)` |
| Contains | `contains("x")` | `contains(@, 'x')` |
| Replace | `gsub("a"; "b")` | `replace(@, 'a', 'b')` |

### Array Functions

| Operation | jq | jpx |
|-----------|-----|-----|
| Length | `length` | `length(@)` |
| First | `first` | `first(@)` |
| Last | `last` | `last(@)` |
| Reverse | `reverse` | `reverse(@)` |
| Sort | `sort` | `sort(@)` |
| Unique | `unique` | `unique(@)` |
| Flatten | `flatten` | `flatten(@)` |
| Group by | `group_by(.key)` | `group_by(@, 'key')` |
| Index of | `index(x)` | `find_index(@, x)` |
| All indices | `indices(x)` | `indices_array(@, x)` |

### Math Functions

| Operation | jq | jpx |
|-----------|-----|-----|
| Sum | `add` | `sum(@)` |
| Min | `min` | `min(@)` |
| Max | `max` | `max(@)` |
| Average | N/A (manual) | `avg(@)` |
| Floor | `floor` | `floor(@)` |
| Ceil | `ceil` | `ceil(@)` |
| Round | N/A (manual) | `round(@, 2)` |
| Absolute | `fabs` | `abs(@)` |

### Object Functions

| Operation | jq | jpx |
|-----------|-----|-----|
| Keys | `keys` | `keys(@)` |
| Values | `values` | `values(@)` |
| Has key | `has("key")` | `has(@, 'key')` |
| Entries | `to_entries` | `items(@)` |
| From entries | `from_entries` | `from_items(@)` |
| With entries | `with_entries(...)` | `with_entries(@, &...)` |
| Merge | `* ` or `+` | `merge(@, other)` |

### Control Flow

| Operation | jq | jpx |
|-----------|-----|-----|
| Recurse | `recurse` | `recurse(@)` |
| Recurse with expr | `recurse(f)` | `recurse_with(@, &f)` |
| While | `while(cond; update)` | `while_expr(init, &cond, &update)` |
| Until | `until(cond; update)` | `until_expr(init, &cond, &update)` |

## jpx Advantages

### 1. Extended Function Library (250+)
jpx includes functions that jq doesn't have:

```bash
# Date/time operations
jpx -n 'now()'
jpx -n 'format_date(now(), "%Y-%m-%d")'

# Hashing
echo '"hello"' | jpx 'sha256(@)'

# Network utilities  
echo '"192.168.1.0/24"' | jpx 'cidr_contains(@, "192.168.1.100")'

# Fuzzy matching
echo '["hello", "hallo", "world"]' | jpx '[*].{word: @, dist: levenshtein(@, "hello")}'

# UUID/ULID generation
jpx -n 'ulid()'

# Semantic versioning
echo '"1.2.3"' | jpx 'semver_parse(@)'
```

### 2. Built-in JSON Patch Support

```bash
# Generate diff between two files
jpx --diff old.json new.json

# Apply JSON Patch
jpx --patch changes.json -f document.json

# Apply JSON Merge Patch
jpx --merge updates.json -f document.json
```

### 3. MCP Server for AI Integration
jpx can run as an MCP server for integration with Claude and other AI assistants:

```bash
jpx mcp
```

### 4. Function Discovery

```bash
# Search for functions
jpx --search "date"

# Get function details
jpx --describe format_date

# List by category
jpx --list-category datetime
```

### 5. Expression Explanation

```bash
jpx --explain '[*].name | sort(@)'
```

## jq Advantages

### 1. More Powerful Transformations
jq is Turing-complete and supports:
- Recursive functions
- Variable bindings with `as`
- More complex conditionals
- Custom function definitions

### 2. Streaming Parser
jq can process very large files with `--stream`.

### 3. Wider Adoption
jq has been around longer and has more community resources.

## Migration Tips

1. **Start simple**: Basic queries translate almost directly
2. **Functions need `@`**: In jpx, you typically pass the current value explicitly: `length(@)` instead of `length`
3. **Backticks for literals**: Use `` `5` `` instead of `5` for literal numbers in expressions
4. **Filter syntax**: Use `[?condition]` instead of `select()`
5. **Projections**: Use `[*].field` instead of `.[].field`

## When to Use Which

**Choose jpx when:**
- You want a standardized query language (JMESPath)
- You need the extended function library
- You're integrating with systems that use JMESPath (AWS CLI, etc.)
- You want AI assistant integration via MCP

**Choose jq when:**
- You need complex recursive transformations
- You're working with streaming data
- You need to define custom functions
- Your team already knows jq well
