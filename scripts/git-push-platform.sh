#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# git-push-platform.sh — Push the full monorepo to PRIVATE repo
# Repo: github.com/aiknol/knol-platform  (private)
# ─────────────────────────────────────────────────────────────
set -euo pipefail

MONO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PRIVATE_REMOTE="origin"
PRIVATE_REPO="git@github.com:aiknol/knol-platform.git"
BRANCH="${1:-main}"

cd "$MONO_ROOT"

# ── Ensure origin points to the private repo ───────────────
CURRENT_URL=$(git remote get-url "$PRIVATE_REMOTE" 2>/dev/null || echo "")
if [[ -z "$CURRENT_URL" ]]; then
  echo "Adding remote '$PRIVATE_REMOTE' → $PRIVATE_REPO"
  git remote add "$PRIVATE_REMOTE" "$PRIVATE_REPO"
elif [[ "$CURRENT_URL" != "$PRIVATE_REPO" ]]; then
  echo "Updating remote '$PRIVATE_REMOTE' → $PRIVATE_REPO"
  git remote set-url "$PRIVATE_REMOTE" "$PRIVATE_REPO"
fi

# ── Push everything ────────────────────────────────────────
echo "Pushing full monorepo → $PRIVATE_REMOTE/$BRANCH …"
git push "$PRIVATE_REMOTE" "$BRANCH"

echo "✓ Platform push complete → github.com/aiknol/knol-platform ($BRANCH)"
