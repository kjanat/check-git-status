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
	echo "check-git-status.sh 1.1.2" >&2
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

find "$ROOT" -maxdepth "$MAXDEPTH" -type d -name .git -print0 |
	xargs -0 -n1 dirname |
	tee >(
		awk 'END{print "📦 Total repos: " NR}' >&2
	) |
	xargs -I{} sh -c '
    if test -n "$(git -C "$1" status --porcelain 2>/dev/null)"; then
      printf "📦 %s\n" "$(basename "$1")" >&2
      git -C "$1" status --short 2>/dev/null | sed "s/^/  /" >&2
      echo >&2
      echo "$1"
    fi
  ' _ {} |
	tee >(
		awk 'END{print "📊 Dirty repos: " NR}' >&2
	) >/dev/null

exit "$(
	find "$ROOT" -maxdepth "$MAXDEPTH" -type d -name .git -print0 |
		xargs -0 -n1 dirname |
		xargs -I{} sh -c '
      test -n "$(git -C "$1" status --porcelain 2>/dev/null)" && echo .
    ' _ {} |
		awk 'END{n=NR+0; if(n>255)n=255; print n}'
)"
