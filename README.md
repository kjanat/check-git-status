# check-git-status

Fast, parallel git repository status checker written in Rust.

## Features

- **Parallel Processing**: Leverage multi-core CPUs for faster scanning
- **Color Output**: Beautiful, colorized terminal output for better readability
- **JSON Mode**: Machine-readable output for scripting and automation
- **Branch Information**: Show current branch names with status
- **Input Validation**: Path sanitization and depth bounds (1-100)
- **Robust Error Handling**: Detailed error messages with context
- **Shell Completions**: Auto-completion for bash, zsh, and fish
- **Modular Architecture**: Clean separation of concerns for maintainability

## Installation

```bash
cargo build --release
cp target/release/check-git-status ~/.local/bin/
# or install globally
cargo install --path .
```

## Usage

```bash
# Summary only (default)
check-git-status

# Quiet mode (exit code only)
check-git-status -q

# Verbose mode (show git status details)
check-git-status -v

# Custom path and depth
check-git-status ~/dev 2

# Show help
check-git-status --help

# Show version
check-git-status --version
```

## Options

- `-q, --quiet`: Only exit code (0=all clean, N=dirty count)
- `-v, --verbose`: Show detailed git status for all dirty repos
- `-j, --json`: Output results in JSON format
- `-b, --branch`: Show branch names in output
- `--generate-completion <SHELL>`: Generate shell completion script (bash, zsh, fish)
- `-h, --help`: Print help information
- `-V, --version`: Print version

## Arguments

- `[path]`: Root directory to search (default: `$HOME/projects`)
- `[maxdepth]`: Maximum directory depth (default: 3)

## Exit Code

Returns the number of dirty repositories found (capped at 255).

## Advanced Usage

### JSON Output

Perfect for scripting and automation:

```bash
# Get all dirty repos as JSON
check-git-status --json | jq '.repositories[] | select(.status == "dirty") | .path'

# Count dirty repos
check-git-status --json | jq '.dirty'
```

### Branch Information

Show branch names alongside status:

```bash
check-git-status -vb  # Verbose with branch names
```

### Shell Completions

Generate completion scripts for your shell:

```bash
# Bash
check-git-status --generate-completion bash > ~/.local/share/bash-completion/completions/check-git-status

# Zsh
check-git-status --generate-completion zsh > ~/.zfunc/_check-git-status

# Fish
check-git-status --generate-completion fish > ~/.config/fish/completions/check-git-status.fish
```

## Performance

Uses parallel processing via Rayon for checking multiple repositories concurrently, making it significantly faster than sequential approaches on multi-core systems.

## Compatibility

Requires:
- Rust 1.70+ (2021 edition)
- Git installed and available in PATH
