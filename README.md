# jmespath_extensions

[![Crates.io](https://img.shields.io/crates/v/jmespath_extensions.svg)](https://crates.io/crates/jmespath_extensions)
[![Documentation](https://docs.rs/jmespath_extensions/badge.svg)](https://docs.rs/jmespath_extensions)
[![License](https://img.shields.io/crates/l/jmespath_extensions.svg)](https://github.com/joshrotenberg/jmespath-extensions#license)
[![CI](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml)

Extended functions for JMESPath queries in Rust. **320+ functions** for strings, arrays, dates, hashing, encoding, and more.

## MCP Server for AI Assistants

Use jpx as an [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server to give AI assistants like Claude the ability to query and transform JSON data.

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx",
      "args": ["mcp"]
    }
  }
}
```

**12 tools available:** `evaluate`, `evaluate_file`, `batch_evaluate`, `functions`, `describe`, `categories`, `validate`, `format`, `diff`, `patch`, `merge`, `keys`

See the [jpx README](jpx/README.md) for details.

---

## Acknowledgments

### JMESPath

[JMESPath](https://jmespath.org/) is a query language for JSON, created by [James Saryerwinnie](https://github.com/jamesls). The specification, tutorials, and compliance test suite are maintained by the JMESPath community. We're grateful for this well-designed, portable query language that serves as our foundation.

- **Specification**: [jmespath.org](https://jmespath.org/)
- **GitHub**: [jmespath/jmespath.spec](https://github.com/jmespath/jmespath.spec)
- **Community**: [jmespath-community](https://github.com/jmespath-community)

### jmespath.rs

This crate extends the [`jmespath`](https://crates.io/crates/jmespath) crate by [@mtdowling](https://github.com/mtdowling), which provides the complete Rust implementation of the [JMESPath specification](https://jmespath.org/specification.html). All spec-compliant parsing, evaluation, and the 26 built-in functions come from that foundational library—we simply add extra functions on top.

**If you only need standard JMESPath functionality, use [`jmespath`](https://crates.io/crates/jmespath) directly.**

### jp - The Official JMESPath CLI

The official [jp](https://github.com/jmespath/jp) CLI tool (written in Go) is a minimal, focused implementation of a JMESPath command-line interface. If you need a lightweight tool that works with standard JMESPath and don't require the extended functions that jpx provides, jp is an excellent choice.

```bash
# Install jp via Homebrew
brew install jmespath/jmespath/jp
```

jpx is inspired by jp's simplicity while extending it with 320+ additional functions and features like multiple output formats, expression pipelines, and interactive REPL mode.

---

## jpx vs jq

[jq](https://jqlang.org/) is the most popular JSON command-line tool. Here's how jpx compares:

| Aspect | jq | jpx |
|--------|-----|-----|
| **Language** | Custom DSL (Turing-complete) | JMESPath (standardized query language) |
| **Learning curve** | Steeper (unique syntax) | Gentler (declarative, function-based) |
| **Functions** | ~70 built-in | 320+ (string, math, date, geo, hash, etc.) |
| **Ecosystem** | Standalone | JMESPath works in AWS CLI, Ansible, many languages |
| **Streaming** | Yes (`--stream`) | No (loads full document) |
| **Custom functions** | Yes (`def`) | No (fixed function set) |

### Syntax Comparison

```bash
# Filtering - jq uses select(), jpx uses [?]
jq '[.[] | select(.age > 30)]' data.json
jpx '[?age > `30`]' data.json

# Projection - similar but jpx uses [*]
jq '[.[].name]' data.json
jpx '[*].name' data.json

# String manipulation
jq '.name | ascii_upcase' data.json
jpx 'upper(name)' data.json
```

### When to Choose jpx

- **Extended functions**: 320+ functions including geo, hashing, fuzzy matching, semver, validation
- **JMESPath compatibility**: Queries work in AWS CLI (`--query`), Ansible, and other tools
- **Multiple output formats**: JSON, YAML, TOML, CSV, TSV, table
- **AI integration**: MCP server for Claude and other assistants
- **Function discovery**: `--search`, `--describe`, `--similar` to explore available functions

### When to Choose jq

- **Complex transformations**: Recursive functions, variable bindings, custom function definitions
- **Streaming**: Process very large files without loading into memory
- **Team familiarity**: If your team already knows jq well

**[Full comparison guide →](https://joshrotenberg.github.io/jmespath-extensions/examples/jq-comparison.html)**

---

**Want to try it out?** Install the `jpx` CLI tool:

```bash
# Homebrew (macOS/Linux)
brew install joshrotenberg/brew/jpx

# Or cargo
cargo install jpx

# Then use it!
echo '{"name": "world"}' | jpx 'upper(name)'
# "WORLD"
```

> **Non-Standard Extensions - Not Portable**
>
> This crate provides **custom extension functions** that are **NOT part of the [JMESPath specification](https://jmespath.org/specification.html)**.
> Queries using these functions will **NOT work** in other JMESPath implementations (Python, JavaScript, Go, AWS CLI, Ansible, etc.).

## JMESPath Spec vs This Library

| | **JMESPath Specification** | **jmespath_extensions** |
|---|---|---|
| **Functions** | 26 built-in functions | 320+ extension functions |
| **Portability** | Works everywhere (Python, JS, Go, AWS CLI, Ansible) | Rust only |
| **Design** | Minimal, query-focused | Transformation-heavy, practical |
| **Governance** | JEP process, multi-year consensus | Opinionated, can change |
| **Philosophy** | "Spec purity" | "Useful > Pure" |

### What This Means

1. **Not portable**: Queries using `upper()`, `map_expr()`, `haversine()`, etc. won't work in AWS CLI's `--query`, Ansible filters, or any other JMESPath implementation.

2. **No spec backing**: Function names, signatures, and behaviors are our decisions. While we align with JEPs where possible (`items`, `find_first`), many functions are novel.

3. **Expression functions are unique**: `map_expr`, `filter_expr`, `sort_by_expr` etc. leverage Rust runtime access—these don't exist in any JMESPath spec or implementation.

### Use Cases

This library is ideal for:

- **Backend data transformation**: Reshape API responses, filter datasets, compute derived fields
- **Configuration processing**: Query and transform JSON/YAML configs with complex logic
- **ETL pipelines**: Extract, transform, and validate data with expressive queries
- **Log/event processing**: Filter and aggregate structured log data
- **CLI tools**: Build jq-like tools with domain-specific functions
- **Embedded queries**: Let users write safe, sandboxed data queries in your application

### For Portable Queries

Use only the [26 standard JMESPath built-in functions](https://jmespath.org/specification.html#built-in-functions):
`abs`, `avg`, `ceil`, `contains`, `ends_with`, `floor`, `join`, `keys`, `length`, `map`, `max`, `max_by`, `merge`, `min`, `min_by`, `not_null`, `reverse`, `sort`, `sort_by`, `starts_with`, `sum`, `to_array`, `to_number`, `to_string`, `type`, `values`

## Overview

This crate provides 320+ additional functions beyond the standard JMESPath built-ins, organized into feature-gated categories.

**[Full API Documentation →](https://docs.rs/jmespath_extensions)**

## Quick Start

```rust
use jmespath::Runtime;
use jmespath_extensions::register_all;

let mut runtime = Runtime::new();
runtime.register_builtin_functions();
register_all(&mut runtime);

// Now you can use extended functions in queries
let expr = runtime.compile("items[*].name | upper(@)").unwrap();
```

## Runtime Function Registry

For applications that need runtime control over function availability (ACLs, config-based gating, introspection):

```rust
use jmespath::Runtime;
use jmespath_extensions::registry::{FunctionRegistry, Category};

let mut registry = FunctionRegistry::new();

// Register specific categories
registry.register_category(Category::String);
registry.register_category(Category::Math);

// Or register all available functions
// registry.register_all();

// Disable specific functions (e.g., for security policies)
registry.disable_function("md5");
registry.disable_function("sha256");

// Apply to runtime
let mut runtime = Runtime::new();
runtime.register_builtin_functions();
registry.apply(&mut runtime);

// Introspection - list available functions
for func in registry.functions() {
    let type_label = if func.is_standard { "standard" } else { "extension" };
    println!("[{}] {}: {}", type_label, func.name, func.description);
}
```

This enables:
- **Runtime gating**: Enable/disable functions via config instead of compile-time features
- **ACL support**: Disable specific functions for security policies
- **Introspection**: Query available functions with signatures, descriptions, examples, and whether they are standard JMESPath or extensions

## jpx CLI

See [jpx/README.md](jpx/README.md) for full CLI documentation, or use `jpx --help`.

```bash
# Expression functions (the novel stuff!)
echo '{"users": [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]}' \
  | jpx 'filter_expr(users, &age > `26`) | [].name'
# ["alice"]

# Strict mode - only standard JMESPath functions
echo '[1, 2, 3]' | jpx --strict 'length(@)'
# 3

# Function discovery
jpx --list-functions           # List all 320+ functions
jpx --list-category expression # List expression functions
jpx --describe map_expr        # Detailed function info
```

## Features

All features are opt-in. Use `default-features = false` to select only what you need.

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `full` (default) | All functions | All below |
| `core` | Essential functions, no external deps | None |
| **Core Modules** | | |
| `string` | `upper`, `lower`, `split`, `replace`, `camel_case`, etc. | None |
| `array` | `first`, `last`, `unique`, `chunk`, `zip`, `range`, etc. | None |
| `object` | `items`, `pick`, `omit`, `deep_merge`, `flatten_keys`, etc. | None |
| `math` | `round`, `sqrt`, `median`, `stddev`, `sin`, `cos`, etc. | None |
| `type` | `type_of`, `is_string`, `is_empty`, `to_number`, etc. | None |
| `utility` | `default`, `if`, `coalesce`, `now`, `now_ms`, etc. | None |
| `validation` | `is_email`, `is_url`, `is_uuid`, `is_ipv4`, `is_ipv6` | None |
| `path` | `path_basename`, `path_dirname`, `path_ext`, `path_join` | None |
| `expression` | `map_expr`, `filter_expr`, `sort_by_expr`, `group_by_expr`, etc. | None |
| `text` | `word_count`, `reading_time`, `word_frequencies`, etc. | None |
| **External Deps** | | |
| `hash` | `md5`, `sha1`, `sha256`, `crc32` | md-5, sha1, sha2, crc32fast |
| `encoding` | `base64_encode`, `base64_decode`, `hex_encode`, `hex_decode` | base64, hex |
| `regex` | `regex_match`, `regex_extract`, `regex_replace` | regex |
| `url` | `url_encode`, `url_decode`, `url_parse` | url, urlencoding |
| `uuid` | `uuid` (v4 generation) | uuid |
| `rand` | `random`, `shuffle`, `sample` | rand |
| `datetime` | `parse_date`, `format_date`, `date_add`, `date_diff` | chrono |
| `fuzzy` | `levenshtein`, `jaro_winkler`, `sorensen_dice`, etc. | strsim |
| `phonetic` | `soundex`, `metaphone`, `double_metaphone`, `nysiis`, etc. | rphonetic |
| `language` | `detect_language`, `detect_language_iso`, `detect_script`, `detect_language_info` | whatlang |
| `geo` | `geo_distance`, `geo_distance_km`, `geo_distance_miles`, `geo_bearing` | geoutils |
| `semver` | `semver_parse`, `semver_compare`, `semver_satisfies`, etc. | semver |
| `network` | `ip_to_int`, `cidr_contains`, `cidr_network`, `is_private_ip` | ipnetwork |
| `ids` | `nanoid`, `ulid`, `ulid_timestamp` | nanoid, ulid |
| `duration` | `parse_duration`, `format_duration`, etc. | None |
| `color` | `hex_to_rgb`, `rgb_to_hex`, `lighten`, `darken`, etc. | None |
| `computing` | `parse_bytes`, `format_bytes`, `bit_and`, `bit_or`, etc. | None |
| `jsonpatch` | `json_patch`, `json_merge_patch`, `json_diff` (RFC 6902/7396) | json-patch |
| `multi-match` | `match_any`, `match_all`, `match_which`, `match_count`, `replace_many` | aho-corasick |

### Minimal Dependencies

```toml
[dependencies]
jmespath_extensions = { version = "0.2", default-features = false, features = ["core"] }
```

### Specific Features

```toml
[dependencies]
jmespath_extensions = { version = "0.2", default-features = false, features = ["string", "array", "datetime"] }
```

## Examples

### String Manipulation

```
upper('hello')                    → "HELLO"
split('a,b,c', ',')               → ["a", "b", "c"]
camel_case('hello_world')         → "helloWorld"
```

### Array Operations

```
first([1, 2, 3])                  → 1
unique([1, 2, 1, 3])              → [1, 2, 3]
chunk([1, 2, 3, 4], `2`)          → [[1, 2], [3, 4]]
```

### Expression Functions

```
map_expr('name', users)           → ["alice", "bob"]
filter_expr('age >= `18`', users) → [{...}, {...}]
sort_by_expr('score', items)      → [{score: 1}, {score: 2}, ...]
```

### Date/Time

```
now()                             → 1699900000
format_date(`0`, '%Y-%m-%d')      → "1970-01-01"
date_add(`0`, `1`, 'days')        → 86400
```

### Fuzzy Matching

```
levenshtein('kitten', 'sitting')  → 3
jaro_winkler('hello', 'hallo')    → 0.88
sounds_like('Robert', 'Rupert')   → true
```

### Geospatial

```
geo_distance_km(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) → 5570.2
geo_bearing(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`)     → 51.2
```

### Network

```
cidr_contains('192.168.0.0/16', '192.168.1.1') → true
is_private_ip('10.0.0.1')                      → true
```

### JSON Patch (RFC 6902/7396)

```
json_patch({a: 1}, [{op: 'add', path: '/b', value: 2}])  → {a: 1, b: 2}
json_merge_patch({a: 1}, {b: 2})                        → {a: 1, b: 2}
json_diff({a: 1}, {a: 2})                               → [{op: 'replace', path: '/a', value: 2}]
```

### Multi-Pattern Matching (Aho-Corasick)

```
match_any('hello world', ['world', 'foo'])              → true
match_all('hello world', ['hello', 'world'])            → true
match_which('hello world', ['hello', 'foo', 'world'])   → ["hello", "world"]
match_count('abcabc', ['a', 'b'])                       → 4
replace_many('hello world', {hello: 'hi', world: 'earth'}) → "hi earth"
```

### Safe Path Navigation

```
get({a: {b: 1}}, 'a.b')                       → 1
get({a: 1}, 'x.y.z', 'default')               → "default"
has({a: {b: 1}}, 'a.b')                       → true
has({a: 1}, 'x.y')                            → false
set_path({a: 1}, '/b', `2`)                   → {a: 1, b: 2}
delete_path({a: 1, b: 2}, '/b')               → {a: 1}
```

### Array Indexing & Lookups

```
index_by([{id: 1, name: 'alice'}], 'id')      → {"1": {id: 1, name: "alice"}}
index_at([1, 2, 3], `-1`)                     → 3
```

### Data Cleanup

```
remove_nulls({a: 1, b: null})                 → {a: 1}
remove_empty({a: '', b: [], c: 'x'})          → {c: "x"}
remove_empty_strings({a: '', b: 'hi'})        → {b: "hi"}
compact([1, null, 2, null])                   → [1, 2]
```

### Data Quality & Redaction

```
data_quality_score({a: null, b: ''}).score    → 50
mask('4111111111111111')                      → "************1111"
mask('555-1234', `3`)                         → "****234"
redact({pass: 'x', name: 'y'}, ['pass'])      → {pass: "[REDACTED]", name: "y"}
redact_keys({api_key: 'x'}, 'api.*')          → {api_key: "[REDACTED]"}
```

### Key Transformation & Search

```
camel_keys({user_name: 'alice'})              → {userName: "alice"}
snake_keys({userName: 'bob'})                 → {user_name: "bob"}
pluck_deep({a: {id: 1}, b: {id: 2}}, 'id')    → [1, 2]
paths_to({a: {id: 1}, b: {id: 2}}, 'id')      → ["a.id", "b.id"]
```

### Statistical Analysis

```
quartiles([1, 2, 3, 4, 5])                    → {min: 1, q1: 2, q2: 3, q3: 4, max: 5, iqr: 2}
outliers_iqr([1, 2, 3, 4, 100])               → [100]
outliers_zscore([1, 2, 3, 4, 100])            → [100]
percentile([1, 2, 3, 4, 5], `75`)             → 4
```

### Language Detection

```
detect_language('Hello world')                → "English"
detect_language('Bonjour le monde')           → "Français"
detect_language_iso('Hola mundo')             → "spa"
detect_script('Привет мир')                   → "Cyrillic"
detect_language_info('Test').confidence       → 0.95
```

See the [API documentation](https://docs.rs/jmespath_extensions) for complete function reference with examples.

## JMESPath Community JEP Alignment

This crate aligns with several [JMESPath Enhancement Proposals](https://github.com/jmespath-community/jmespath.spec):

- **JEP-014** (String Functions): `lower`, `upper`, `trim`, `trim_left`, `trim_right`, `pad_left`, `pad_right`, `replace`, `split`, `find_first`, `find_last`
- **JEP-013** (Object Functions): `items`, `from_items`, `zip`

Functions that align with JEPs have `jep: Some("JEP-XXX")` in their `FunctionInfo` metadata, accessible via the registry.

Additional functions extend well beyond these proposals. Some JEPs (like arithmetic operators) require parser changes and cannot be implemented as extension functions.

## Development

### Adding a New Function

1. **Add metadata to `jmespath_extensions/functions.toml`**:
   ```toml
   [[functions]]
   name = "my_function"
   category = "string"  # Must match an existing category
   description = "Brief description of what it does"
   signature = "string, number -> string"
   example = "my_function('hello', `3`) -> 'hellohellohello'"
   features = ["string"]  # Feature flags that enable this function
   # Optional fields:
   # jep = "JEP-014"      # If aligned with a JMESPath Enhancement Proposal
   # aliases = ["my_func"] # Alternative names
   ```

2. **Implement the function** in the appropriate module (e.g., `src/string.rs`):
   ```rust
   define_function! {
       name = "my_function";
       doc = "Brief description of what it does.";
       args = [value: Value, count: usize];
       fn run(value, count) {
           // Implementation
           let s = value.as_string().ok_or_else(|| /* error */)?;
           Ok(Rc::new(Variable::String(s.repeat(count))))
       }
   }
   ```

3. **Register the function** in the module's `register_*` function:
   ```rust
   pub fn register_string(runtime: &mut Runtime) {
       // ... existing registrations
       runtime.register_function(Box::new(MyFunction));
   }
   ```

4. **Run the build** to regenerate documentation:
   ```bash
   cargo build  # build.rs generates docs from functions.toml
   cargo test   # Verify everything works
   cargo clippy --all-features
   ```

### Adding a New Feature/Category

1. **Add the feature to `Cargo.toml`**:
   ```toml
   [features]
   my_category = ["dep:some-crate"]  # If it needs dependencies
   full = ["my_category", ...]       # Add to full feature
   ```

2. **Create the module** `src/my_category.rs` with the standard structure

3. **Add to `src/lib.rs`**:
   ```rust
   #[cfg(feature = "my_category")]
   pub mod my_category;
   ```

4. **Add the category variant** to `Category` enum in `src/registry.rs`

5. **Update `build.rs`** to handle the new category in `category_variant()`

### Contribution Guidelines

When proposing new functions, please ensure they meet these criteria:

1. **Generally useful**: Functions should solve common problems that many users encounter. Avoid highly specialized or niche use cases.

2. **Minimal dependencies**: Prefer zero dependencies. If a dependency is needed, limit to one well-maintained crate. Dependencies should be feature-gated.

3. **Thoughtful naming**: Function names should be:
   - Clear and descriptive
   - General enough to not imply overly specific behavior
   - Consistent with existing naming conventions in the category

4. **Fit existing categories**: New functions should naturally belong to an existing feature/category. Proposing a new category requires strong justification and multiple related functions.

5. **No duplicate functionality**: Check that the function doesn't duplicate existing functionality or can be trivially composed from existing functions.

6. **Include tests and examples**: All new functions must include tests and a working example in `functions.toml`.

## Benchmarks

Run benchmarks with:

```bash
cargo bench --all-features
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
