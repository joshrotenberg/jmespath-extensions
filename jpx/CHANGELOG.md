# Changelog

All notable changes to jpx will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
