# check-git-status (Rust)

Rust rewrite of the bash `check-git-status.sh` script with identical functionality.

## Features

- Recursively finds and checks git repositories
- Parallel processing for faster execution
- Three verbosity levels: quiet, summary, verbose
- Configurable search depth and root directory
- Returns exit code based on dirty repo count

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
- `-v, --verbose`: Show detailed git status for dirty repos
- `-h, --help`: Print help
- `-V, --version`: Print version

## Arguments

- `[path]`: Root directory to search (default: `$HOME/projects`)
- `[maxdepth]`: Maximum directory depth (default: 3)

## Exit Code

Returns the number of dirty repositories found (capped at 255).

## Performance

Uses parallel processing via Rayon for checking multiple repositories concurrently, making it faster than the sequential bash version on systems with multiple cores.

## Compatibility

Requires:
- Rust 1.70+ (2021 edition)
- Git installed and available in PATH
