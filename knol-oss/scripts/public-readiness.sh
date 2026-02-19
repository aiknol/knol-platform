#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> Knol OSS public-readiness checks"

required_files=(
  "LICENSE"
  "README.md"
  "CONTRIBUTING.md"
  "SECURITY.md"
  ".env.example"
  ".github/workflows/ci.yml"
)

for file in "${required_files[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "ERROR: Missing required file: $file"
    exit 1
  fi
done

if git ls-files | rg -q '(^|/)\.env$'; then
  echo "ERROR: Tracked .env file detected. Remove it from git before publishing."
  exit 1
fi

echo "==> Running high-confidence secret scan (working tree)"
TREE_PATTERN='BEGIN (RSA|EC|OPENSSH|DSA|PRIVATE) KEY|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{35}|sk-[A-Za-z0-9]{32,}|T-Z_[A-Za-z0-9_-]{20,}'
if rg -n --hidden --glob '!.git' --glob '!target' --glob '!**/package-lock.json' --glob '!**/Cargo.lock' "$TREE_PATTERN" .; then
  echo "ERROR: Potential secret detected in working tree."
  exit 1
fi

echo "==> Running high-confidence secret scan (git history)"
HISTORY_PATTERN='BEGIN (RSA|EC|OPENSSH|DSA|PRIVATE) KEY|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|AKIA[0-9A-Z]{16}|ASIA[0-9A-Z]{16}|AIza[0-9A-Za-z_-]{35}|sk-[A-Za-z0-9]{32,}|T-Z_[A-Za-z0-9_-]{20,}'
history_hit=""
while IFS= read -r commit; do
  hit="$(git grep -I -nE "$HISTORY_PATTERN" "$commit" -- . || true)"
  if [[ -n "$hit" ]]; then
    history_hit="$hit"
    break
  fi
done < <(git rev-list --all)

if [[ -n "$history_hit" ]]; then
  echo "ERROR: Potential secret detected in git history."
  echo "$history_hit" | head -n 20
  exit 1
fi

echo "==> Checking for proprietary coupling references"
if rg -n "knol-enterprise|aiknol/knol-platform|private monorepo" . \
  --glob '!.git' \
  --glob '!target' \
  --glob '!scripts/*.sh'; then
  echo "ERROR: Found proprietary repository references in OSS tree."
  exit 1
fi

echo "==> Running Rust quality gates"
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

echo "✅ Public-readiness checks passed."
