# jpx-lsp

Language Server Protocol (LSP) implementation for JMESPath with extensions.

## Features

- **Autocomplete**: Function name completion with signatures and descriptions
- **Hover**: Documentation on hover for function names
- **Diagnostics**: Real-time syntax error detection

## Installation

```bash
cargo install --path jpx-lsp
```

## Usage

The LSP server communicates via stdin/stdout using the standard LSP protocol.

### VS Code

Add to your `settings.json`:

```json
{
  "jmespath.server.path": "/path/to/jpx-lsp"
}
```

Or create a VS Code extension that uses this LSP server.

### Neovim (with nvim-lspconfig)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.jpx_lsp then
  configs.jpx_lsp = {
    default_config = {
      cmd = { 'jpx-lsp' },
      filetypes = { 'jmespath' },
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname) or vim.fn.getcwd()
      end,
    },
  }
end

lspconfig.jpx_lsp.setup{}
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "jmespath"
scope = "source.jmespath"
file-types = ["jmespath"]
language-servers = ["jpx-lsp"]

[language-server.jpx-lsp]
command = "jpx-lsp"
```

## Supported Functions

The LSP provides completions for all 100+ JMESPath extension functions including:

- String functions (`upper`, `lower`, `trim`, `split`, etc.)
- Array functions (`zip`, `chunk`, `take`, `drop`, etc.)
- Math functions (`add`, `multiply`, `round`, `sqrt`, etc.)
- Date/time functions (`now`, `parse_date`, `format_date`, etc.)
- And many more...

See the [jmespath_extensions documentation](../jmespath_extensions/README.md) for the full function list.
