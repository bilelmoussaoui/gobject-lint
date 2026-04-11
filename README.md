# goblin

A tree-sitter-based linter for GObject/C applications.

**goblin** = **G**Object **L**inter

## Usage

```bash
# Lint current directory with default config
goblin

# Lint specific directory
goblin /path/to/project

# Use custom config file
goblin --config my-lint.toml /path/to/project

# Verbose output
goblin -v

# List all available rules with their enabled/disabled status
goblin --list-rules

# Run only specific rules (overrides config)
goblin --only use_g_strcmp0 --only use_clear_functions

# Add custom ignore patterns
goblin --ignore "build/**" --ignore "tests/**"
```

## Available Rules

See [RULES.md](RULES.md) for a complete list of all available rules organized by category.

Run `goblin --list-rules` to see the current status of all rules.

## Configuration

Create a `goblin.toml` file in your project root:

```toml
# Minimum supported GLib version (optional)
# Rules requiring newer GLib versions will be automatically disabled
min_glib_version = "2.40"

[rules]
# Ensure g_param_spec_* functions have NULL for nick and blurb parameters
g_param_spec_null_nick_blurb = true
```

See goblin.toml for all the supported rules/configurations.

## CI/CD Integration

### Container Image

goblin is available as a container image for easy CI/CD integration:

```bash
podman run --rm -v "$PWD:/workspace:Z" ghcr.io/bilelmoussaoui/goblin:latest
```

### GitHub Actions

Using the container image with GitHub Code Scanning:

```yaml
name: GObject Lint

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  lint:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/bilelmoussaoui/goblin:latest
    permissions:
      security-events: write  # Required for uploading SARIF results

    steps:
      - uses: actions/checkout@v4

      - name: Run goblin
        run: goblin --format sarif > goblin.sarif

      - name: Upload SARIF results
        if: always()
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: goblin.sarif
          category: goblin
```

The results will appear in the "Security" tab under "Code scanning alerts" for your repository, and as inline comments on pull requests.

### GitLab CI

Using the container image with GitLab's SARIF ingestion:

```yaml
goblin:
  stage: lint
  image: ghcr.io/bilelmoussaoui/goblin:latest
  script:
    - goblin --format sarif > goblin.sarif
  artifacts:
    reports:
      sarif: goblin.sarif
```

The results will appear in the merge request's security report and as inline comments.

### Installation Alternative

If you prefer installing locally instead of using containers:

```bash
cargo install --git https://github.com/bilelmoussaoui/goblin goblin
```

## LSP Server

For real-time linting in your editor:

```bash
cargo build --release --bin goblin-lsp
```

**Neovim** (nvim-lspconfig):
```lua
require('lspconfig.configs').gobject_lsp = {
  default_config = {
    cmd = {'goblin-lsp'},
    filetypes = {'c', 'h'},
    root_dir = require('lspconfig.util').root_pattern('goblin.toml', '.git'),
  },
}
require('lspconfig').gobject_lsp.setup{}
```

**VS Code**: Use a generic LSP client extension pointing to `goblin-lsp`

**Helix** (`~/.config/helix/languages.toml`):
```toml
[[language]]
name = "c"
language-servers = ["clangd", "goblin-lsp"]

[language-server.goblin-lsp]
command = "goblin-lsp"
```

Co-Authored by Claude Code.
