# jmespath_extensions

[![Crates.io](https://img.shields.io/crates/v/jmespath_extensions.svg)](https://crates.io/crates/jmespath_extensions)
[![Documentation](https://docs.rs/jmespath_extensions/badge.svg)](https://docs.rs/jmespath_extensions)
[![License](https://img.shields.io/crates/l/jmespath_extensions.svg)](https://github.com/joshrotenberg/jmespath-extensions#license)
[![CI](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml)

Extended functions for JMESPath queries in Rust.

> **Warning: Non-Standard Extension**
>
> This crate provides **custom extension functions** that are **NOT part of the JMESPath specification**.
> Queries using these functions will **NOT work** in other JMESPath implementations (Python, JavaScript, Go, etc.).
>
> For portable queries, use only the [26 standard JMESPath built-in functions](https://jmespath.org/specification.html#built-in-functions).

## Overview

This crate provides 150+ additional functions beyond the standard JMESPath built-ins, organized into feature-gated categories. These extensions are useful when you need more powerful data transformation capabilities and portability across JMESPath implementations is not a concern.

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
| `geo` | `haversine`, `haversine_km`, `haversine_mi`, `bearing` | geoutils |
| `semver` | `semver_parse`, `semver_compare`, `semver_matches`, etc. | semver |
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
haversine_km(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`) → 5570.2
bearing(`40.7128`, `-74.0060`, `51.5074`, `-0.1278`)      → 51.2
```

### Network

```
cidr_contains('192.168.0.0/16', '192.168.1.1') → true
is_private_ip('10.0.0.1')                      → true
```

See the [API documentation](https://docs.rs/jmespath_extensions) for complete function reference with examples.

## JMESPath Community JEP Alignment

This crate aligns with several [JMESPath Enhancement Proposals](https://github.com/jmespath-community/jmespath.spec):

- **JEP-014** (String Functions): `lower`, `upper`, `trim`, `pad_left`, `pad_right`, `replace`, `split`, `find_first`, `find_last`
- **JEP-013** (Object Functions): `items`, `from_items`, `zip`

Additional functions extend well beyond these proposals. Some JEPs (like arithmetic operators) require parser changes and cannot be implemented as extension functions.

## Portability Warning

These extension functions are designed for use cases where:
- You control both the query author and the runtime environment
- You need functionality beyond standard JMESPath
- Cross-implementation compatibility is not required

For portable queries, use only the standard JMESPath built-in functions.

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
