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

This crate provides 130+ additional functions beyond the standard JMESPath built-ins, organized into feature-gated categories. These extensions are useful when you need more powerful data transformation capabilities and portability across JMESPath implementations is not a concern.

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
| `datetime` | Date/time functions | chrono |
| `fuzzy` | Fuzzy string matching | strsim |
| `expression` | Expression-based higher-order functions | None |
| `phonetic` | Phonetic encoding algorithms | rphonetic |
| `geo` | Geospatial distance and bearing | geoutils |
| `semver` | Semantic versioning operations | semver |
| `network` | IP address and CIDR operations | ipnetwork |
| `ids` | ID generation (NanoID, ULID) | nanoid, ulid |
| `text` | Text analysis (word count, reading time) | None |

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
| `trim_left(string)` | Remove leading whitespace | `trim_left('  hi')` → `"hi"` |
| `trim_right(string)` | Remove trailing whitespace | `trim_right('hi  ')` → `"hi"` |
| `capitalize(string)` | Capitalize first letter | `capitalize('hello')` → `"Hello"` |
| `title(string)` | Capitalize each word | `title('hello world')` → `"Hello World"` |
| `split(string, delim)` | Split string into array | `split('a,b,c', ',')` → `["a","b","c"]` |
| `replace(string, old, new)` | Replace all occurrences | `replace('foo', 'o', 'a')` → `"faa"` |
| `repeat(string, count)` | Repeat string n times | `repeat('ab', 3)` → `"ababab"` |
| `pad_left(string, width, char)` | Left-pad string | `pad_left('5', 3, '0')` → `"005"` |
| `pad_right(string, width, char)` | Right-pad string | `pad_right('5', 3, '0')` → `"500"` |
| `substr(string, start, len?)` | Extract substring | `substr('hello', 1, 3)` → `"ell"` |
| `slice(string, start, end?)` | Extract by indices | `slice('hello', 1, 4)` → `"ell"` |
| `find_first(string, search)` | Find first occurrence | `find_first('hello', 'l')` → `2` |
| `find_last(string, search)` | Find last occurrence | `find_last('hello', 'l')` → `3` |
| `concat(array, sep?)` | Join strings | `concat(['a','b'], '-')` → `"a-b"` |
| `camel_case(string)` | Convert to camelCase | `camel_case('hello_world')` → `"helloWorld"` |
| `snake_case(string)` | Convert to snake_case | `snake_case('helloWorld')` → `"hello_world"` |
| `kebab_case(string)` | Convert to kebab-case | `kebab_case('helloWorld')` → `"hello-world"` |
| `wrap(string, width)` | Word-wrap to width | `wrap('hello world', 5)` → `"hello\nworld"` |

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
| `items(object)` | Convert to [{key, value}] | `items({a:1})` → `[{"key":"a","value":1}]` |
| `from_items(array)` | Convert [{key, value}] to object | `from_items([{"key":"a","value":1}])` → `{"a":1}` |
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
| `random(min, max)` | Random float in range [min, max) |
| `uuid()` | Generate UUID v4 |
| `shuffle(array)` | Randomly shuffle array |
| `shuffle(array, seed)` | Deterministic shuffle with seed |
| `sample(array, n)` | Random sample of n elements |
| `sample(array, n, seed)` | Deterministic sample with seed |

### Date/Time Functions (feature: `datetime`)

| Function | Description | Example |
|----------|-------------|---------|
| `now()` | Current Unix timestamp (seconds) | `now()` → `1699900000` |
| `now_millis()` | Current Unix timestamp (milliseconds) | `now_millis()` → `1699900000000` |
| `parse_date(string, format?)` | Parse date string to timestamp | `parse_date('2024-07-03')` → `1719964800` |
| `format_date(timestamp, format)` | Format timestamp to string | `format_date(0, '%Y-%m-%d')` → `"1970-01-01"` |
| `date_add(timestamp, amount, unit)` | Add time to timestamp | `date_add(0, 1, 'days')` → `86400` |
| `date_diff(ts1, ts2, unit)` | Difference between timestamps | `date_diff(86400, 0, 'days')` → `1` |

