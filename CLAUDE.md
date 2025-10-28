# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Task-Based Workflow (Recommended)
```bash
task                      # List all available tasks
task install-hooks        # Install pre-commit hooks (do this first)
task ci                   # Run full quality pipeline (fmt, lint, check, test)
task build                # Build release binary
task test                 # Run all tests (unit + integration)
task fmt                  # Check formatting
task lint                 # Run clippy
```

### Direct Cargo Commands (if Task not available)
```bash
cargo fmt -- --check      # Format check
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo check --all-targets --all-features   # Compilation check
cargo test                # Run tests (requires: cargo build --release first)
cargo build --release     # Build release binary
```

### Running Specific Tests
```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run specific test by name
cargo test test_clean_repo_quiet_mode
```

## Architecture & Module Design

### Modular Separation (Strict Boundaries)
- **cli.rs** (~200 lines): Pure argument parsing. Does NOT invoke git or format output.
- **core.rs** (~286 lines): Git operations and business logic. Does NOT format output.
- **output.rs** (~212 lines): Display formatting. Does NOT access filesystem or run commands.
- **error.rs** (~118 lines): Error types with Display impl. Pure data, no side effects.
- **main.rs** (~119 lines): Orchestration layer connecting modules.

**Critical**: This separation prevents circular dependencies and enables independent module testing.

### Exit Code Semantics (Non-Standard)
- **Exit code = dirty repository count** (0-255)
- `0` = all repositories clean (success)
- `N` = N dirty repositories found (still executable success, not error)
- `1` = validation error (invalid path, depth, etc.)
- Exit code is **data**, not traditional success/failure indicator
- Designed for shell scripting pipelines where exit code conveys meaningful information

### Data Flow
1. **Parse** (cli) → Extract flags, paths, validate arguments
2. **Validate** (core) → Canonicalize paths, validate depth bounds (1-100)
3. **Discover** (core) → WalkDir finds `.git` directories up to max_depth
4. **Check** (core) → Rayon parallel processing, run `git status --porcelain` per repo
5. **Format** (output) → Based on OutputFormat (Human/JSON) + Verbosity (Quiet/Summary/Verbose)
6. **Exit** (main) → Return dirty_count as exit code

### Three-Level Validation Strategy
1. **Argument validation** (cli.rs): Args struct methods, clap constraints
2. **Semantic validation** (core.rs): `validate_path()`, `validate_depth()` functions
3. **Runtime validation** (during execution): Git command results, IO operations

Each layer has specific responsibility without duplication.

### RepoStatus Enum Structure
```rust
pub enum RepoStatus {
    Clean { path, branch: Option<String> },
    Dirty { path, changes: String, branch: Option<String> }
}
```
- Tagged serialization with `#[serde(tag = "status")]`
- `branch` field optional (populated only with `--branch` flag)
- `changes` contains raw `git status --porcelain` output (not parsed)
- JSON omits None branch via `#[serde(skip_serializing_if = "Option::is_none")]`

### Parallel Processing Model
- Uses **Rayon** `par_iter()` for concurrent git checks
- Error resilience: Failed repos don't halt batch processing
- Results collected as `Vec<Result<T>>` then partitioned post-processing
- Errors separated from successes after parallel iteration completes
- No thread pool configuration (uses Rayon defaults)

### Output Format vs Verbosity (Orthogonal)
- **OutputFormat**: Human or JSON (affects structure)
- **Verbosity**: Quiet, Summary, Verbose (affects detail level)
- These are independent concerns
- JSON mode always outputs structured data regardless of verbosity
- Human mode respects verbosity: Quiet (exit code only), Summary (stats), Verbose (details)
- Header/summary printed to **stderr**, data to **stdout** (proper Unix convention)

## Testing Strategy

### Unit Tests (Inline in Source Files)
- Located in `#[cfg(test)] mod tests` within each module
- Test individual functions and data structures
- **cli.rs**: Args parsing, verbosity mapping, output format selection
- **core.rs**: Validation bounds (1-100 depth), RepoStatus methods, path canonicalization
- **output.rs**: Format equality, verbosity ordering, JSON serialization
- **error.rs**: Display impl for all variants, From trait conversions

