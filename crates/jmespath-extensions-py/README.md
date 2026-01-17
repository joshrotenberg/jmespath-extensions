# jmespath-extensions

[![PyPI](https://img.shields.io/pypi/v/jmespath-extensions.svg)](https://pypi.org/project/jmespath-extensions/)
[![Python](https://img.shields.io/pypi/pyversions/jmespath-extensions.svg)](https://pypi.org/project/jmespath-extensions/)
[![License](https://img.shields.io/pypi/l/jmespath-extensions.svg)](https://github.com/joshrotenberg/jmespath-extensions#license)

JMESPath with **400+ extension functions** for strings, arrays, dates, hashing, encoding, and more.

This package provides Python bindings for [jmespath-extensions](https://github.com/joshrotenberg/jmespath-extensions), a Rust library that extends JMESPath with hundreds of additional functions.

## Installation

```bash
pip install jmespath-extensions
```

## Quick Start

```python
import jmespath_extensions as jpx

# String functions
jpx.search("upper(name)", {"name": "alice"})
# 'ALICE'

# Math functions
jpx.search("sum(values)", {"values": [1, 2, 3, 4, 5]})
# 15

jpx.search("median(scores)", {"scores": [10, 20, 30, 40, 50]})
# 30

# Date/time functions
jpx.search("format_date(now(), '%Y-%m-%d')", {})
# '2024-01-15'

# Array functions
jpx.search("unique(items)", {"items": [1, 2, 2, 3, 3, 3]})
# [1, 2, 3]

jpx.search("chunk(items, `2`)", {"items": [1, 2, 3, 4, 5, 6]})
# [[1, 2], [3, 4], [5, 6]]

# Object functions
jpx.search("pick(user, ['name', 'email'])", {"user": {"name": "alice", "email": "a@b.com", "age": 30}})
# {'name': 'alice', 'email': 'a@b.com'}

# Hash functions
jpx.search("md5(password)", {"password": "secret"})
# '5ebe2294ecd0e0f08eab7690d2a6ee69'

# Fuzzy matching
jpx.search("levenshtein(a, b)", {"a": "kitten", "b": "sitting"})
# 3
```

## Compiled Expressions

For repeated searches, compile the expression once:

```python
import jmespath_extensions as jpx

# Compile once
expr = jpx.compile("users[?age > `18`].name | sort(@)")

# Use many times
expr.search({"users": [{"name": "alice", "age": 30}, {"name": "bob", "age": 16}]})
# ['alice']

expr.search({"users": [{"name": "charlie", "age": 25}, {"name": "dave", "age": 19}]})
# ['charlie', 'dave']
```

## Function Discovery

```python
import jmespath_extensions as jpx

# List all categories
jpx.list_categories()
# ['standard', 'string', 'array', 'object', 'math', 'type', 'utility', ...]

# List functions in a category
jpx.list_functions("string")
# ['upper', 'lower', 'trim', 'split', 'replace', 'camel_case', ...]

# Get function details
jpx.describe("upper")
# {
#     'name': 'upper',
#     'category': 'string',
#     'description': 'Convert string to uppercase',
#     'signature': 'string -> string',
#     'example': "upper('hello') -> \"HELLO\"",
#     'is_standard': False
# }
```

## Function Categories

| Category | Examples |
|----------|----------|
| **string** | `upper`, `lower`, `trim`, `split`, `replace`, `camel_case`, `snake_case` |
| **array** | `first`, `last`, `unique`, `chunk`, `zip`, `flatten`, `group_by` |
| **object** | `pick`, `omit`, `deep_merge`, `flatten_keys`, `items`, `from_items` |
| **math** | `round`, `sqrt`, `median`, `stddev`, `sum`, `avg`, `min`, `max` |
| **datetime** | `now`, `format_date`, `parse_date`, `date_add`, `date_diff` |
| **hash** | `md5`, `sha1`, `sha256`, `crc32` |
| **encoding** | `base64_encode`, `base64_decode`, `hex_encode`, `url_encode` |
| **regex** | `regex_match`, `regex_extract`, `regex_replace` |
| **fuzzy** | `levenshtein`, `jaro_winkler`, `soundex`, `metaphone` |
| **geo** | `geo_distance_km`, `geo_bearing` |
| **validation** | `is_email`, `is_url`, `is_uuid`, `is_ipv4` |
| **uuid** | `uuid` (v4 generation) |
| **ids** | `nanoid`, `ulid` |

See the [full documentation](https://github.com/joshrotenberg/jmespath-extensions) for all 400+ functions.

## Compatibility with jmespath-py

This package is designed to be a drop-in enhancement for [jmespath](https://pypi.org/project/jmespath/). All standard JMESPath functions work exactly as expected:

```python
import jmespath_extensions as jpx

# Standard JMESPath still works
jpx.search("length(items)", {"items": [1, 2, 3]})
# 3

jpx.search("sort(items)", {"items": [3, 1, 2]})
# [1, 2, 3]

jpx.search("[?age > `18`]", [{"age": 20}, {"age": 15}])
# [{'age': 20}]
```

## Performance

This package uses Rust under the hood via [PyO3](https://pyo3.rs/), providing near-native performance for JSON processing.

## License

MIT OR Apache-2.0