**Time units**: `seconds`/`second`/`s`, `minutes`/`minute`/`m`, `hours`/`hour`/`h`, `days`/`day`/`d`, `weeks`/`week`/`w`

**Format specifiers** (chrono strftime): `%Y` (year), `%m` (month), `%d` (day), `%H` (hour), `%M` (minute), `%S` (second)

### Fuzzy String Matching (feature: `fuzzy`)

| Function | Description | Returns |
|----------|-------------|---------|
| `levenshtein(s1, s2)` | Edit distance (insertions, deletions, substitutions) | number |
| `normalized_levenshtein(s1, s2)` | Normalized edit distance | 0.0-1.0 |
| `damerau_levenshtein(s1, s2)` | Edit distance with transpositions | number |
| `jaro(s1, s2)` | Jaro similarity | 0.0-1.0 |
| `jaro_winkler(s1, s2)` | Jaro-Winkler similarity (boosts common prefixes) | 0.0-1.0 |
| `sorensen_dice(s1, s2)` | Sørensen-Dice coefficient (bigram-based) | 0.0-1.0 |

**Examples:**
- `levenshtein('kitten', 'sitting')` → `3`
- `jaro_winkler('hello', 'hallo')` → `0.88` (high similarity)
- `sorensen_dice('night', 'nacht')` → `0.25`

### Expression Functions (feature: `expression`)

Higher-order functions that accept JMESPath expressions as arguments for powerful data transformations.

| Function | Description | Example |
|----------|-------------|---------|
| `map_expr(expr, array)` | Apply expression to each element | `map_expr('name', users)` → `["Alice", "Bob"]` |
| `filter_expr(expr, array)` | Keep elements where expression is truthy | `filter_expr('age >= \`18\`', users)` |
| `any_expr(expr, array)` | True if any element matches | `any_expr('active', users)` → `true` |
| `all_expr(expr, array)` | True if all elements match | `all_expr('verified', users)` → `false` |
| `find_expr(expr, array)` | First element matching expression | `find_expr('id == \`2\`', users)` |
| `find_index_expr(expr, array)` | Index of first match or -1 | `find_index_expr('id == \`2\`', users)` → `1` |
| `count_expr(expr, array)` | Count elements where expression is truthy | `count_expr('active', users)` → `3` |
| `sort_by_expr(expr, array)` | Sort array by expression result | `sort_by_expr('age', users)` |
| `group_by_expr(expr, array)` | Group elements by expression result | `group_by_expr('type', items)` → `{"a": [...], "b": [...]}` |
| `partition_expr(expr, array)` | Split into [matches, non_matches] | `partition_expr('active', users)` → `[[...], [...]]` |
| `min_by_expr(expr, array)` | Element with minimum expression value | `min_by_expr('age', users)` |
| `max_by_expr(expr, array)` | Element with maximum expression value | `max_by_expr('age', users)` |
| `unique_by_expr(expr, array)` | Dedupe by expression result | `unique_by_expr('type', items)` |
| `flat_map_expr(expr, array)` | Map and flatten results | `flat_map_expr('tags', posts)` |

**Examples:**
```
// Extract names from objects
map_expr('name', `[{"name": "Alice"}, {"name": "Bob"}]`)
// Result: ["Alice", "Bob"]

// Filter adults
filter_expr('age >= `18`', `[{"age": 25}, {"age": 17}, {"age": 30}]`)
// Result: [{"age": 25}, {"age": 30}]

// Sort by field
sort_by_expr('score', `[{"score": 3}, {"score": 1}, {"score": 2}]`)
// Result: [{"score": 1}, {"score": 2}, {"score": 3}]

// Group by category
group_by_expr('type', `[{"type": "a", "val": 1}, {"type": "b", "val": 2}, {"type": "a", "val": 3}]`)
// Result: {"a": [{"type": "a", "val": 1}, {"type": "a", "val": 3}], "b": [{"type": "b", "val": 2}]}

// Partition into matches and non-matches
partition_expr('@ > `3`', `[1, 2, 3, 4, 5]`)
// Result: [[4, 5], [1, 2, 3]]

// Flatten all tags
flat_map_expr('tags', `[{"tags": ["a", "b"]}, {"tags": ["c"]}]`)
// Result: ["a", "b", "c"]
```

