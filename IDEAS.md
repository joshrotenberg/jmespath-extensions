# JMESPath Extensions - Ideas & Future Directions

## CLI Tool (jpx) Enhancements

### High Value / Low Effort
- [ ] **Query chaining** - Pipe output of one query as input to next: `jpx -e 'foo' | jpx -e 'bar'` (already works) but also `jpx -e 'foo' -e 'bar'` in single invocation
- [ ] **Multiple query files** - Run several query files in sequence: `jpx -Q q1.jmespath -Q q2.jmespath`
- [ ] **Output to file** - `-o/--output <file>` instead of stdout redirect
- [ ] **Quiet mode** - `-q/--quiet` suppress non-result output
- [ ] **Verbose/debug mode** - `-v/--verbose` show parsed expression, timing, etc.
- [ ] **Version info in --help** - Show jmespath_extensions version

### Medium Effort / High Value
- [ ] **Interactive REPL mode** - `jpx --repl` or `jpx -i` for exploratory querying
  - Tab completion for function names
  - Expression history (readline/rustyline)
  - `.load <file>` to load JSON, `.query <expr>` to run queries
  - Show intermediate results
- [ ] **Named queries in file** - YAML/TOML format with multiple named queries:
  ```yaml
  queries:
    users_by_country: "group_by(@, &country)"
    active_users: "[?status == 'active']"
  ```
  Run with: `jpx -Q queries.yaml --run users_by_country`
- [ ] **Watch mode** - `jpx --watch -f data.json -e 'expr'` - re-run on file change
- [ ] **Diff mode** - Compare results of two expressions or two files
- [ ] **Schema inference** - `jpx --schema -f data.json` - infer JSON schema from data

### Higher Effort / Strategic
- [ ] **Shebang support** - Make query files executable:
  ```
  #!/usr/bin/env jpx --query-file
  [?age > `18`] | sort_by(@, &name)
  ```
- [ ] **Variables/parameters** - `jpx -e '$threshold' --set threshold=100 -f data.json`
- [ ] **Import/include** - `@import "common-queries.jmespath"` in query files
- [ ] **Output formats** - CSV, TSV, YAML, table (ASCII), NDJSON
- [ ] **Input formats** - YAML, CSV, NDJSON (not just JSON)
- [ ] **Streaming/large file support** - Process NDJSON line-by-line without loading all into memory

## New Functions to Consider

### Data Transformation
- [ ] `json_patch(obj, patch)` - RFC 6902 JSON Patch
- [ ] `json_merge_patch(obj, patch)` - RFC 7386 JSON Merge Patch
- [ ] `json_pointer(obj, pointer)` - RFC 6901 JSON Pointer (may exist?)
- [ ] `deep_diff(a, b)` - Return structural diff between two values
- [ ] `deep_equals(a, b)` - Deep equality check
- [ ] `schema_validate(obj, schema)` - JSON Schema validation

### String Functions
- [ ] `format(template, ...args)` - String interpolation: `format("Hello {}", name)`
- [ ] `wrap(str, width)` - Word wrap text
- [ ] `truncate(str, len, suffix)` - Truncate with "..."
- [ ] `pluralize(word, count)` - Simple pluralization
- [ ] `humanize(str)` - "user_name" -> "User Name"
- [ ] `deburr(str)` - Remove diacritics: "café" -> "cafe"

### Array Functions
- [ ] `partition(arr, expr)` - Split into [matches, non-matches]
- [ ] `window(arr, size, step)` - Sliding window
- [ ] `combinations(arr, k)` - k-combinations
- [ ] `permutations(arr)` - All permutations
- [ ] `interleave(arr1, arr2)` - Interleave two arrays
- [ ] `scan(arr, expr, init)` - Like reduce but returns intermediate values

### Date/Time Functions
- [ ] `timezone_convert(ts, from_tz, to_tz)` - Timezone conversion
- [ ] `is_weekend(date)`, `is_weekday(date)`
- [ ] `business_days_between(d1, d2)` - Count business days
- [ ] `relative_time(ts)` - "2 hours ago", "in 3 days"
- [ ] `quarter(date)` - Get quarter (1-4)

### Network/Security
- [ ] `jwt_decode(token)` - Decode JWT payload (no verification)
- [ ] `parse_user_agent(ua)` - Parse browser user agent string
- [ ] `parse_cookie(str)` - Parse cookie header
- [ ] `mask(str, char, keep_start, keep_end)` - "1234****7890"

### Statistics
- [ ] `percentile(arr, p)` - Calculate percentile
- [ ] `histogram(arr, bins)` - Bucket values
- [ ] `normalize(arr)` - Normalize to 0-1 range
- [ ] `z_score(arr)` - Calculate z-scores
- [ ] `correlation(arr1, arr2)` - Pearson correlation

## Library Enhancements

### Performance
- [ ] **Expression caching** - LRU cache for compiled expressions
- [ ] **Lazy evaluation** - Don't evaluate branches not needed
- [ ] **SIMD for array ops** - Use SIMD for numeric array operations
- [ ] **Benchmark suite** - Comprehensive benchmarks with criterion

### Developer Experience
- [ ] **Better error messages** - Show position in expression, suggest fixes
- [ ] **Expression linting** - Warn about potentially slow patterns
- [ ] **Type inference** - Infer expected input/output types from expression
- [ ] **Playground** - WASM-based web playground for trying expressions

### Integration
- [ ] **Python bindings** - PyO3 bindings for Python users
- [ ] **Node.js bindings** - NAPI-RS bindings
- [ ] **WASM target** - Compile to WASM for browser use

## Distribution / Ecosystem

### Package Distribution
- [ ] **Homebrew formula** - `brew install jpx`
- [ ] **cargo-binstall support** - Pre-built binaries
- [ ] **Docker image** - `docker run jpx ...`
- [ ] **GitHub releases** - Automated binary releases for platforms
- [ ] **AUR package** - Arch Linux
- [ ] **Debian/RPM packages** - Linux distros

### Documentation
- [ ] **Cookbook** - Real-world examples by use case
- [ ] **Function reference site** - Searchable docs like lodash.com
- [ ] **Video tutorials** - YouTube walkthroughs
- [ ] **Comparison guide** - jpx vs jq vs gron vs fx

### Community
- [ ] **VS Code extension** - Syntax highlighting for .jmespath files
- [ ] **JetBrains plugin** - Same for IntelliJ family
- [ ] **GitHub Action** - `uses: jmespath/jpx-action@v1`
- [ ] **Pre-commit hook** - Validate JSON with JMESPath expressions

## Crazy Ideas (Why Not?)

- [ ] **jpx server** - HTTP API for running JMESPath queries (like jq-web)
- [ ] **jpx as SQL** - `jpx --sql "SELECT name, age FROM @.users WHERE age > 18"`
- [ ] **Visual query builder** - TUI or web UI for building expressions visually
- [ ] **jpx daemon** - Long-running process with Unix socket for fast queries
- [ ] **Expression optimizer** - Rewrite expressions for better performance
- [ ] **JMESPath to Rust codegen** - Compile expression to native Rust code
- [ ] **AI-assisted queries** - "find users over 18 sorted by name" -> JMESPath expression
