#!/usr/bin/env bash
set -euo pipefail

# Pure Functional Git Status Checker
# Constraints: NO loops, NO mutable vars, ONLY recursion + pipes

# ============================================================
# PURE FUNCTIONS (no side effects except I/O)
# ============================================================

# Pure: check if dir is git repo
is_git_repo() {
	[[ -d "$1/.git" ]]
}

# Pure: check if repo is dirty (returns 0=dirty, 1=clean)
is_dirty() {
	is_git_repo "$1" &&
		(cd "$1" && [[ -n "$(git status --porcelain 2>/dev/null)" ]])
}

# Pure: get git status for dir
get_status() {
	(cd "$1" && git status --short 2>/dev/null)
}

# ============================================================
# RECURSIVE LIST PROCESSORS (tail recursion)
# ============================================================

# Recursive: process lines from stdin, check each repo
# Base case: no more input → done
# Recursive case: read one line, process it, recurse on rest
recursive_check() {
	local line
	if read -r line; then
		# Process head
		if is_git_repo "$line"; then
			echo "$line"
		fi
		# Recurse on tail
		recursive_check
	fi
}

# Recursive: filter dirty repos only
# Accumulates dirty repos through recursion
recursive_filter_dirty() {
	local line
	if read -r line; then
		# Process head: check if dirty
		is_dirty "$line" && echo "$line"
		# Recurse on tail
		recursive_filter_dirty
	fi
}

# Recursive: display repo status
# Pure display function with recursion
recursive_display() {
	local dir
	if read -r dir; then
		# Display head
		echo "📦 $(basename "$dir")" >&2
		get_status "$dir" | sed 's/^/  /' >&2
		echo "" >&2
		# Recurse on tail
		recursive_display
	fi
}

# Recursive: count lines (accumulator pattern)
# $1 = accumulator (current count)
recursive_count() {
	local acc="${1:-0}"
	local line
	if read -r line; then
		# Recurse with incremented accumulator
		recursive_count $((acc + 1))
	else
		# Base case: return accumulated count
		echo "$acc"
	fi
}

# ============================================================
# MAIN PIPELINE (pure function composition)
# ============================================================

main() {
	readonly PROJECTS_DIR="${HOME}/projects"

	echo "🔍 Checking git repos in $PROJECTS_DIR" >&2
	echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
	echo "" >&2

	# Functional pipeline: find → transform → filter → process
	# Step 1: Find all .git directories, get parent dirs
	readonly all_repos=$(
		find "$PROJECTS_DIR" -type d -name ".git" -maxdepth 3 |
			xargs -n1 dirname |
			recursive_check
	)

	# Step 2: Filter to dirty repos only (recursive filter)
	readonly dirty_repos=$(echo "$all_repos" | recursive_filter_dirty)

	# Step 3: Display dirty repos (recursive display)
	echo "$dirty_repos" | recursive_display

	# Step 4: Count results (recursive counting)
	readonly total_count=$(echo "$all_repos" | recursive_count)
	readonly dirty_count=$(echo "$dirty_repos" | recursive_count)

	echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
	echo "📊 Summary: $dirty_count dirty / $total_count repos" >&2

	# Return dirty count as exit code (capped at 255)
	return $((dirty_count > 255 ? 255 : dirty_count))
}

main "$@"