### Phonetic Functions (feature: `phonetic`)

Phonetic encoding algorithms for matching names and words by pronunciation.

| Function | Description | Example |
|----------|-------------|---------|
| `soundex(string)` | Classic 4-character Soundex code | `soundex('Robert')` → `"R163"` |
| `metaphone(string)` | Improved phonetic encoding | `metaphone('Smith')` → `"SM0"` |
| `double_metaphone(string)` | Returns [primary, alternate] | `double_metaphone('Schmidt')` → `["XMT", "SMT"]` |
| `nysiis(string)` | NY State Identification System | `nysiis('Johnson')` → `"JANSAN"` |
| `match_rating_codex(string)` | Western name matching code | `match_rating_codex('Smith')` → `"SMTH"` |
| `caverphone(string)` | Caverphone 1.0 (NZ optimized) | `caverphone('Thompson')` |
| `caverphone2(string)` | Caverphone 2.0 (improved) | `caverphone2('Thompson')` |
| `sounds_like(s1, s2)` | Check if strings sound alike (Soundex) | `sounds_like('Robert', 'Rupert')` → `true` |
| `phonetic_match(s1, s2, algorithm?)` | Compare using specified algorithm | `phonetic_match('Smith', 'Smyth', 'metaphone')` → `true` |

**Algorithm options for `phonetic_match`**: `soundex` (default), `metaphone`, `double_metaphone`, `nysiis`, `match_rating`/`mra`, `caverphone`/`caverphone1`, `caverphone2`

**Examples:**
```
// Check if names sound alike
sounds_like('Robert', 'Rupert')
// Result: true

// Compare using different algorithms
phonetic_match('Smith', 'Smyth', 'metaphone')
// Result: true

// Get double metaphone encodings
double_metaphone('Schmidt')
// Result: ["XMT", "SMT"]
```

### Geospatial Functions (feature: `geo`)

Calculate distances and bearings between geographic coordinates.

| Function | Description | Example |
|----------|-------------|---------|
| `haversine(lat1, lon1, lat2, lon2)` | Distance in meters between two points | `haversine(40.7128, -74.0060, 51.5074, -0.1278)` → `5570222.1` |
| `haversine_km(lat1, lon1, lat2, lon2)` | Distance in kilometers | `haversine_km(40.7128, -74.0060, 51.5074, -0.1278)` → `5570.2` |
| `haversine_mi(lat1, lon1, lat2, lon2)` | Distance in miles | `haversine_mi(40.7128, -74.0060, 51.5074, -0.1278)` → `3461.2` |
| `bearing(lat1, lon1, lat2, lon2)` | Compass bearing in degrees (0-360) | `bearing(40.7128, -74.0060, 51.5074, -0.1278)` → `51.2` |

**Examples:**
```
// Distance from NYC to London in km
haversine_km(40.7128, -74.0060, 51.5074, -0.1278)
// Result: ~5570.2

// Bearing from NYC to London
bearing(40.7128, -74.0060, 51.5074, -0.1278)
// Result: ~51.2 (northeast)
```

### Semantic Versioning Functions (feature: `semver`)

Parse and compare semantic versions (SemVer 2.0.0 compliant).

