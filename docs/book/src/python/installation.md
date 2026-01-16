# Installation

## From PyPI

```bash
pip install jmespath-extensions
```

## From Source

```bash
# Clone the repository
git clone https://github.com/joshrotenberg/jmespath-extensions
cd jmespath-extensions/jmespath-extensions-py

# Install maturin if needed
pip install maturin

# Build and install
maturin develop --release
```

## Verify Installation

```python
import jmespath_extensions

# Check it works
result = jmespath_extensions.search("length(@)", [1, 2, 3])
print(result)  # 3

# List available functions
functions = jmespath_extensions.list_functions()
print(f"Available functions: {len(functions)}")
```

## Requirements

- Python 3.8+
- No runtime dependencies (Rust extensions are statically linked)
