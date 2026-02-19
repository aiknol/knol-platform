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

echo "==> Rust tests"
(cd "$ROOT_DIR/knol-oss" && cargo test --workspace --lib)
(cd "$ROOT_DIR/knol-enterprise" && cargo test --workspace --lib)

echo "==> Web build"
(cd "$ROOT_DIR/knol-web" && npm ci && npm run build)

echo "✅ Local CI gates passed"
