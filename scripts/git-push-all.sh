#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
# git-push-all.sh — Push to BOTH repos in one command
#   1. Private monorepo  → doaide/cortex-platform
#   2. Public OSS subtree → doaide/cortex
# ─────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BRANCH="${1:-main}"

echo "═══════════════════════════════════════════"
echo " Cortex — Dual-repo push ($BRANCH)"
echo "═══════════════════════════════════════════"
echo ""

echo "── Step 1: Private platform repo ──────────"
"$SCRIPT_DIR/git-push-platform.sh" "$BRANCH"
echo ""

echo "── Step 2: Public OSS repo ────────────────"
"$SCRIPT_DIR/git-push-oss.sh" "$BRANCH"
echo ""

echo "═══════════════════════════════════════════"
echo " All pushes complete."
echo "═══════════════════════════════════════════"