| Function | Description | Example |
|----------|-------------|---------|
| `semver_parse(string)` | Parse version into components | `semver_parse('1.2.3-beta+build')` → `{"major":1,"minor":2,"patch":3,"pre":"beta","build":"build"}` |
| `semver_major(string)` | Extract major version | `semver_major('1.2.3')` → `1` |
| `semver_minor(string)` | Extract minor version | `semver_minor('1.2.3')` → `2` |
| `semver_patch(string)` | Extract patch version | `semver_patch('1.2.3')` → `3` |
| `semver_compare(v1, v2)` | Compare versions (-1, 0, 1) | `semver_compare('1.0.0', '2.0.0')` → `-1` |
| `semver_matches(version, requirement)` | Check if version matches requirement | `semver_matches('1.2.3', '>=1.0.0')` → `true` |
| `is_semver(string)` | Check if valid semver | `is_semver('1.2.3')` → `true` |

**Examples:**
```
// Parse a version
semver_parse('2.1.0-alpha.1+build.123')
// Result: {"major": 2, "minor": 1, "patch": 0, "pre": "alpha.1", "build": "build.123"}

// Check version requirements
semver_matches('1.5.0', '>=1.0.0, <2.0.0')
// Result: true

// Compare versions
semver_compare('1.2.3', '1.2.4')
// Result: -1 (first is less than second)
```

### Network Functions (feature: `network`)

IP address and CIDR network operations.

| Function | Description | Example |
|----------|-------------|---------|
| `ip_to_int(string)` | Convert IPv4 to integer | `ip_to_int('192.168.1.1')` → `3232235777` |
| `int_to_ip(number)` | Convert integer to IPv4 | `int_to_ip(3232235777)` → `"192.168.1.1"` |
| `cidr_contains(cidr, ip)` | Check if IP is in CIDR range | `cidr_contains('192.168.0.0/16', '192.168.1.1')` → `true` |
| `cidr_network(cidr)` | Get network address | `cidr_network('192.168.1.100/24')` → `"192.168.1.0"` |
| `cidr_broadcast(cidr)` | Get broadcast address | `cidr_broadcast('192.168.1.0/24')` → `"192.168.1.255"` |
| `cidr_prefix(cidr)` | Get prefix length | `cidr_prefix('192.168.1.0/24')` → `24` |
| `is_private_ip(string)` | Check if IP is private (RFC 1918) | `is_private_ip('192.168.1.1')` → `true` |

**Examples:**
```
// Check if IP is in subnet
cidr_contains('10.0.0.0/8', '10.255.255.255')
// Result: true

// Get network boundaries
cidr_network('192.168.1.100/24')
// Result: "192.168.1.0"

cidr_broadcast('192.168.1.100/24')
// Result: "192.168.1.255"

// Check private IP ranges
is_private_ip('172.16.0.1')  // 172.16.0.0/12
// Result: true
```

### ID Generation Functions (feature: `ids`)

Generate unique identifiers.

| Function | Description | Example |
|----------|-------------|---------|
| `nanoid()` | Generate 21-char NanoID | `nanoid()` → `"V1StGXR8_Z5jdHi6B-myT"` |
| `nanoid(size)` | Generate NanoID with custom length | `nanoid(10)` → `"IRFa-VaY2b"` |
| `ulid()` | Generate ULID (sortable, timestamp-based) | `ulid()` → `"01ARZ3NDEKTSV4RRFFQ69G5FAV"` |
| `ulid_timestamp(ulid)` | Extract Unix timestamp from ULID (ms) | `ulid_timestamp('01ARZ3NDEKTSV4RRFFQ69G5FAV')` → `1469918176385` |

**Examples:**
```
// Generate compact URL-safe ID
nanoid()
// Result: "V1StGXR8_Z5jdHi6B-myT" (21 chars, URL-safe)

// Generate sortable ULID
ulid()
// Result: "01HQMYV7K5KCSKFJ8Y45VX3QJT" (26 chars, Crockford base32)

// Extract timestamp from ULID
ulid_timestamp('01HQMYV7K5KCSKFJ8Y45VX3QJT')
// Result: 1706000000000 (milliseconds since Unix epoch)
```

### Text Analysis Functions (feature: `text`)

Analyze text content for statistics and metrics.

