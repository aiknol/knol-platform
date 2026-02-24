#!/usr/bin/env bash
# =============================================================================
# E2E Test Runner
# =============================================================================
# Runs end-to-end tests against the running Docker Compose stack.
# Requires: gateway (port 3000), tenant-service, write-service, retrieve-service,
#           PostgreSQL, Redis, and NATS to be running via docker compose.
#
# Usage:
#   ./scripts/e2e-tests.sh              # run all e2e test modules
#   ./scripts/e2e-tests.sh <filter>     # run tests matching filter
#
# Environment:
#   GATEWAY_URL   - gateway base URL (default: http://localhost:3000)
#   E2E_THREADS   - test thread count (default: 1)
# =============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

GATEWAY_URL="${GATEWAY_URL:-http://localhost:3000}"
E2E_THREADS="${E2E_THREADS:-1}"
FILTER="${1:-}"

# ── Preflight: verify Docker services are reachable ─────────────────────────

echo "  Checking gateway at $GATEWAY_URL ..."
if ! curl -sf --max-time 5 "$GATEWAY_URL/health" >/dev/null 2>&1; then
  echo ""
  echo "ERROR: Gateway is not reachable at $GATEWAY_URL/health"
  echo "Make sure Docker Compose services are running:"
  echo "  docker compose up -d"
  echo ""
  exit 1
fi

# ── Flush Redis rate-limit counters ─────────────────────────────────────────

echo "  Flushing Redis rate-limit counters ..."
docker exec knol-redis-1 redis-cli FLUSHDB >/dev/null 2>&1 || true

# ── Define the test modules to run ──────────────────────────────────────────

TEST_MODULES=(
  "test_signup_flow"
  "test_session_management"
  "test_memory_crud"
  "test_user_management"
  "test_team_invites"
  "test_graph_api"
  "test_webhooks"
  "test_tenant_settings"
)

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_IGNORED=0
FAILED_MODULES=()

# ── Run each module ─────────────────────────────────────────────────────────

for module in "${TEST_MODULES[@]}"; do
  # If a filter was given, skip modules that don't match
  if [[ -n "$FILTER" ]] && [[ "$module" != *"$FILTER"* ]]; then
    continue
  fi

  echo ""
  echo "  ── $module ──"

  # Flush Redis between modules to avoid rate-limit bleed
  docker exec knol-redis-1 redis-cli FLUSHDB >/dev/null 2>&1 || true

  set +e
  OUTPUT=$(GATEWAY_URL="$GATEWAY_URL" cargo test \
    --manifest-path "$ROOT_DIR/tests/e2e/Cargo.toml" \
    -- --test-threads="$E2E_THREADS" "${module}::" 2>&1)
  EXIT_CODE=$?
  set -e

  # Parse pass/fail/ignored counts from cargo test output.
  # Cargo prints multiple "test result:" lines (lib, integration, doc-tests).
  # Sum across all of them to get the real totals for this module.
  PASS=0; FAIL=0; IGNORED=0
  while IFS= read -r RESULT_LINE; do
    P=$(echo "$RESULT_LINE" | grep -oE '[0-9]+ passed'  | grep -oE '[0-9]+' || echo 0)
    F=$(echo "$RESULT_LINE" | grep -oE '[0-9]+ failed'  | grep -oE '[0-9]+' || echo 0)
    I=$(echo "$RESULT_LINE" | grep -oE '[0-9]+ ignored' | grep -oE '[0-9]+' || echo 0)
    PASS=$((PASS + P)); FAIL=$((FAIL + F)); IGNORED=$((IGNORED + I))
  done < <(echo "$OUTPUT" | grep "^test result:")

  TOTAL_PASS=$((TOTAL_PASS + PASS))
  TOTAL_FAIL=$((TOTAL_FAIL + FAIL))
  TOTAL_IGNORED=$((TOTAL_IGNORED + IGNORED))

  if [[ $EXIT_CODE -ne 0 ]]; then
    FAILED_MODULES+=("$module")
    echo "  FAIL  $module  ($PASS passed, $FAIL failed, $IGNORED ignored)"
    # Print failure details
    echo "$OUTPUT" | grep -A2 "^---- .* stdout ----" || true
  else
    echo "  OK    $module  ($PASS passed, $IGNORED ignored)"
  fi
done

# ── Summary ─────────────────────────────────────────────────────────────────

echo ""
echo "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  E2E Results: $TOTAL_PASS passed, $TOTAL_FAIL failed, $TOTAL_IGNORED ignored"
echo "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ ${#FAILED_MODULES[@]} -gt 0 ]]; then
  echo ""
  echo "  Failed modules:"
  for m in "${FAILED_MODULES[@]}"; do
    echo "    - $m"
  done
  echo ""
  exit 1
fi
