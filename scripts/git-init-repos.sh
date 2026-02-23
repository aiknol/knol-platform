#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# git-init-repos.sh — One-time setup for the dual-repo strategy
#
# Creates two GitHub repos under aiknol org:
#   1. aiknol/knol-platform  (private) — full monorepo
#   2. aiknol/knol           (public)  — OSS subtree
#
# Prerequisites:
#   - `gh` CLI authenticated with org write access
#   - SSH key configured for git@github.com
# ─────────────────────────────────────────────────────────────
set -euo pipefail

MONO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ORG="aiknol"
cd "$MONO_ROOT"

echo "═══════════════════════════════════════════"
echo " Knol — Repository initialization"
echo "═══════════════════════════════════════════"

# ── Step 1: Create GitHub repos if they don't exist ────────
echo ""
echo "── Creating GitHub repos ──────────────────"

if gh repo view "$ORG/knol-platform" &>/dev/null; then
  echo "  ✓ $ORG/knol-platform already exists"
else
  echo "  Creating $ORG/knol-platform (private)…"
  gh repo create "$ORG/knol-platform" \
    --private \
    --description "Knol platform monorepo — context engineering for LLM applications" \
    --confirm
fi

if gh repo view "$ORG/knol" &>/dev/null; then
  echo "  ✓ $ORG/knol already exists"
else
  echo "  Creating $ORG/knol (public)…"
  gh repo create "$ORG/knol" \
    --public \
    --description "Knol — open-source context engineering platform. Semantic + keyword + graph memory for LLMs." \
    --license Apache-2.0 \
    --confirm
fi

# ── Step 2: Initialize local git repo ─────────────────────
echo ""
echo "── Initializing local git ─────────────────"

if [[ -d .git ]]; then
  echo "  ✓ Git already initialized"
else
  git init -b main
  echo "  ✓ Git initialized (branch: main)"
fi

# ── Step 3: Configure remotes ─────────────────────────────
echo ""
echo "── Configuring remotes ────────────────────"

# origin → private monorepo
if git remote get-url origin &>/dev/null; then
  git remote set-url origin "git@github.com:$ORG/knol-platform.git"
else
  git remote add origin "git@github.com:$ORG/knol-platform.git"
fi
echo "  ✓ origin → git@github.com:$ORG/knol-platform.git"

# oss-public → public OSS repo
if git remote get-url oss-public &>/dev/null; then
  git remote set-url oss-public "git@github.com:$ORG/knol.git"
else
  git remote add oss-public "git@github.com:$ORG/knol.git"
fi
echo "  ✓ oss-public → git@github.com:$ORG/knol.git"

# ── Step 4: Initial commit ────────────────────────────────
echo ""
echo "── Creating initial commit ────────────────"

git add -A
git commit -m "Initial commit — Knol context engineering platform

Monorepo layout:
  knol-oss/        — open-source core (Apache-2.0)
  knol-enterprise/ — proprietary enterprise services
  frontend/web/        — main marketing website
  frontend/admin/      — admin frontend
  frontend/cloud/      — tenant cloud frontend
  frontend/demo/       — interactive demo frontend
  frontend/docs/       — public docs website
  private/docs/        — local-only private docs website
  scripts/         — tooling & deployment
  deploy/          — infrastructure configs"

echo "  ✓ Initial commit created"

# ── Step 5: Push to both repos ────────────────────────────
echo ""
echo "── Pushing to GitHub ──────────────────────"

echo "  Pushing to knol-platform (private)…"
git push -u origin main

echo "  Pushing OSS subtree to knol (public)…"
git subtree push --prefix=knol-oss oss-public main

echo ""
echo "═══════════════════════════════════════════"
echo " Setup complete!"
echo ""
echo " Private: https://github.com/$ORG/knol-platform"
echo " Public:  https://github.com/$ORG/knol"
echo ""
echo " Daily workflow:"
echo "   git add . && git commit -m '...'"
echo "   ./scripts/git-push-all.sh"
echo "═══════════════════════════════════════════"
