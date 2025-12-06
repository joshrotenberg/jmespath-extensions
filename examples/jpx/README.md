# jpx - JMESPath CLI with Extended Functions

A command-line tool for querying JSON data using JMESPath expressions with 189 additional functions beyond the standard JMESPath specification. Also includes the 26 standard JMESPath built-in functions.

## Installation

```bash
# From the jmespath_extensions repository
cd examples/jpx
cargo install --path .

# Or run directly
cargo run -- '<expression>'
```

## Usage

```bash
jpx [OPTIONS] [EXPRESSION]

Arguments:
  [EXPRESSION]  JMESPath expression to evaluate

Options:
  -f, --file <FILE>           Input file (reads from stdin if not provided)
  -r, --raw                   Output raw strings without quotes
  -c, --compact               Compact output (no pretty printing)
      --list-functions        List all available extension functions
      --list-category <NAME>  List functions in a specific category
      --describe <FUNCTION>   Show detailed info for a specific function
  -h, --help                  Print help
  -V, --version               Print version
```

## Function Discovery

```bash
# List all available functions grouped by category
# Shows 26 standard JMESPath functions and 189 extension functions
jpx --list-functions

# List functions in a specific category
jpx --list-category string
jpx --list-category math
jpx --list-category geo
jpx --list-category standard  # List all 26 standard JMESPath functions

# Get detailed info about a specific function
# Shows type (standard JMESPath or extension), category, signature, and example
jpx --describe upper
jpx --describe haversine_km
jpx --describe abs  # Standard JMESPath function
```

## Examples

### Basic Queries

```bash
# Simple field access
echo '{"name": "Alice", "age": 30}' | jpx 'name'
# "Alice"

# Array operations
echo '[1, 2, 3, 4, 5]' | jpx 'sum(@)'
# 15.0

# Nested access
echo '{"user": {"profile": {"email": "alice@example.com"}}}' | jpx 'user.profile.email'
# "alice@example.com"
```

### String Functions

```bash
# Case conversion
echo '{"name": "hello world"}' | jpx 'upper(name)'
# "HELLO WORLD"

echo '{"name": "Hello World"}' | jpx 'snake_case(name)'
# "hello_world"

# String manipulation
echo '{"text": "  trim me  "}' | jpx -r 'trim(text)'
# trim me

echo '{"words": ["hello", "world"]}' | jpx -r 'join(", ", words)'
# hello, world
```

### Array Functions

```bash
# Get unique values
echo '{"nums": [1, 2, 2, 3, 3, 3]}' | jpx 'unique(nums)'
# [1, 2, 3]

# Chunk arrays
echo '{"items": [1, 2, 3, 4, 5, 6]}' | jpx 'chunk(items, `2`)'
# [[1, 2], [3, 4], [5, 6]]

# Array statistics
echo '[10, 20, 30, 40, 50]' | jpx 'median(@)'
# 30.0

echo '[10, 20, 30, 40, 50]' | jpx 'stddev(@)'
# 14.142135623730951
```

### Date/Time Functions

```bash
# Current timestamp
echo '{}' | jpx 'now()'
# "2024-01-15T10:30:00Z"

# Parse and format dates
echo '{"date": "2024-01-15"}' | jpx 'format_date(parse_date(date, `%Y-%m-%d`), `%B %d, %Y`)'
# "January 15, 2024"

# Date arithmetic
echo '{"date": "2024-01-15T00:00:00Z"}' | jpx 'date_add(date, `7`, `days`)'
# "2024-01-22T00:00:00Z"
```

### Duration Functions

```bash
# Parse human-readable durations
echo '{"duration": "1h30m"}' | jpx 'parse_duration(duration)'
# 5400.0

echo '{"duration": "2d12h"}' | jpx 'duration_hours(parse_duration(duration))'
# 60.0

# Format seconds as duration
echo '{"seconds": 3725}' | jpx -r 'format_duration(seconds)'
# 1h2m5s
```

### Color Functions

```bash
# Convert colors
echo '{"color": "#ff5500"}' | jpx 'hex_to_rgb(color)'
# {"r": 255, "g": 85, "b": 0}

echo '{"r": 255, "g": 128, "b": 0}' | jpx -r 'rgb_to_hex(r, g, b)'
# #ff8000

# Color manipulation
echo '{"color": "#3366cc"}' | jpx -r 'lighten(color, `20`)'
# #5c85d6

echo '{"color": "#ff0000"}' | jpx -r 'color_complement(color)'
# #00ffff
```

### Computing Functions

```bash
# Parse byte sizes
echo '{"size": "1.5 GB"}' | jpx 'parse_bytes(size)'
# 1500000000.0

# Format bytes
echo '{"bytes": 1073741824}' | jpx -r 'format_bytes_binary(bytes)'
# 1.00 GiB

# Bitwise operations
echo '{"a": 12, "b": 10}' | jpx 'bit_and(a, b)'
# 8

echo '{"n": 1}' | jpx 'bit_shift_left(n, `4`)'
# 16
```

### Hash and Encoding Functions

```bash
# Hash functions
echo '{"text": "hello"}' | jpx -r 'md5(text)'
# 5d41402abc4b2a76b9719d911017c592

echo '{"text": "hello"}' | jpx -r 'sha256(text)'
# 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824

# Base64 encoding
echo '{"text": "hello world"}' | jpx -r 'base64_encode(text)'
# aGVsbG8gd29ybGQ=

# URL encoding
echo '{"query": "hello world & more"}' | jpx -r 'url_encode(query)'
# hello%20world%20%26%20more
```

### Geo Functions

