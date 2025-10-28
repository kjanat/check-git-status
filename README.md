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

## Development

This project uses [Task](https://taskfile.dev) for build automation and development workflows.

### Installing Task

```bash
# macOS/Linux via Homebrew
brew install go-task

# Linux via install script
sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d -b ~/.local/bin

# Other installation methods: https://taskfile.dev/installation
```

### Available Tasks

Run `task` or `task --list` to see all available tasks:

```bash
task default              # Show available tasks
task install-hooks        # Install git pre-commit hooks
task fmt                  # Check code formatting
task fmt-fix (format)     # Fix code formatting
task lint                 # Run clippy linter
task check                # Check compilation
task test                 # Run all tests
task build                # Build release binary
task dev                  # Build debug binary
task ci                   # Run all CI checks
task clean                # Clean build artifacts
task watch                # Watch and rebuild on changes
task completions          # Generate shell completions
task install              # Install to ~/.local/bin
task uninstall            # Uninstall from ~/.local/bin
task run                  # Run the binary
task help                 # Show binary help
```

### Pre-commit Hooks

This project includes a comprehensive pre-commit hook that runs quality checks before allowing commits. The hook automatically runs:

1. **Format Check** (`cargo fmt --check`) - Ensures code formatting is consistent
2. **Linting** (`cargo clippy`) - Catches common mistakes and style issues
3. **Compilation** (`cargo check`) - Verifies code compiles successfully
4. **Tests** (`cargo test`) - Runs all 36 unit and integration tests
5. **Release Build** (`cargo build --release`) - Ensures production build works

#### Installing the Hook

```bash
# Install git hooks using Task
task install-hooks
```

This copies the pre-commit hook from `scripts/pre-commit.hook` to `.git/hooks/pre-commit`.

#### Bypassing the Hook

In rare cases where you need to commit without running checks (not recommended):

```bash
git commit --no-verify -m "your message"
```

#### What Happens During Commit

When you run `git commit`, you'll see output like:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Pre-commit Quality Checks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

▶ Checking code formatting...
✓ Code formatting is correct
▶ Running clippy (linter)...
✓ Clippy checks passed
▶ Running cargo check...
✓ Compilation successful
▶ Running tests...
✓ All tests passed (36 tests)
▶ Building release binary...
✓ Release build successful

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ All quality checks passed!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

If any check fails, the commit will be blocked with a helpful error message.

## Compatibility

Requires:
- Rust 1.85+ (2024 edition)
- Git installed and available in PATH
- Task (optional, for development workflows)
