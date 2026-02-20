#!/bin/bash
# =============================================================================
# Knol — Comprehensive Health Check
# Checks all infrastructure and application services
# =============================================================================
# Usage:
#   ./scripts/health-check.sh             # Local dev (default ports)
#   ./scripts/health-check.sh --prod      # Production (via Caddy)
# =============================================================================

set -euo pipefail

MODE="${1:-dev}"
HEALTHY=0
TOTAL=0
FAILED=()

check_service() {
    local name="$1"
    local url="$2"
    TOTAL=$((TOTAL + 1))
    if curl -sf --max-time 5 "$url" >/dev/null 2>&1; then
        HEALTHY=$((HEALTHY + 1))
        printf "  %-20s OK\n" "$name:"
    else
        FAILED+=("$name")
        printf "  %-20s FAIL\n" "$name:"
    fi
}

check_docker() {
    local name="$1"
    local container="$2"
    local cmd="$3"
    TOTAL=$((TOTAL + 1))
    if docker exec "$container" $cmd >/dev/null 2>&1; then
        HEALTHY=$((HEALTHY + 1))
        printf "  %-20s OK\n" "$name:"
    else
        FAILED+=("$name")
        printf "  %-20s FAIL\n" "$name:"
    fi
}

echo "=== Knol Health Check ($MODE) ==="
echo ""

# Infrastructure
echo "Infrastructure:"
if [ "$MODE" = "--prod" ]; then
    check_service "NATS" "http://localhost:8222/healthz"
    check_service "MinIO" "http://localhost:9000/minio/health/live"
else
    check_docker "Postgres" "ml-postgres" "pg_isready -U memory"
    check_docker "Redis" "ml-redis" "redis-cli ping"
    check_service "NATS" "http://localhost:8222/healthz"
    check_service "MinIO" "http://localhost:9000/minio/health/live"
fi
echo ""

# OSS Services
echo "OSS Services:"
if [ "$MODE" = "--prod" ]; then
    check_service "Gateway" "http://localhost:8080/health"
    check_service "Write" "http://localhost:8081/health"
    check_service "Retrieve" "http://localhost:8082/health"
    check_service "Graph" "http://localhost:8083/health"
else
    check_service "Gateway" "http://localhost:3000/health"
    check_service "Write" "http://localhost:8081/health"
    check_service "Retrieve" "http://localhost:8082/health"
    check_service "Graph" "http://localhost:8083/health"
fi
echo ""

# Enterprise Services
echo "Enterprise Services:"
if [ "$MODE" = "--prod" ]; then
    check_service "Admin" "http://localhost:8084/health"
    check_service "Jobs" "http://localhost:8085/health"
    check_service "Billing" "http://localhost:8086/health"
    check_service "Ingest" "http://localhost:8087/health"
    check_service "Marketing" "http://localhost:8088/health"
else
    check_service "Admin" "http://localhost:3001/health"
    check_service "Jobs" "http://localhost:8085/health"
    check_service "Billing" "http://localhost:3003/health"
    check_service "Ingest" "http://localhost:3004/health"
fi
echo ""

# Summary
echo "════════════════════════════════════"
if [ "$HEALTHY" -eq "$TOTAL" ]; then
    echo "  All $TOTAL services healthy"
else
    echo "  $HEALTHY/$TOTAL healthy"
    echo "  Failed: ${FAILED[*]}"
fi
echo "════════════════════════════════════"

# Exit with failure if any service is down
[ "$HEALTHY" -eq "$TOTAL" ] || exit 1
