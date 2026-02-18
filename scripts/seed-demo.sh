#!/bin/bash
set -euo pipefail

echo "=== Seeding Demo Data ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$ROOT_DIR/knol-demo"

if command -v python3 >/dev/null 2>&1; then
    python3 seed_data.py
elif command -v python >/dev/null 2>&1; then
    python seed_data.py
else
    echo "Python is required. Install from https://python.org"
    exit 1
fi

echo "Demo data seeded successfully."
