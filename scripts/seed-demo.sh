#!/bin/bash
set -euo pipefail

echo "=== Seeding Demo Data ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

if command -v python3 >/dev/null 2>&1; then
    python3 "$ROOT_DIR/scripts/demo/seed_data.py"
elif command -v python >/dev/null 2>&1; then
    python "$ROOT_DIR/scripts/demo/seed_data.py"
else
    echo "Python is required. Install from https://python.org"
    exit 1
fi

echo "Demo data seeded successfully."
