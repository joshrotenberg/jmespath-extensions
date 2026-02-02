# jmespath-extensions

Extended JMESPath implementation with 400+ functions for JSON processing.

## Project Structure

```
jmespath-extensions/
├── src/                    # Rust library source
├── python/                 # Python bindings (separate crate, PyPI release)
├── benches/                # Benchmarks
├── tests/                  # Integration tests
├── functions.toml          # Function registry (single source of truth)
├── build.rs                # Generates registry.rs from functions.toml
└── Cargo.toml              # Crate configuration
```

## Related Repositories

The CLI and server tools are in separate repositories:
- **[jpx](https://github.com/joshrotenberg/jpx)** - CLI tool, MCP server, query engine

## Function Development Guidelines

When adding or updating functions:

1. **Implementation** in the appropriate module (e.g., `src/array.rs`)
   - Register function in both `register()` and `register_filtered()` functions
   - Use regular comments (`//`) not doc comments (`///`) above `define_function!` macros

2. **functions.toml update** - Add entry with name, category, description, signature, examples

3. **Tests** - Unit tests in the same module, cover edge cases

4. **CI Feature Matrix** - New features must be added to `.github/workflows/ci.yml`

## Testing Commands

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
