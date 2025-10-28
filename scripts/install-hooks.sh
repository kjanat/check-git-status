#!/usr/bin/env bash
# Install git hooks for check-git-status development

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing git hooks..."

# Copy pre-commit hook
if [ -f "$SCRIPT_DIR/pre-commit.hook" ]; then
    cp "$SCRIPT_DIR/pre-commit.hook" "$HOOKS_DIR/pre-commit"
    chmod +x "$HOOKS_DIR/pre-commit"
    echo "✓ Installed pre-commit hook"
else
    echo "✗ pre-commit.hook not found in scripts directory"
    exit 1
fi

echo ""
echo "Git hooks installed successfully!"
echo "Commits will now run: fmt, clippy, check, test, and build"
echo ""
echo "To bypass hooks (not recommended): git commit --no-verify"
