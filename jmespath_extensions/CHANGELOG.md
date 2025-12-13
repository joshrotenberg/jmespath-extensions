# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.6.1...v0.6.2) - 2025-12-13

### Fixed

- add missing chrono and base64 deps to validation feature ([#157](https://github.com/joshrotenberg/jmespath-extensions/pull/157))

## [0.6.1](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.6.0...v0.6.1) - 2025-12-13

### Added

- add ngrams, bigrams, trigrams text functions ([#154](https://github.com/joshrotenberg/jmespath-extensions/pull/154))
- auto-generate docs and registry from functions.toml ([#153](https://github.com/joshrotenberg/jmespath-extensions/pull/153))
- add multi-match extension functions ([#152](https://github.com/joshrotenberg/jmespath-extensions/pull/152))
- add statistics functions for time-series and analytics ([#151](https://github.com/joshrotenberg/jmespath-extensions/pull/151))
- add validation functions for credit cards, phone, JWT, dates, JSON, and encoding ([#150](https://github.com/joshrotenberg/jmespath-extensions/pull/150))
- add JSON/Object path operations and schema discovery functions ([#149](https://github.com/joshrotenberg/jmespath-extensions/pull/149))
- add epoch conversion, period boundaries, and comparison datetime functions ([#148](https://github.com/joshrotenberg/jmespath-extensions/pull/148))
- add array functional programming functions ([#147](https://github.com/joshrotenberg/jmespath-extensions/pull/147))
- add string utility functions (mask, redact, normalize_whitespace, is_blank, abbreviate, center, reverse_string) ([#146](https://github.com/joshrotenberg/jmespath-extensions/pull/146))
- add crypto/security functions (HMAC, SHA-512, JWT) and flatten ([#144](https://github.com/joshrotenberg/jmespath-extensions/pull/144))

### Other

- add jsonpatch and multi-match to CI and benchmarks ([#155](https://github.com/joshrotenberg/jmespath-extensions/pull/155))

## [0.6.0](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.5.0...v0.6.0) - 2025-12-12

### Other

- add comprehensive test coverage for array and expression functions ([#135](https://github.com/joshrotenberg/jmespath-extensions/pull/135))
- add reduce_expr, scan_expr, order_by to registry metadata ([#134](https://github.com/joshrotenberg/jmespath-extensions/pull/134))
- add missing jsonpatch and multi-match to documentation ([#132](https://github.com/joshrotenberg/jmespath-extensions/pull/132))

## [0.5.0](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.4.1...v0.5.0) - 2025-12-10

### Added

- add aho-corasick multi-pattern matching functions ([#129](https://github.com/joshrotenberg/jmespath-extensions/pull/129))

## [0.4.1](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.4.0...v0.4.1) - 2025-12-10

### Added

- *(datetime)* add is_after, is_before, is_between, time_ago functions ([#126](https://github.com/joshrotenberg/jmespath-extensions/pull/126))

## [0.4.0](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.6...v0.4.0) - 2025-12-09

### Added

- [**breaking**] align functions with JEP-013 and JEP-014 specs ([#125](https://github.com/joshrotenberg/jmespath-extensions/pull/125))
- add partial application functions (partial, apply) ([#124](https://github.com/joshrotenberg/jmespath-extensions/pull/124))

## [0.3.6](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.5...v0.3.6) - 2025-12-09

### Added

- add lodash-inspired object path functions ([#115](https://github.com/joshrotenberg/jmespath-extensions/pull/115))
- add count_by expression function ([#114](https://github.com/joshrotenberg/jmespath-extensions/pull/114))
- add date/time utility functions ([#111](https://github.com/joshrotenberg/jmespath-extensions/pull/111))

### Other

- add attribution to jmespath crate ([#112](https://github.com/joshrotenberg/jmespath-extensions/pull/112))

## [0.3.5](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.4...v0.3.5) - 2025-12-09

### Added

- add JSON Patch and Merge Patch functions (Issue #60) ([#108](https://github.com/joshrotenberg/jmespath-extensions/pull/108))
- add array utility functions (Issue #62) ([#107](https://github.com/joshrotenberg/jmespath-extensions/pull/107))
- add statistical functions (histogram, normalize, z_score, correlation) ([#104](https://github.com/joshrotenberg/jmespath-extensions/pull/104))

## [0.3.4](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.3...v0.3.4) - 2025-12-09

### Added

- add new array and string functions ([#101](https://github.com/joshrotenberg/jmespath-extensions/pull/101))

## [0.3.3](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.2...v0.3.3) - 2025-12-09

### Other

- move library to jmespath_extensions/ subdirectory ([#98](https://github.com/joshrotenberg/jmespath-extensions/pull/98))

## [0.3.2](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.1...v0.3.2) - 2025-12-08

### Fixed

- pass clean version tag to homebrew action ([#93](https://github.com/joshrotenberg/jmespath-extensions/pull/93))
- correct function names in registry metadata ([#92](https://github.com/joshrotenberg/jmespath-extensions/pull/92))

## [0.3.1](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.3.0...v0.3.1) - 2025-12-08

### Fixed

- enable GitHub release creation for jmespath_extensions library ([#91](https://github.com/joshrotenberg/jmespath-extensions/pull/91))
- only trigger binary release workflow for jpx tags ([#90](https://github.com/joshrotenberg/jmespath-extensions/pull/90))
- correct homebrew asset URL to match cargo-dist naming ([#88](https://github.com/joshrotenberg/jmespath-extensions/pull/88))

### Other

- add crates.io badges to jpx README ([#89](https://github.com/joshrotenberg/jmespath-extensions/pull/89))

## [0.3.0](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.2.3...v0.3.0) - 2025-12-08

### Added

- add cargo-dist for binary releases ([#83](https://github.com/joshrotenberg/jmespath-extensions/pull/83))
- promote jpx to workspace member for crates.io publishing ([#80](https://github.com/joshrotenberg/jmespath-extensions/pull/80))
- *(jpx)* add environment variable configuration support ([#79](https://github.com/joshrotenberg/jmespath-extensions/pull/79))
- *(jpx)* add --strict mode for spec-only JMESPath ([#78](https://github.com/joshrotenberg/jmespath-extensions/pull/78))
- add alias and feature metadata to function registry ([#77](https://github.com/joshrotenberg/jmespath-extensions/pull/77))
- add lodash-style FP functions ([#73](https://github.com/joshrotenberg/jmespath-extensions/pull/73))
- add deep_equals and deep_diff functions ([#72](https://github.com/joshrotenberg/jmespath-extensions/pull/72))
- *(jpx)* add verbose mode, query chaining, and shell completions ([#70](https://github.com/joshrotenberg/jmespath-extensions/pull/70))
- *(jpx)* add -o/--output and -q/--quiet flags ([#53](https://github.com/joshrotenberg/jmespath-extensions/pull/53))
- add to_fixed and format_number functions ([#52](https://github.com/joshrotenberg/jmespath-extensions/pull/52))
- *(jpx)* add --query-file / -Q flag ([#51](https://github.com/joshrotenberg/jmespath-extensions/pull/51))
- add new math and string functions ([#50](https://github.com/joshrotenberg/jmespath-extensions/pull/50))
- *(jpx)* customize color scheme for better visibility ([#42](https://github.com/joshrotenberg/jmespath-extensions/pull/42))
- *(jpx)* add colored JSON output ([#41](https://github.com/joshrotenberg/jmespath-extensions/pull/41))
- add jpx CLI enhancements and json_pointer function ([#40](https://github.com/joshrotenberg/jmespath-extensions/pull/40))

### Fixed

- use correct [[package]] array syntax in release-plz.toml ([#86](https://github.com/joshrotenberg/jmespath-extensions/pull/86))
- remove invalid allow_dirty option from release-plz.toml ([#85](https://github.com/joshrotenberg/jmespath-extensions/pull/85))
- correct rust-toolchain action name in release-plz.yml ([#84](https://github.com/joshrotenberg/jmespath-extensions/pull/84))
- specify benchmark name to avoid --noplot being passed to lib tests ([#26](https://github.com/joshrotenberg/jmespath-extensions/pull/26))

### Other

- enhance expression function descriptions in registry ([#82](https://github.com/joshrotenberg/jmespath-extensions/pull/82))
- comprehensive documentation audit fixes ([#81](https://github.com/joshrotenberg/jmespath-extensions/pull/81))

## [0.2.3](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.2.2...v0.2.3) - 2025-12-07

### Added

- add JEP reference field and improve documentation ([#24](https://github.com/joshrotenberg/jmespath-extensions/pull/24))
- add FunctionRegistry for runtime control and introspection ([#22](https://github.com/joshrotenberg/jmespath-extensions/pull/22))

## [0.2.2](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.2.1...v0.2.2) - 2025-12-06

### Other

- add crates.io metadata ([#20](https://github.com/joshrotenberg/jmespath-extensions/pull/20))

## [0.2.1](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.2.0...v0.2.1) - 2025-12-06

### Added

- add duration, color, and computing modules ([#19](https://github.com/joshrotenberg/jmespath-extensions/pull/19))
- add benchmarks, improve CI, slim down README ([#18](https://github.com/joshrotenberg/jmespath-extensions/pull/18))
- add geo, semver, network, ids, and text modules ([#17](https://github.com/joshrotenberg/jmespath-extensions/pull/17))
- add phonetic encoding functions ([#16](https://github.com/joshrotenberg/jmespath-extensions/pull/16))
- add expression-based higher-order functions ([#15](https://github.com/joshrotenberg/jmespath-extensions/pull/15))
- add fuzzy string matching functions ([#13](https://github.com/joshrotenberg/jmespath-extensions/pull/13))
- add datetime functions ([#12](https://github.com/joshrotenberg/jmespath-extensions/pull/12))

## [0.2.0](https://github.com/joshrotenberg/jmespath-extensions/compare/v0.1.0...v0.2.0) - 2025-12-06

### Fixed

- json_decode returns null for invalid JSON instead of error ([#11](https://github.com/joshrotenberg/jmespath-extensions/pull/11))
- address remaining RedisJSON compatibility issues ([#10](https://github.com/joshrotenberg/jmespath-extensions/pull/10))
- improve RedisJSON compatibility ([#8](https://github.com/joshrotenberg/jmespath-extensions/pull/8))
- relax dependency versions for downstream compatibility ([#6](https://github.com/joshrotenberg/jmespath-extensions/pull/6))

### Other

- add error helper functions and improve error messages ([#9](https://github.com/joshrotenberg/jmespath-extensions/pull/9))
- add standard badges to README ([#7](https://github.com/joshrotenberg/jmespath-extensions/pull/7))
- [**breaking**] align function names with JMESPath JEPs ([#5](https://github.com/joshrotenberg/jmespath-extensions/pull/5))
- add JMESPath JEP alignment section to README ([#4](https://github.com/joshrotenberg/jmespath-extensions/pull/4))
