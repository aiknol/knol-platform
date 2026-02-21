#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Running local CI gates"
echo "Root: $ROOT_DIR"

echo "==> Rust fmt checks"
(cd "$ROOT_DIR/knol-oss" && cargo fmt --all -- --check)
(cd "$ROOT_DIR/knol-enterprise" && cargo fmt --all -- --check)

echo "==> Rust checks"
cargo check --workspace --manifest-path "$ROOT_DIR/knol-oss/Cargo.toml"
cargo check --workspace --manifest-path "$ROOT_DIR/knol-enterprise/Cargo.toml"

echo "==> Rust clippy"
(cd "$ROOT_DIR/knol-oss" && cargo clippy --workspace --all-targets -- -D warnings)
(cd "$ROOT_DIR/knol-enterprise" && cargo clippy --workspace --all-targets -- -D warnings)

echo "==> OSS service-graph strict clippy"
(cd "$ROOT_DIR/knol-oss" && cargo clippy -p service-graph --all-targets -- -D warnings)

echo "==> Rust tests"
(cd "$ROOT_DIR/knol-oss" && cargo test --workspace --lib)
(cd "$ROOT_DIR/knol-enterprise" && cargo test --workspace --lib)

echo "==> Web builds"
(cd "$ROOT_DIR/frontend" && npm install --no-audit --no-fund)
(cd "$ROOT_DIR/frontend/web" && npm run build)
(cd "$ROOT_DIR/frontend/admin" && npm run build)
(cd "$ROOT_DIR/frontend/cloud" && npm run build)
(cd "$ROOT_DIR/frontend/demo" && npm run build)

echo "==> Frontend smoke checks"
"$ROOT_DIR/scripts/frontend-smoke.sh"

echo "✅ Local CI gates passed"
