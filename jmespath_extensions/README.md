# jmespath_extensions

[![Crates.io](https://img.shields.io/crates/v/jmespath_extensions.svg)](https://crates.io/crates/jmespath_extensions)
[![Documentation](https://docs.rs/jmespath_extensions/badge.svg)](https://docs.rs/jmespath_extensions)
[![License](https://img.shields.io/crates/l/jmespath_extensions.svg)](https://github.com/joshrotenberg/jmespath-extensions#license)
[![CI](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml)

Extended functions for JMESPath queries in Rust.

## Built on jmespath.rs

This crate extends the [`jmespath`](https://crates.io/crates/jmespath) crate by [@mtdowling](https://github.com/mtdowling), which provides the complete Rust implementation of the [JMESPath specification](https://jmespath.org/specification.html). All spec-compliant parsing, evaluation, and the 26 built-in functions come from that foundational library—we simply add extra functions on top.

**If you only need standard JMESPath functionality, use [`jmespath`](https://crates.io/crates/jmespath) directly.**

> **Non-Standard Extensions - Not Portable**
>
> This crate provides **custom extension functions** that are **NOT part of the [JMESPath specification](https://jmespath.org/specification.html)**.
> Queries using these functions will **NOT work** in other JMESPath implementations (Python, JavaScript, Go, AWS CLI, Ansible, etc.).

## JMESPath Spec vs This Library

| | **JMESPath Specification** | **jmespath_extensions** |
|---|---|---|
| **Functions** | 26 built-in functions | 189 extension functions |
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

This crate provides 189 additional functions beyond the standard JMESPath built-ins, organized into feature-gated categories.

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

## CLI Tool: jpx

The `jpx` CLI tool lets you experiment with all functions from the command line:

```bash
# Install via Homebrew (macOS/Linux)
brew tap joshrotenberg/brew
brew install jpx

# Install from crates.io
cargo install jpx

# Install pre-built binaries (macOS, Linux, Windows)
# See https://github.com/joshrotenberg/jmespath-extensions/releases
```

```bash
# String functions
echo '{"name": "hello"}' | jpx 'upper(name)'
# "HELLO"

# Expression functions (the novel stuff!)
echo '{"users": [{"name": "alice", "age": 30}, {"name": "bob", "age": 25}]}' \
  | jpx 'filter_expr(users, &age > `26`) | [].name'
# ["alice"]

# Duration parsing
echo '{"d": "1h30m"}' | jpx 'parse_duration(d)'
# 5400.0

# Strict mode - only standard JMESPath functions
echo '[1, 2, 3]' | jpx --strict 'length(@)'
# 3

# Function discovery
jpx --list-functions           # List all 189+ functions
jpx --list-category expression # List expression functions
jpx --describe map_expr        # Detailed function info
```

See [jpx/README.md](jpx/README.md) for full documentation.

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
| `geo` | `geo_distance`, `geo_distance_km`, `geo_distance_miles`, `geo_bearing` | geoutils |
| `semver` | `semver_parse`, `semver_compare`, `semver_satisfies`, etc. | semver |
| `network` | `ip_to_int`, `cidr_contains`, `cidr_network`, `is_private_ip` | ipnetwork |
| `ids` | `nanoid`, `ulid`, `ulid_timestamp` | nanoid, ulid |
| `duration` | `parse_duration`, `format_duration`, etc. | None |
| `color` | `hex_to_rgb`, `rgb_to_hex`, `lighten`, `darken`, etc. | None |
| `computing` | `parse_bytes`, `format_bytes`, `bit_and`, `bit_or`, etc. | None |

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

See the [API documentation](https://docs.rs/jmespath_extensions) for complete function reference with examples.

## JMESPath Community JEP Alignment

This crate aligns with several [JMESPath Enhancement Proposals](https://github.com/jmespath-community/jmespath.spec):

- **JEP-014** (String Functions): `lower`, `upper`, `trim`, `trim_left`, `trim_right`, `pad_left`, `pad_right`, `replace`, `split`, `find_first`, `find_last`
- **JEP-013** (Object Functions): `items`, `from_items`, `zip`

Functions that align with JEPs have `jep: Some("JEP-XXX")` in their `FunctionInfo` metadata, accessible via the registry.

Additional functions extend well beyond these proposals. Some JEPs (like arithmetic operators) require parser changes and cannot be implemented as extension functions.

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
