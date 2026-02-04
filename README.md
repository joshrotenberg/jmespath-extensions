# jmespath-extensions

[![Crates.io](https://img.shields.io/crates/v/jmespath_extensions.svg)](https://crates.io/crates/jmespath_extensions)
[![Documentation](https://docs.rs/jmespath_extensions/badge.svg)](https://docs.rs/jmespath_extensions)
[![CI](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml)

Extended JMESPath with 400+ functions. Rust library and Python bindings.

**[Documentation](https://joshrotenberg.github.io/jpx/)** | **[Function Reference](https://joshrotenberg.github.io/jpx/functions/overview.html)**

## Quick Start

```bash
# Install the CLI
brew install joshrotenberg/brew/jpx
# or: cargo install jpx

# Use it
echo '{"name": "world"}' | jpx 'upper(name)'
# "WORLD"

curl -s https://api.github.com/users/octocat | jpx '{
  login: login,
  created: format_date(parse_date(created_at), `%B %Y`)
}'
# {"login": "octocat", "created": "January 2011"}
```

## Packages

| Package | Description | Install |
|---------|-------------|---------|
| **[jmespath_extensions](https://crates.io/crates/jmespath_extensions)** | Rust library | `cargo add jmespath_extensions` |
| **[jmespath-extensions](https://pypi.org/project/jmespath-extensions/)** | Python bindings | `pip install jmespath-extensions` |

For CLI tools and MCP server, see the **[jpx repository](https://github.com/joshrotenberg/jpx)**:
- **jpx** - CLI with REPL, multiple output formats
- **jpx-mcp** - MCP server for AI assistants
- **jpx-engine** - Query engine with discovery features

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

[Full function reference](https://joshrotenberg.github.io/jpx/functions/overview.html)

## Examples

```bash
# Filter and transform
echo '[{"name":"alice","age":30},{"name":"bob","age":25}]' \
  | jpx '[?age > `26`].{name: upper(name), birth_year: `2024` - age}'
# [{"name": "ALICE", "birth_year": 1994}]

# Fuzzy matching
jpx 'levenshtein(`kitten`, `sitting`)'
# 3

# Date arithmetic
jpx 'format_date(date_add(now(), `7`, `days`), `%Y-%m-%d`)'
# "2024-01-24"

# Network validation
echo '["10.0.0.1", "8.8.8.8", "192.168.1.1"]' \
  | jpx '[?is_private_ip(@)]'
# ["10.0.0.1", "192.168.1.1"]
```

## Library Usage

### Rust

```rust
use jmespath_extensions::search;
use serde_json::json;

let data = json!({"items": [1, 2, 3, 4, 5]});
let result = search("sum(items)", &data)?;
assert_eq!(result, json!(15));
```

### Python

```python
import jmespath_extensions as jpx

data = {"items": [1, 2, 3, 4, 5]}
result = jpx.search("sum(items)", data)
assert result == 15
```

## Related Projects

- **[jpx](https://github.com/joshrotenberg/jpx)** - CLI, MCP server, and query engine
- **[JMESPath](https://jmespath.org/)** - The query language specification
- **[jmespath.rs](https://crates.io/crates/jmespath)** - Rust implementation

## License

MIT or Apache-2.0
