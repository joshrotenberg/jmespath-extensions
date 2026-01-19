# jmespath-extensions

[![Crates.io](https://img.shields.io/crates/v/jmespath_extensions.svg)](https://crates.io/crates/jmespath_extensions)
[![Documentation](https://docs.rs/jmespath_extensions/badge.svg)](https://docs.rs/jmespath_extensions)
[![CI](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/joshrotenberg/jmespath-extensions/actions/workflows/ci.yml)

Extended JMESPath with 400+ functions. Available as a CLI, MCP server, Rust library, and Python bindings.

**[Documentation](https://joshrotenberg.github.io/jmespath-extensions/)** | **[Function Reference](https://joshrotenberg.github.io/jmespath-extensions/functions/overview.html)**

## Quick Start

```bash
# Install
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

## What's Included

| Package | Description |
|---------|-------------|
| **[jpx](crates/jpx/)** | CLI tool with REPL, multiple output formats |
| **[jpx-server](crates/jpx-server/)** | MCP server for AI assistants |
| **[jmespath-extensions](crates/jmespath-extensions/)** | Rust library |
| **[jmespath-extensions-py](https://pypi.org/project/jmespath-extensions/)** | Python bindings |

## MCP Server

Give Claude (or any MCP client) the ability to query and transform JSON:

```json
{
  "mcpServers": {
    "jpx": {
      "command": "jpx-server"
    }
  }
}
```

**Tools:** `evaluate`, `batch_evaluate`, `validate`, `functions`, `describe`, `search`, `similar`, `format`, `diff`, `patch`, `merge`, `stats`, `paths`, `keys`

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

[Full function reference](https://joshrotenberg.github.io/jmespath-extensions/functions/overview.html)

## A Taste

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

## Acknowledgments

- **[JMESPath](https://jmespath.org/)** - The query language specification
- **[jmespath.rs](https://crates.io/crates/jmespath)** - Rust implementation by [@mtdowling](https://github.com/mtdowling)
- **[jp](https://github.com/jmespath/jp)** - The official JMESPath CLI

## License

MIT or Apache-2.0