```bash
# Calculate distance between coordinates
echo '{"lat1": 40.7128, "lon1": -74.0060, "lat2": 34.0522, "lon2": -118.2437}' | jpx 'haversine_km(lat1, lon1, lat2, lon2)'
# 3935.746...

# Calculate bearing
echo '{"lat1": 40.7128, "lon1": -74.0060, "lat2": 34.0522, "lon2": -118.2437}' | jpx 'bearing(lat1, lon1, lat2, lon2)'
# 273.429...
```

### Network Functions

```bash
# IP address operations
echo '{"ip": "192.168.1.1"}' | jpx 'ip_to_int(ip)'
# 3232235777

echo '{"cidr": "10.0.0.0/8", "ip": "10.1.2.3"}' | jpx 'cidr_contains(cidr, ip)'
# true

echo '{"ip": "192.168.1.1"}' | jpx 'is_private_ip(ip)'
# true
```

### Semver Functions

```bash
# Parse semantic versions
echo '{"version": "1.2.3-beta.1"}' | jpx 'semver_parse(version)'
# {"major": 1, "minor": 2, "patch": 3, "prerelease": "beta.1", "build": null}

# Compare versions
echo '{"v1": "1.2.3", "v2": "1.3.0"}' | jpx 'semver_compare(v1, v2)'
# -1

# Check version constraints
echo '{"version": "1.5.0", "constraint": "^1.0.0"}' | jpx 'semver_matches(version, constraint)'
# true
```

### Text Analysis Functions

```bash
# Word and character counts
echo '{"text": "Hello world, this is a test."}' | jpx 'word_count(text)'
# 6

echo '{"text": "Hello world!"}' | jpx 'char_count(text)'
# 12

# Reading time estimation
echo '{"article": "This is a longer article with many words..."}' | jpx 'reading_time(article)'
# "1 min read"

# Word frequencies
echo '{"text": "the quick brown fox jumps over the lazy dog the"}' | jpx 'word_frequencies(text)'
# {"the": 3, "quick": 1, "brown": 1, ...}
```

### Phonetic Functions

```bash
# Soundex encoding
echo '{"name": "Robert"}' | jpx -r 'soundex(name)'
# R163

# Check if names sound alike
echo '{"name1": "Robert", "name2": "Rupert"}' | jpx 'sounds_like(name1, name2)'
# true

# Double Metaphone
echo '{"name": "Smith"}' | jpx 'double_metaphone(name)'
# {"primary": "SM0", "secondary": "XMT"}
```

### Fuzzy Matching Functions

```bash
# Levenshtein distance
echo '{"s1": "kitten", "s2": "sitting"}' | jpx 'levenshtein(s1, s2)'
# 3

# Jaro-Winkler similarity
echo '{"s1": "MARTHA", "s2": "MARHTA"}' | jpx 'jaro_winkler(s1, s2)'
# 0.961...

# Sorensen-Dice coefficient
echo '{"s1": "night", "s2": "nacht"}' | jpx 'sorensen_dice(s1, s2)'
# 0.25
```

### ID Generation Functions

```bash
# Generate UUIDs
echo '{}' | jpx -r 'uuid()'
# 550e8400-e29b-41d4-a716-446655440000

# Generate nanoids
echo '{}' | jpx -r 'nanoid()'
# V1StGXR8_Z5jdHi6B-myT

# Generate ULIDs
echo '{}' | jpx -r 'ulid()'
# 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Validation Functions

```bash
# Email validation
echo '{"email": "user@example.com"}' | jpx 'is_email(email)'
# true

# URL validation
echo '{"url": "https://example.com/path"}' | jpx 'is_url(url)'
# true

# UUID validation
echo '{"id": "550e8400-e29b-41d4-a716-446655440000"}' | jpx 'is_uuid(id)'
# true

# IP address validation
echo '{"ip": "192.168.1.1"}' | jpx 'is_ipv4(ip)'
# true
```

### Expression Functions (Higher-Order)

```bash
# Filter with expression
echo '{"nums": [1, 2, 3, 4, 5]}' | jpx 'filter_expr(nums, &@ > `2`)'
# [3, 4, 5]

# Map with expression
echo '{"nums": [1, 2, 3]}' | jpx 'map_expr(nums, &@ * `2`)'
# [2, 4, 6]

# Group by expression
echo '{"items": [{"type": "a", "v": 1}, {"type": "b", "v": 2}, {"type": "a", "v": 3}]}' | jpx 'group_by_expr(items, &type)'
# {"a": [{"type": "a", "v": 1}, {"type": "a", "v": 3}], "b": [{"type": "b", "v": 2}]}
```

## Using Test Data Files

The `testdata/` directory contains sample JSON files for experimenting:

```bash
# Users data
jpx -f testdata/users.json '[].name'
jpx -f testdata/users.json 'filter_expr(@, &age > `25`) | [].name'

# Server logs
jpx -f testdata/servers.json 'filter_expr(@, &status == `active`) | length(@)'
jpx -f testdata/servers.json '[].{name: name, uptime: format_duration(uptime_seconds)}'

# E-commerce orders
jpx -f testdata/orders.json 'sum([].total)'
jpx -f testdata/orders.json 'group_by_expr(@, &status)'

# Geo locations
jpx -f testdata/locations.json 'haversine_km([0].lat, [0].lon, [1].lat, [1].lon)'

# Versions
jpx -f testdata/packages.json 'sort_by_expr(@, &semver_parse(version).major)'
```

## Tips

- Use `-r` (raw) when piping string output to other commands
- Use `-c` (compact) for single-line JSON output
- Use `--list-functions` to see all available functions
- Backticks create literal values: `` `5` `` is number 5, `` `"hello"` `` is string
- Use `&` prefix for expression references in higher-order functions

## License

MIT OR Apache-2.0
