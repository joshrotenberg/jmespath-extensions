# Changelog

All notable changes to jpx will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
