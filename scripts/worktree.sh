#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Sibling-worktree helper (see AGENTS.md "Worktrees").
#   scripts/worktree.sh add <slug> [base-branch]   -> ../<project>-<slug> on branch feature/<slug>
#   scripts/worktree.sh remove <slug>              -> removes worktree, deletes branch if merged
#   scripts/worktree.sh list
set -euo pipefail
top="$(git rev-parse --show-toplevel)"
main="$(git worktree list --porcelain | awk '/^worktree /{print $2; exit}')"
project="$(basename "$main")"
cmd="${1:-list}"; slug="${2:-}"
dir="$(dirname "$main")/${project}-${slug}"
case "$cmd" in
  add)    [ -n "$slug" ] || { echo "usage: $0 add <slug> [base]" >&2; exit 2; }
          git -C "$main" worktree add -b "feature/$slug" "$dir" "${3:-main}"
          echo "created $dir (run 'cargo build' there; target/ is per-worktree)" ;;
  remove) [ -n "$slug" ] || { echo "usage: $0 remove <slug>" >&2; exit 2; }
          git -C "$main" worktree remove "$dir"
          git -C "$main" branch -d "feature/$slug" ;;
  list)   git -C "$top" worktree list ;;
  *)      echo "usage: $0 add|remove|list" >&2; exit 2 ;;
esac
