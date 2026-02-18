#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# git-push-oss.sh — Push knol-oss subtree to the PUBLIC repo
# Repo: github.com/aiknol/knol  (public, Apache-2.0)
# ─────────────────────────────────────────────────────────────
set -euo pipefail

MONO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OSS_PREFIX="knol-oss"
PUBLIC_REMOTE="oss-public"
PUBLIC_REPO="git@github.com:aiknol/knol.git"
BRANCH="${1:-main}"

cd "$MONO_ROOT"

# ── Ensure the public remote exists ────────────────────────
if ! git remote get-url "$PUBLIC_REMOTE" &>/dev/null; then
  echo "Adding remote '$PUBLIC_REMOTE' → $PUBLIC_REPO"
  git remote add "$PUBLIC_REMOTE" "$PUBLIC_REPO"
fi

# ── Safety: ensure we're on the expected branch ────────────
CURRENT=$(git branch --show-current)
if [[ "$CURRENT" != "$BRANCH" && "$CURRENT" != "main" ]]; then
  echo "⚠ You're on branch '$CURRENT'. Pushing subtree from '$BRANCH'."
fi

# ── Push the subtree ──────────────────────────────────────
echo "Pushing '$OSS_PREFIX/' → $PUBLIC_REMOTE/$BRANCH …"
git subtree push --prefix="$OSS_PREFIX" "$PUBLIC_REMOTE" "$BRANCH"

echo "✓ OSS push complete → github.com/aiknol/knol ($BRANCH)"
