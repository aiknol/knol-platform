#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ ! -f "$ROOT_DIR/.githooks/pre-push" ]]; then
  echo "ERROR: Missing $ROOT_DIR/.githooks/pre-push"
  exit 1
fi

chmod +x "$ROOT_DIR/.githooks/pre-push"
chmod +x "$ROOT_DIR/scripts/ci-local.sh"

git -C "$ROOT_DIR" config core.hooksPath .githooks

echo "✅ Git hooks installed."
echo "Hooks path: $(git -C "$ROOT_DIR" config --get core.hooksPath)"
