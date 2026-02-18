#!/bin/bash
set -euo pipefail

echo "=== Knol Dev Environment Setup ==="

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "Docker is required. Install from https://docker.com"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Rust is required. Install from https://rustup.rs"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Node.js is required. Install from https://nodejs.org"; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo ""
echo "1. Starting infrastructure..."
cd "$ROOT_DIR/knol-oss"
docker compose up -d
echo "   Waiting for services to be ready..."
sleep 5

echo ""
echo "2. Building OSS crates..."
cargo build --workspace

echo ""
echo "3. Building Enterprise crates..."
cd "$ROOT_DIR/knol-enterprise"
cargo build --workspace

echo ""
echo "4. Installing website dependencies..."
cd "$ROOT_DIR/knol-web"
npm install

echo ""
echo "=== Setup Complete ==="
echo ""
echo "To start services:"
echo "  cd knol-oss && cargo run --bin service-gateway &"
echo "  cd knol-oss && cargo run --bin service-write &"
echo "  cd knol-oss && cargo run --bin service-retrieve &"
echo "  cd knol-oss && cargo run --bin service-graph &"
echo ""
echo "To start website dev server:"
echo "  cd knol-web && npm run dev"
echo ""
echo "API available at: http://localhost:3000"
echo "Website dev at:   http://localhost:3001"
