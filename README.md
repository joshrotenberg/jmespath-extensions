# jmespath_extensions

Extended functions for JMESPath queries in Rust.

> **Warning: Non-Standard Extension**
>
> This crate provides **custom extension functions** that are **NOT part of the JMESPath specification**.
> Queries using these functions will **NOT work** in other JMESPath implementations (Python, JavaScript, Go, etc.).
>
> For portable queries, use only the [26 standard JMESPath built-in functions](https://jmespath.org/specification.html#built-in-functions).

## Overview

This crate provides 100+ additional functions beyond the standard JMESPath built-ins, organized into feature-gated categories. These extensions are useful when you need more powerful data transformation capabilities and portability across JMESPath implementations is not a concern.

## Quick Start

```rust
use jmespath::Runtime;
use jmespath_extensions::register_all;

let mut runtime = Runtime::new();
runtime.register_builtin_functions();
register_all(&mut runtime);

// Now you can use extended functions in queries
let expr = runtime.compile("items[*].name | lower(@)").unwrap();
```

## Features

| Feature | Description | External Dependencies |
|---------|-------------|----------------------|
| `full` (default) | All functions | All deps below |
| `core` | No external deps | None |
| `string` | String manipulation | None |
| `array` | Array operations | None |
| `object` | Object operations | None |
| `math` | Math/statistics | None |
| `type` | Type checking/conversion | None |
| `utility` | Utility functions | None |
| `validation` | Email, URL, IP validation | regex |
| `path` | File path operations | None |
| `hash` | MD5, SHA1, SHA256, CRC32 | md-5, sha1, sha2, crc32fast |
| `encoding` | Base64, hex encoding | base64, hex |
| `regex` | Regular expressions | regex |
| `url` | URL encoding/parsing | url, urlencoding |
| `uuid` | UUID generation | uuid |
| `rand` | Random number generation | rand |

### Minimal Dependencies

```toml
[dependencies]
jmespath_extensions = { version = "0.1", default-features = false, features = ["core"] }
```

## Function Reference

### String Functions

| Function | Description | Example |
|----------|-------------|---------|
| `lower(string)` | Convert to lowercase | `lower('HELLO')` → `"hello"` |
| `upper(string)` | Convert to uppercase | `upper('hello')` → `"HELLO"` |
| `trim(string)` | Remove leading/trailing whitespace | `trim('  hi  ')` → `"hi"` |
| `trim_start(string)` | Remove leading whitespace | `trim_start('  hi')` → `"hi"` |
| `trim_end(string)` | Remove trailing whitespace | `trim_end('hi  ')` → `"hi"` |
| `capitalize(string)` | Capitalize first letter | `capitalize('hello')` → `"Hello"` |
| `title(string)` | Capitalize each word | `title('hello world')` → `"Hello World"` |
| `split(string, delim)` | Split string into array | `split('a,b,c', ',')` → `["a","b","c"]` |
| `replace(string, old, new)` | Replace all occurrences | `replace('foo', 'o', 'a')` → `"faa"` |
| `repeat(string, count)` | Repeat string n times | `repeat('ab', 3)` → `"ababab"` |
| `pad_left(string, width, char)` | Left-pad string | `pad_left('5', 3, '0')` → `"005"` |
| `pad_right(string, width, char)` | Right-pad string | `pad_right('5', 3, '0')` → `"500"` |
| `substr(string, start, len?)` | Extract substring | `substr('hello', 1, 3)` → `"ell"` |
| `slice(string, start, end?)` | Extract by indices | `slice('hello', 1, 4)` → `"ell"` |
| `index_of(string, search)` | Find first occurrence | `index_of('hello', 'l')` → `2` |
| `last_index_of(string, search)` | Find last occurrence | `last_index_of('hello', 'l')` → `3` |
| `concat(array, sep?)` | Join strings | `concat(['a','b'], '-')` → `"a-b"` |
| `camel_case(string)` | Convert to camelCase | `camel_case('hello_world')` → `"helloWorld"` |
| `snake_case(string)` | Convert to snake_case | `snake_case('helloWorld')` → `"hello_world"` |
| `kebab_case(string)` | Convert to kebab-case | `kebab_case('helloWorld')` → `"hello-world"` |

### Array Functions

| Function | Description | Example |
|----------|-------------|---------|
| `first(array)` | First element or null | `first([1,2,3])` → `1` |
| `last(array)` | Last element or null | `last([1,2,3])` → `3` |
| `unique(array)` | Remove duplicates | `unique([1,2,1])` → `[1,2]` |
| `take(array, n)` | First n elements | `take([1,2,3], 2)` → `[1,2]` |
| `drop(array, n)` | Skip n elements | `drop([1,2,3], 1)` → `[2,3]` |
| `chunk(array, size)` | Split into chunks | `chunk([1,2,3,4], 2)` → `[[1,2],[3,4]]` |
| `zip(arr1, arr2)` | Pair elements | `zip([1,2], ['a','b'])` → `[[1,"a"],[2,"b"]]` |
| `flatten_deep(array)` | Recursive flatten | `flatten_deep([[1,[2]],3])` → `[1,2,3]` |
| `compact(array)` | Remove null/false | `compact([1,null,2])` → `[1,2]` |
| `range(start, end, step?)` | Generate sequence | `range(0, 5, 2)` → `[0,2,4]` |
| `index_at(array, idx)` | Get by index (neg ok) | `index_at([1,2,3], -1)` → `3` |
| `includes(array, val)` | Check membership | `includes([1,2], 2)` → `true` |
| `find_index(array, val)` | Find element index | `find_index([1,2,3], 2)` → `1` |
| `difference(arr1, arr2)` | Set difference | `difference([1,2,3], [2])` → `[1,3]` |
| `intersection(arr1, arr2)` | Set intersection | `intersection([1,2], [2,3])` → `[2]` |
| `union(arr1, arr2)` | Set union | `union([1,2], [2,3])` → `[1,2,3]` |
| `group_by(array, field)` | Group by field value | `group_by(items, 'category')` |
| `frequencies(array)` | Count occurrences | `frequencies(['a','b','a'])` → `{"a":2,"b":1}` |

### Object Functions

| Function | Description | Example |
|----------|-------------|---------|
| `entries(object)` | Convert to [{key, value}] | `entries({a:1})` → `[{"key":"a","value":1}]` |
| `from_entries(array)` | Convert [{key, value}] to object | `from_entries([{"key":"a","value":1}])` → `{"a":1}` |
| `pick(object, keys)` | Select specific keys | `pick({a:1,b:2}, ['a'])` → `{"a":1}` |
| `omit(object, keys)` | Exclude specific keys | `omit({a:1,b:2}, ['a'])` → `{"b":2}` |
| `deep_merge(obj1, obj2)` | Recursively merge objects | `deep_merge({a:{b:1}}, {a:{c:2}})` |
| `invert(object)` | Swap keys and values | `invert({a:'x'})` → `{"x":"a"}` |
| `rename_keys(object, mapping)` | Rename keys | `rename_keys({a:1}, {a:'b'})` → `{"b":1}` |
| `flatten_keys(object, sep?)` | Flatten nested object | `flatten_keys({a:{b:1}})` → `{"a.b":1}` |

### Math/Statistics Functions

| Function | Description | Example |
|----------|-------------|---------|
| `round(n, precision?)` | Round to decimals | `round(3.14159, 2)` → `3.14` |
| `floor_fn(n)` | Round down | `floor_fn(3.7)` → `3` |
| `ceil_fn(n)` | Round up | `ceil_fn(3.2)` → `4` |
| `abs_fn(n)` | Absolute value | `abs_fn(-5)` → `5` |
| `mod_fn(n, divisor)` | Modulo | `mod_fn(7, 3)` → `1` |
| `pow(base, exp)` | Exponentiation | `pow(2, 3)` → `8` |
| `sqrt(n)` | Square root | `sqrt(16)` → `4` |
| `log(n, base?)` | Logarithm | `log(100, 10)` → `2` |
| `clamp(n, min, max)` | Constrain to range | `clamp(5, 0, 3)` → `3` |
| `median(array)` | Median value | `median([1,2,3,4,5])` → `3` |
| `percentile(array, p)` | Nth percentile (0-100) | `percentile([1,2,3,4,5], 50)` → `3` |
| `variance(array)` | Population variance | `variance([1,2,3])` → `0.666...` |
| `stddev(array)` | Standard deviation | `stddev([1,2,3])` → `0.816...` |
| `sin(n)`, `cos(n)`, `tan(n)` | Trigonometric functions | `sin(0)` → `0` |

### Type Functions

| Function | Description | Example |
|----------|-------------|---------|
| `to_string(any)` | Convert to string | `to_string(123)` → `"123"` |
| `to_number(any)` | Convert to number | `to_number('42')` → `42` |
| `to_boolean(any)` | Convert to boolean | `to_boolean('')` → `false` |
| `type_of(any)` | Get type name | `type_of([])` → `"array"` |
| `is_string(any)` | Check if string | `is_string('hi')` → `true` |
| `is_number(any)` | Check if number | `is_number(42)` → `true` |
| `is_boolean(any)` | Check if boolean | `is_boolean(true)` → `true` |
| `is_array(any)` | Check if array | `is_array([])` → `true` |
| `is_object(any)` | Check if object | `is_object({})` → `true` |
| `is_null(any)` | Check if null | `is_null(null)` → `true` |

### Utility Functions

| Function | Description | Example |
|----------|-------------|---------|
| `now()` | Unix timestamp (seconds) | `now()` → `1699900000` |
| `now_ms()` | Unix timestamp (milliseconds) | `now_ms()` → `1699900000000` |
| `default(value, fallback)` | Return fallback if null | `default(null, 'N/A')` → `"N/A"` |
| `if(cond, then, else)` | Ternary conditional | `if(true, 'yes', 'no')` → `"yes"` |
| `coalesce(...)` | First non-null value | `coalesce(null, null, 'a')` → `"a"` |

### Hash Functions (feature: `hash`)

| Function | Description |
|----------|-------------|
| `md5(string)` | MD5 hash (hex) |
| `sha1(string)` | SHA-1 hash (hex) |
| `sha256(string)` | SHA-256 hash (hex) |
| `crc32(string)` | CRC32 checksum (number) |

### Encoding Functions (feature: `encoding`)

| Function | Description |
|----------|-------------|
| `base64_encode(string)` | Base64 encode |
| `base64_decode(string)` | Base64 decode |
| `hex_encode(string)` | Hex encode |
| `hex_decode(string)` | Hex decode |

### Validation Functions (feature: `validation` or `regex`)

| Function | Description |
|----------|-------------|
| `is_email(string)` | Check if valid email |
| `is_url(string)` | Check if valid URL |
| `is_uuid(string)` | Check if valid UUID |
| `is_ipv4(string)` | Check if valid IPv4 |
| `is_ipv6(string)` | Check if valid IPv6 |

### Path Functions (feature: `path`)

| Function | Description | Example |
|----------|-------------|---------|
| `path_basename(string)` | Get filename from path | `path_basename('/a/b.txt')` → `"b.txt"` |
| `path_dirname(string)` | Get directory from path | `path_dirname('/a/b.txt')` → `"/a"` |
| `path_ext(string)` | Get file extension | `path_ext('/a/b.txt')` → `".txt"` |

### URL Functions (feature: `url`)

| Function | Description |
|----------|-------------|
| `url_encode(string)` | URL encode |
| `url_decode(string)` | URL decode |
| `url_parse(string)` | Parse URL into components |

### Regex Functions (feature: `regex`)

| Function | Description |
|----------|-------------|
| `regex_match(string, pattern)` | Check if pattern matches |
| `regex_extract(string, pattern)` | Extract first match |
| `regex_replace(string, pattern, replacement)` | Replace matches |

### Random Functions (feature: `rand`)

| Function | Description |
|----------|-------------|
| `random()` | Random float 0.0-1.0 |
| `uuid()` | Generate UUID v4 |
| `shuffle(array)` | Randomly shuffle array |
| `sample(array, n)` | Random sample of n elements |

## JMESPath Community JEP Alignment

This crate aligns with several [JMESPath Enhancement Proposals (JEPs)](https://github.com/jmespath-community/jmespath.spec) from the JMESPath community, while also providing additional functionality.

### JEP-014: String Functions

| JEP-014 Function | jmespath_extensions | Notes |
|------------------|---------------------|-------|
| `lower` | `lower` | |
| `upper` | `upper` | |
| `trim` | `trim` | |
| `trim_left` | `trim_start` | Different name |
| `trim_right` | `trim_end` | Different name |
| `pad_left` | `pad_left` | |
| `pad_right` | `pad_right` | |
| `replace` | `replace` | |
| `split` | `split` | |
| `find_first` | `index_of` | Different name |
| `find_last` | `last_index_of` | Different name |

**Additional string functions**: `capitalize`, `title`, `title_case`, `camel_case`, `snake_case`, `kebab_case`, `substr`, `slice`, `concat`, `repeat`, `truncate`

### JEP-013: Object Functions

| JEP-013 Function | jmespath_extensions | Notes |
|------------------|---------------------|-------|
| `items` | `entries` | Different name (JS-style) |
| `from_items` | `from_entries` | Different name (JS-style) |
| `zip` | `zip` | In array module |

**Additional object functions**: `pick`, `omit`, `deep_merge`, `rename_keys`, `invert`, `flatten_keys`, `unflatten_keys`

### Language-Level JEPs (Not Applicable)

Some JEPs propose grammar-level changes that cannot be implemented via extension functions:

| JEP | Description | Status |
|-----|-------------|--------|
| JEP-016 | Arithmetic operators (`+`, `-`, `*`, `/`, `%`) | Requires parser changes |
| JEP-011a | Lexical scoping (`let x = expr in expr`) | Requires parser changes |
| JEP-017 | Root reference (`$`) | Requires parser changes |

While these can't be implemented as functions, this crate provides related math functions: `abs_fn`, `ceil_fn`, `floor_fn`, `mod_fn`, `pow`, `sqrt`, `log`, `round`, `clamp`, `sign`, plus trigonometric functions.

### Beyond JEPs

This crate provides extensive functionality not yet addressed by the JEP process:

| Category | Functions |
|----------|-----------|
| **Hash** | `md5`, `sha1`, `sha256`, `crc32` |
| **Encoding** | `base64_encode`, `base64_decode`, `hex_encode`, `hex_decode` |
| **URL** | `url_encode`, `url_decode`, `url_parse` |
| **Regex** | `regex_match`, `regex_replace`, `regex_extract` |
| **UUID** | `uuid`, `is_uuid` |
| **Random** | `random`, `sample`, `shuffle` |
| **Validation** | `is_email`, `is_url`, `is_ipv4`, `is_ipv6`, `is_blank` |
| **Path** | `path_basename`, `path_dirname`, `path_ext`, `path_join` |
| **Time** | `now`, `now_ms` |
| **Statistics** | `median`, `percentile`, `variance`, `stddev` |

## Portability Warning

These extension functions are designed for use cases where:
- You control both the query author and the runtime environment
- You need functionality beyond standard JMESPath
- Cross-implementation compatibility is not required

If you need portable JMESPath queries, stick to the standard built-in functions:
`abs`, `avg`, `ceil`, `contains`, `ends_with`, `floor`, `join`, `keys`, `length`, `map`, `max`, `max_by`, `merge`, `min`, `min_by`, `not_null`, `reverse`, `sort`, `sort_by`, `starts_with`, `sum`, `to_array`, `to_number`, `to_string`, `type`, `values`

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
