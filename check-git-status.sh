#!/usr/bin/env bash
set -euo pipefail

_DEFAULT_MAX_DEPTH=3
_DEFAULT_ROOT="$HOME/projects"

# If any arg is --help or -h, show help immediately
if printf '%s\n' "$@" | grep -qx -- '-h\|--help'; then
	cat >&2 <<EOF
Usage: check-git-status.sh [path] [maxdepth]

  path        Root directory to search (default: $_DEFAULT_ROOT)
  maxdepth    Maximum directory depth (default: $_DEFAULT_MAX_DEPTH)

Examples:
  check-git-status.sh
  check-git-status.sh ~/dev 2
  check-git-status.sh --help  # works anywhere
EOF
	exit 0
fi

# If any arg is --version or -v, show version
if printf '%s\n' "$@" | grep -qx -- '-v\|--version'; then
	echo "check-git-status.sh 1.2.0" >&2
	exit 0
fi

ROOT="${1:-$_DEFAULT_ROOT}"
MAXDEPTH="${2:-$_DEFAULT_MAX_DEPTH}"

# Validate numeric
if ! [[ "$MAXDEPTH" =~ ^[0-9]+$ ]]; then
	printf "❌ Invalid maxdepth: %s (must be a positive integer)\n" "$MAXDEPTH" >&2
	exit 1
fi

printf "🔍 Checking git repos in %s (maxdepth=%s)\n" "$ROOT" "$MAXDEPTH" >&2
printf '%*s\n\n' "$(tput cols)" '' | tr ' ' '━' >&2

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
	awk -v RS='\0' -F'\t' '
    BEGIN { total=0; dirty=0; ORS="" }

    # Process each record
    {
      total++
      if ($1 == "DIRTY") {
        dirty++
        # Display dirty repo
        repo_name = $2
        sub(/.*\//, "", repo_name)
        printf "📦 %s\n", repo_name > "/dev/stderr"

        # Display git status lines
        git_output = $3
        gsub(/\n/, "\n  ", git_output)
        printf "  %s\n\n", git_output > "/dev/stderr"
      }
    }

    END {
      printf "📦 Total repos: %d\n", total > "/dev/stderr"
      printf "📊 Dirty repos: %d\n", dirty > "/dev/stderr"

      # Exit with dirty count (capped at 255)
      exit_code = dirty > 255 ? 255 : dirty
      exit exit_code
    }
  '
