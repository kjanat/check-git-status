#!/usr/bin/env bash
set -euo pipefail

VERSION="1.3.1"
SCRIPT_NAME="$(basename "$0")"
_DEFAULT_MAX_DEPTH=3
_DEFAULT_ROOT="$HOME/projects"
VERBOSITY=1 # 0=quiet, 1=summary, 2=verbose

# Parse flags
while [[ $# -gt 0 ]]; do
	case "$1" in
	-h | --help)
		cat >&2 <<EOF
Usage: $SCRIPT_NAME [OPTIONS] [path] [maxdepth]

Options:
  -q, --quiet     Only exit code (0=all clean, N=dirty count)
  -v, --verbose   Show detailed git status for dirty repos
  --help          Show this help message
  --version       Show version

Arguments:
  path            Root directory to search (default: $_DEFAULT_ROOT)
  maxdepth        Maximum directory depth (default: $_DEFAULT_MAX_DEPTH)

Examples:
  $SCRIPT_NAME              # summary only
  $SCRIPT_NAME -q           # silent, check exit code
  $SCRIPT_NAME -v           # verbose with details
  $SCRIPT_NAME ~/dev 2      # custom path/depth
EOF
		exit 0
		;;
	--version)
		echo "$SCRIPT_NAME $VERSION" >&2
		exit 0
		;;
	-q | --quiet)
		VERBOSITY=0
		shift
		;;
	-v | --verbose)
		VERBOSITY=2
		shift
		;;
	-*)
		printf "❌ Unknown option: %s\n" "$1" >&2
		exit 1
		;;
	*)
		break
		;;
	esac
done

ROOT="${1:-$_DEFAULT_ROOT}"
MAXDEPTH="${2:-$_DEFAULT_MAX_DEPTH}"

# Validate numeric
if ! [[ "$MAXDEPTH" =~ ^[0-9]+$ ]]; then
	printf "❌ Invalid maxdepth: %s (must be a positive integer)\n" "$MAXDEPTH" >&2
	exit 1
fi

# Header only for verbose mode
if [ "$VERBOSITY" -ge 2 ]; then
	printf "🔍 Checking git repos in %s (maxdepth=%s)\n" "$ROOT" "$MAXDEPTH" >&2
	printf '%*s\n\n' "$(tput cols)" '' | tr ' ' '━' >&2
fi

# OPTIMIZATION: Single find + parallel processing + single git status per repo
# Output format: STATUS<TAB>PATH<TAB>GIT_OUTPUT (NUL-terminated for safety)
find "$ROOT" -maxdepth "$MAXDEPTH" -type d -name .git -print0 |
	xargs -0 -n1 dirname |
	xargs -P"$(nproc)" -I{} sh -c '
    status=$(git -C "$1" status --porcelain 2>/dev/null || true)
    if [ -n "$status" ]; then
      printf "DIRTY\t%s\t%s\0" "$1" "$status"
    else
      printf "CLEAN\t%s\0" "$1"
    fi
  ' _ {} |
	awk -v RS='\0' -F'\t' -v VERBOSITY="$VERBOSITY" '
    BEGIN { total=0; dirty=0; ORS="" }

    # Process each record
    {
      total++
      if ($1 == "DIRTY") {
        dirty++

        # Verbose mode: show detailed status
        if (VERBOSITY >= 2) {
          repo_name = $2
          sub(/.*\//, "", repo_name)
          printf "📦 %s\n", repo_name > "/dev/stderr"

          git_output = $3
          gsub(/\n/, "\n  ", git_output)
          printf "  %s\n\n", git_output > "/dev/stderr"
        }
      }
    }

    END {
      # Summary mode: show counts only
      if (VERBOSITY >= 1) {
        printf "📦 Total repos: %d\n", total > "/dev/stderr"
        printf "📊 Dirty repos: %d\n", dirty > "/dev/stderr"
      }

      # Quiet mode: no output at all, just exit code

      # Exit with dirty count (capped at 255)
      exit_code = dirty > 255 ? 255 : dirty
      exit exit_code
    }
  '