### Integration Tests (tests/integration_tests.rs)
- Uses actual compiled binary via `CARGO_BIN_EXE_check-git-status`
- Creates temporary git repos with `tempfile` crate
- Tests real git command execution paths
- Helper: `create_temp_git_repo(name, dirty: bool)` - creates clean or dirty temp repos
- Tests: flags (--help, --version), modes (quiet, verbose, JSON), exit codes, invalid inputs
- **Important**: Integration tests require `cargo build --release` to run first

### Running Tests
```bash
# Pre-commit hook runs this (recommended workflow)
cargo build --release && cargo test

# Just unit tests (fast)
cargo test --lib

# Just integration tests
cargo test --test integration_tests

# Via Task (handles build dependency automatically)
task test
```

## Pre-Commit Hook System

### 5-Stage Quality Gate (scripts/pre-commit.hook)
1. **Format check**: `cargo fmt --check`
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Compilation**: `cargo check --all-targets --all-features`
4. **Tests**: `cargo test` (unit + integration, 36 tests)
5. **Release build**: `cargo build --release`

### Hook Behavior
- **Fail-fast**: Uses `set -e`, exits on first error
- **Colored output**: Progress indicators and status messages
- **Release build validation**: Ensures production binary works, not just debug
- **Bypass**: Use `git commit --no-verify` only in emergencies

### Installation
```bash
# Via Task (recommended)
task install-hooks

# Manual
cp scripts/pre-commit.hook .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Critical Implementation Details

### Path Handling
- **Default root**: `$HOME/projects` (NOT current directory)
- **Max depth**: 1-100 bounded (prevents runaway traversals)
- **Canonicalization**: All paths resolved via `path.canonicalize()` (resolves symlinks)
- **Validation**: Ensures path exists and is a directory before traversal

### Git Command Integration
- **Status check**: `git status --porcelain` (machine-readable format)
- **Branch detection**: `git rev-parse --abbrev-ref HEAD` (separate command)
- **Output interpretation**: Empty porcelain output = clean, any output = dirty
- **No parsing**: Individual change types not interpreted (M, A, D, etc.)

### Error Resilience Pattern
```rust
// Parallel processing collects Results
let results: Vec<Result<RepoStatus>> = repos.par_iter().map(...).collect();

// Partition AFTER parallel processing (not during)
for result in results {
    match result {
        Ok(status) => statuses.push(status),
        Err(e) => errors.push(e),
    }
}
```
- Single failed repo doesn't halt execution
- Errors reported at Summary+ verbosity
- Exit code based only on successful checks

### Shell Completion Generation
- Integrated into binary: `--generate-completion <bash|zsh|fish>`
- Uses `clap_complete` crate
- Completion generation is early-exit path (runs before repo discovery)
- Task automation: `task completions` generates all three to `completions/` directory

## Development Requirements

### Rust Version
- **Required**: Rust 1.85+ (edition 2024)
- **Edition 2024** features used throughout codebase
- Check: `rustc --version` (must be ≥1.85.0)

### External Dependencies
- **git**: Must be in PATH for status checks and branch detection
- **Task**: Optional but recommended (install via: `brew install go-task` or see taskfile.dev)

### Integration Test Prerequisites
- **Binary must exist**: `cargo build --release` before `cargo test`
- **Git binary**: Integration tests invoke real git commands
- **Tempfile cleanup**: Tests use `tempfile` crate, auto-cleaned on drop

### Development Workflow
1. **First time**: `task install-hooks` (activates quality gates)
2. **Make changes**: Edit code
3. **Commit**: Pre-commit hook runs 5-stage validation automatically
4. **If validation fails**: Fix issues, commit rejects until all checks pass
5. **CI equivalent**: `task ci` runs same checks as pre-commit hook

### When Adding New Features
- Add unit tests to relevant module (`#[cfg(test)] mod tests`)
- Add integration test if feature affects binary behavior
- Ensure changes pass all 5 pre-commit stages
- Update CLAUDE.md if architectural patterns change
