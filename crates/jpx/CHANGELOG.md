# Changelog

All notable changes to jpx will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.18](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.17...jpx-v0.1.18) - 2026-01-17

### Added

- use default query for single-query .jpx files ([#362](https://github.com/joshrotenberg/jmespath-extensions/pull/362))
- enable MCP feature by default ([#361](https://github.com/joshrotenberg/jmespath-extensions/pull/361))
- *(mcp)* add runtime query storage tools ([#347](https://github.com/joshrotenberg/jmespath-extensions/pull/347))

### Other

- documentation audit and improvements ([#355](https://github.com/joshrotenberg/jmespath-extensions/pull/355))
- improve error message for non-JSON input ([#343](https://github.com/joshrotenberg/jmespath-extensions/pull/343))

## [0.1.17](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.16...jpx-v0.1.17) - 2026-01-16

### Added

- *(discovery)* add search preprocessing for better BM25 indexing ([#333](https://github.com/joshrotenberg/jmespath-extensions/pull/333))
- *(cli)* improve help discoverability and set default binary ([#315](https://github.com/joshrotenberg/jmespath-extensions/pull/315))
- *(cli)* add two-tier help system (-h short, --help long) ([#313](https://github.com/joshrotenberg/jmespath-extensions/pull/313))

## [0.1.16](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.15...jpx-v0.1.16) - 2026-01-14

### Added

- *(discovery)* BM25 search quality improvements ([#296](https://github.com/joshrotenberg/jmespath-extensions/pull/296))
- *(mcp)* add BM25 search indexing and discovery protocol ([#291](https://github.com/joshrotenberg/jmespath-extensions/pull/291))
- *(discovery)* add fuzzy_search, fuzzy_match, and fuzzy_score functions ([#288](https://github.com/joshrotenberg/jmespath-extensions/pull/288))
- *(mcp)* add discovery and analysis tools for AI agents ([#283](https://github.com/joshrotenberg/jmespath-extensions/pull/283))
- *(cli)* add --stream flag for line-by-line NDJSON processing ([#281](https://github.com/joshrotenberg/jmespath-extensions/pull/281))
- *(cli)* add config file support with TOML ([#278](https://github.com/joshrotenberg/jmespath-extensions/pull/278))
- *(cli)* add --debug flag for diagnostic information ([#277](https://github.com/joshrotenberg/jmespath-extensions/pull/277))
- *(cli)* add --similar flag to find related functions ([#272](https://github.com/joshrotenberg/jmespath-extensions/pull/272))
- *(cli)* add --bench flag for expression benchmarking ([#269](https://github.com/joshrotenberg/jmespath-extensions/pull/269))
- *(cli)* add --paths and --table flags ([#267](https://github.com/joshrotenberg/jmespath-extensions/pull/267))
- *(cli)* add output format options (--yaml, --toml, --csv, --tsv, --lines) ([#266](https://github.com/joshrotenberg/jmespath-extensions/pull/266))
- *(cli)* add --stats flag for quick data inspection ([#265](https://github.com/joshrotenberg/jmespath-extensions/pull/265))
- *(cli)* support multiple positional expressions as pipeline ([#262](https://github.com/joshrotenberg/jmespath-extensions/pull/262))
- *(cli)* add --diff, --patch, and --merge flags for JSON Patch operations ([#243](https://github.com/joshrotenberg/jmespath-extensions/pull/243))
- *(cli)* add colored output and --search flag for function discovery ([#234](https://github.com/joshrotenberg/jmespath-extensions/pull/234))

### Fixed

- *(mcp)* Parameters<()> schema bug + mock server + BM25 improvements issue ([#294](https://github.com/joshrotenberg/jmespath-extensions/pull/294))
- *(cli)* prefix all error messages with 'jpx:' ([#276](https://github.com/joshrotenberg/jmespath-extensions/pull/276))
- make Category match statements exhaustive for compile-time safety ([#213](https://github.com/joshrotenberg/jmespath-extensions/pull/213))
- add language category to MCP parse_category ([#212](https://github.com/joshrotenberg/jmespath-extensions/pull/212))

### Other

- acknowledge JMESPath project and add jq comparison ([#279](https://github.com/joshrotenberg/jmespath-extensions/pull/279))
- add comprehensive integration tests for jpx CLI and MCP server ([#224](https://github.com/joshrotenberg/jmespath-extensions/pull/224))

## [0.1.15](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.14...jpx-v0.1.15) - 2026-01-12

### Added

- *(mcp)* add --strict flag for standard-only JMESPath mode ([#189](https://github.com/joshrotenberg/jmespath-extensions/pull/189))
- *(mcp)* add evaluate_file tool for file-based queries ([#187](https://github.com/joshrotenberg/jmespath-extensions/pull/187))
- *(mcp)* add keys tool for extracting object keys ([#186](https://github.com/joshrotenberg/jmespath-extensions/pull/186))
- *(mcp)* add format, diff, patch, and merge tools ([#185](https://github.com/joshrotenberg/jmespath-extensions/pull/185))
- *(mcp)* add batch_evaluate tool for multiple expressions ([#183](https://github.com/joshrotenberg/jmespath-extensions/pull/183))
- *(jpx)* add MCP server support ([#177](https://github.com/joshrotenberg/jmespath-extensions/pull/177))

### Other

- *(jpx)* update MCP server documentation with all 12 tools ([#188](https://github.com/joshrotenberg/jmespath-extensions/pull/188))

## [0.1.14](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.13...jpx-v0.1.14) - 2025-12-18

### Added

- move REPL demos to demos.toml with build.rs generation ([#168](https://github.com/joshrotenberg/jmespath-extensions/pull/168))

## [0.1.13](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.12...jpx-v0.1.13) - 2025-12-14

### Added

- add .suggest command for smart query suggestions ([#161](https://github.com/joshrotenberg/jmespath-extensions/pull/161))
- add interactive REPL with syntax highlighting and demos ([#159](https://github.com/joshrotenberg/jmespath-extensions/pull/159))

## [0.1.12](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.11...jpx-v0.1.12) - 2025-12-13

### Other

- updated the following local packages: jmespath_extensions

## [0.1.11](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.10...jpx-v0.1.11) - 2025-12-13

### Added

- auto-generate docs and registry from functions.toml ([#153](https://github.com/joshrotenberg/jmespath-extensions/pull/153))

## [0.1.10](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.9...jpx-v0.1.10) - 2025-12-12

### Other

- updated the following local packages: jmespath_extensions

## [0.1.9](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.8...jpx-v0.1.9) - 2025-12-10

### Other

- updated the following local packages: jmespath_extensions

## [0.1.8](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.7...jpx-v0.1.8) - 2025-12-10

### Other

- updated the following local packages: jmespath_extensions

## [0.1.7](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.6...jpx-v0.1.7) - 2025-12-09

### Added

- *(jpx)* add --explain flag to show parsed AST ([#118](https://github.com/joshrotenberg/jmespath-extensions/pull/118))
- *(jpx)* add cargo-style colored help output ([#116](https://github.com/joshrotenberg/jmespath-extensions/pull/116))

## [0.1.6](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.5...jpx-v0.1.6) - 2025-12-09

### Other

- add attribution to jmespath crate ([#112](https://github.com/joshrotenberg/jmespath-extensions/pull/112))

## [0.1.5](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.4...jpx-v0.1.5) - 2025-12-09

### Other

- reduce unnecessary allocations in jpx CLI ([#109](https://github.com/joshrotenberg/jmespath-extensions/pull/109))

## [0.1.4](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.3...jpx-v0.1.4) - 2025-12-09

### Other

- updated the following local packages: jmespath_extensions

## [0.1.3](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.2...jpx-v0.1.3) - 2025-12-09

### Other

- move library to jmespath_extensions/ subdirectory ([#98](https://github.com/joshrotenberg/jmespath-extensions/pull/98))

## [0.1.2](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.1...jpx-v0.1.2) - 2025-12-08

### Fixed

- correct function names in registry metadata ([#92](https://github.com/joshrotenberg/jmespath-extensions/pull/92))

## [0.1.1](https://github.com/joshrotenberg/jmespath-extensions/compare/jpx-v0.1.0...jpx-v0.1.1) - 2025-12-08

### Other

- add crates.io badges to jpx README ([#89](https://github.com/joshrotenberg/jmespath-extensions/pull/89))

## [0.1.0] - Initial Release

### Added

- JMESPath CLI with 150+ extended functions from jmespath_extensions
- Multiple input modes: file, stdin, inline JSON (`-e`)
- Multiple output formats: JSON (default), pretty JSON, raw text, YAML-style
- Quiet mode (`-q`) for silent operation
- Verbose mode (`-v`) for debugging
- Strict/spec-only mode (`--strict`) to disable extensions
- Shell completions generation (`--completions`)
- Function discovery (`--list-functions`, `--describe-function`)
- Environment variable configuration:
  - `JPX_OUTPUT_FORMAT`: Set default output format
  - `JPX_STRICT`: Enable strict mode by default
  - `JPX_QUIET`: Enable quiet mode by default
  - `JPX_VERBOSE`: Enable verbose mode by default

### Function Categories

- **String**: 30+ functions (trim, split, join, case conversion, etc.)
- **Array**: 20+ functions (flatten, unique, sort, chunk, etc.)
- **Object**: 10+ functions (keys, values, merge, pick, omit, etc.)
- **Math**: 15+ functions (abs, ceil, floor, round, sum, avg, etc.)
- **DateTime**: Date parsing, formatting, arithmetic
- **Hash**: MD5, SHA1, SHA256, CRC32
- **Encoding**: Base64, hex, URL encoding
- **Regex**: Pattern matching and replacement
- **Expression**: Higher-order functions (map_expr, filter_expr, etc.)
- **And more**: UUID, geo, phonetic, semver, network, color functions
