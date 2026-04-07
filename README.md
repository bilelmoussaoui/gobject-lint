# gobject-lint

A tree-sitter-based linter for GObject/C applications.

## Usage

```bash
# Lint current directory with default config
gobject-lint

# Lint specific directory
gobject-lint /path/to/project

# Use custom config file
gobject-lint --config my-lint.toml /path/to/project

# Verbose output
gobject-lint -v
```

## Configuration

Create a `gobject-lint.toml` file in your project root:

```toml
[rules]
# Ensure g_param_spec_* functions have NULL for nick and blurb parameters
g_param_spec_null_nick_blurb = true
```

See gobject-lint.toml for all the supported rules/configurations.

## LSP Server

For real-time linting in your editor:

```bash
cargo build --release --bin gobject-lsp
```

**Neovim** (nvim-lspconfig):
```lua
require('lspconfig.configs').gobject_lsp = {
  default_config = {
    cmd = {'gobject-lsp'},
    filetypes = {'c', 'h'},
    root_dir = require('lspconfig.util').root_pattern('gobject-lint.toml', '.git'),
  },
}
require('lspconfig').gobject_lsp.setup{}
```

**VS Code**: Use a generic LSP client extension pointing to `gobject-lsp`

**Helix** (`~/.config/helix/languages.toml`):
```toml
[[language]]
name = "c"
language-servers = ["clangd", "gobject-lsp"]

[language-server.gobject-lsp]
command = "gobject-lsp"
```

Co-Authored by Claude Code.