| Function | Description | Example |
|----------|-------------|---------|
| `word_count(string)` | Count words | `word_count('Hello world!')` → `2` |
| `char_count(string)` | Count characters (excluding whitespace) | `char_count('Hello world!')` → `11` |
| `sentence_count(string)` | Count sentences | `sentence_count('Hello. World!')` → `2` |
| `paragraph_count(string)` | Count paragraphs | `paragraph_count('Para 1\n\nPara 2')` → `2` |
| `reading_time(string)` | Estimated reading time (minutes) | `reading_time(long_text)` → `5` |
| `reading_time_seconds(string)` | Estimated reading time (seconds) | `reading_time_seconds(text)` → `30` |
| `char_frequencies(string)` | Character frequency map | `char_frequencies('aab')` → `{"a":2,"b":1}` |
| `word_frequencies(string)` | Word frequency map | `word_frequencies('the cat the')` → `{"the":2,"cat":1}` |

**Examples:**
```
// Basic text stats
word_count('The quick brown fox jumps over the lazy dog')
// Result: 9

// Estimate reading time (assumes 200 wpm)
reading_time('... 1000 word article ...')
// Result: 5 (minutes)

// Get word frequencies
word_frequencies('to be or not to be')
// Result: {"to": 2, "be": 2, "or": 1, "not": 1}

// Character analysis
char_frequencies('hello')
// Result: {"h": 1, "e": 1, "l": 2, "o": 1}
```

## JMESPath Community JEP Alignment

This crate aligns with several [JMESPath Enhancement Proposals (JEPs)](https://github.com/jmespath-community/jmespath.spec) from the JMESPath community, while also providing additional functionality.

### JEP-014: String Functions

| JEP-014 Function | jmespath_extensions | Notes |
|------------------|---------------------|-------|
| `lower` | `lower` | |
| `upper` | `upper` | |
| `trim` | `trim` | |
| `trim_left` | `trim_left` | |
| `trim_right` | `trim_right` | |
| `pad_left` | `pad_left` | |
| `pad_right` | `pad_right` | |
| `replace` | `replace` | |
| `split` | `split` | |
| `find_first` | `find_first` | |
| `find_last` | `find_last` | |

**Additional string functions**: `capitalize`, `title`, `title_case`, `camel_case`, `snake_case`, `kebab_case`, `substr`, `slice`, `concat`, `repeat`, `truncate`, `wrap`

### JEP-013: Object Functions

| JEP-013 Function | jmespath_extensions | Notes |
|------------------|---------------------|-------|
| `items` | `items` | |
| `from_items` | `from_items` | |
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
| **Date/Time** | `now`, `now_millis`, `parse_date`, `format_date`, `date_add`, `date_diff` |
| **Statistics** | `median`, `percentile`, `variance`, `stddev` |
| **Fuzzy** | `levenshtein`, `jaro_winkler`, `sorensen_dice`, `damerau_levenshtein` |
| **Expression** | `map_expr`, `filter_expr`, `any_expr`, `all_expr`, `find_expr`, `find_index_expr`, `count_expr`, `sort_by_expr`, `group_by_expr`, `partition_expr`, `min_by_expr`, `max_by_expr`, `unique_by_expr`, `flat_map_expr` |
| **Phonetic** | `soundex`, `metaphone`, `double_metaphone`, `nysiis`, `match_rating_codex`, `caverphone`, `caverphone2`, `sounds_like`, `phonetic_match` |
| **Geo** | `haversine`, `haversine_km`, `haversine_mi`, `bearing` |
| **SemVer** | `semver_parse`, `semver_major`, `semver_minor`, `semver_patch`, `semver_compare`, `semver_matches`, `is_semver` |
| **Network** | `ip_to_int`, `int_to_ip`, `cidr_contains`, `cidr_network`, `cidr_broadcast`, `cidr_prefix`, `is_private_ip` |
| **IDs** | `nanoid`, `ulid`, `ulid_timestamp` |
| **Text** | `word_count`, `char_count`, `sentence_count`, `paragraph_count`, `reading_time`, `reading_time_seconds`, `char_frequencies`, `word_frequencies` |

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
