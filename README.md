# jmespath-extensions

## NOTE

This work has moved to https://github.com/joshrotenberg/jpx which includes a full [JMESPath Specification](https://jmespath.org/specification.html) 
implementation with some performance improvements over the official crate, the full extensions (plus more) implemented here, a higher level "engine" 
library that includes meta for both evaluation and function discovery, and the `jpx` CLI tool and the `jpx-mcp` MCP server.

No further development will be done on this repository.

[![Crates.io](https://img.shields.io/crates/v/jmespath_extensions.svg)](https://crates.io/crates/jmespath_extensions)
[![Documentation](https://docs.rs/jmespath_extensions/badge.svg)](https://docs.rs/jmespath_extensions)
[![CI](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml)

Extended JMESPath with 400+ functions. Rust library and Python bindings.

## Installation

### Rust

```toml
[dependencies]
jmespath_extensions = "0.9"
```

### Python

```bash
pip install jmespath-extensions
```

## Usage

### Rust

```rust
use jmespath_extensions::search;
use serde_json::json;

let data = json!({"items": [1, 2, 3, 4, 5]});
let result = search("sum(items)", &data)?;
assert_eq!(result, json!(15));

// String functions
let data = json!({"name": "alice"});
let result = search("upper(name)", &data)?;
assert_eq!(result, json!("ALICE"));

// Date functions
let result = search("format_date(now(), '%Y-%m-%d')", &json!({}))?;

// Array functions
let data = json!({"values": [1, 2, 2, 3, 3, 3]});
let result = search("unique(values)", &data)?;
assert_eq!(result, json!([1, 2, 3]));
```

### Python

```python
import jmespath_extensions as jmx

# Basic usage
data = {"items": [1, 2, 3, 4, 5]}
result = jmx.search("sum(items)", data)
assert result == 15

# String functions
result = jmx.search("upper(name)", {"name": "alice"})
assert result == "ALICE"

# Array functions
result = jmx.search("unique(values)", {"values": [1, 2, 2, 3]})
assert result == [1, 2, 3]
```

## Function Categories

| Category | Examples |
|----------|----------|
| **String** | `upper`, `lower`, `split`, `replace`, `camel_case`, `pad_left` |
| **Array** | `first`, `last`, `unique`, `chunk`, `zip`, `flatten`, `group_by` |
| **Math** | `round`, `sqrt`, `median`, `stddev`, `percentile` |
| **Date/Time** | `now`, `parse_date`, `format_date`, `date_add`, `date_diff` |
| **Hash** | `md5`, `sha256`, `hmac_sha256`, `crc32` |
| **Encoding** | `base64_encode`, `base64_decode`, `hex_encode`, `url_encode` |
| **Regex** | `regex_match`, `regex_extract`, `regex_replace` |
| **Geo** | `haversine`, `geo_distance_km`, `geo_bearing` |
| **Network** | `cidr_contains`, `is_private_ip`, `ip_to_int` |
| **JSON Patch** | `json_patch`, `json_merge_patch`, `json_diff` |
| **Fuzzy** | `levenshtein`, `jaro_winkler`, `soundex`, `metaphone` |
| **Expression** | `map_expr`, `filter_expr`, `sort_by_expr`, `group_by_expr` |

See [docs.rs](https://docs.rs/jmespath_extensions) for the full function reference.

## Related Projects

- **[jpx](https://github.com/joshrotenberg/jpx)** - CLI, MCP server, and query engine built on this library
- **[JMESPath](https://jmespath.org/)** - The query language specification
- **[jmespath.rs](https://crates.io/crates/jmespath)** - Rust JMESPath implementation

## License

MIT or Apache-2.0
