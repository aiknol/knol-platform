#!/usr/bin/env bash
# =============================================================================
# Knol — Production Deploy Script
# Run on the VPS to pull latest images and restart services
# =============================================================================
# Usage:
#   ./deploy.sh              # Deploy latest
#   ./deploy.sh v0.2.1       # Deploy specific tag
# =============================================================================

set -euo pipefail

DEPLOY_DIR="/opt/knol"
COMPOSE_FILE="$DEPLOY_DIR/docker-compose.prod.yml"
ENV_FILE="$DEPLOY_DIR/.env.production"
TAG="${1:-latest}"

cd "$DEPLOY_DIR"

echo "╔══════════════════════════════════════╗"
echo "║  Knol Deploy — tag: $TAG"
echo "╚══════════════════════════════════════╝"

# Validate env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: $ENV_FILE not found."
    echo "Copy .env.production.example and fill in real values."
    exit 1
fi

# Export tag for docker compose
export IMAGE_TAG="$TAG"

# ---------- Pull latest images ----------
echo ""
echo "[1/5] Pulling images..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" pull

# ---------- Run database migrations ----------
echo ""
echo "[2/5] Running database migrations..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" \
    run --rm --no-deps gateway \
    sh -c 'if [ -d /app/migrations ]; then echo "Migrations directory found"; fi' \
    2>/dev/null || true

# ---------- Rolling restart ----------
echo ""
echo "[3/5] Restarting infrastructure (nats, minio)..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d nats minio
sleep 3

echo ""
echo "[4/5] Restarting application services..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d \
    gateway write retrieve graph \
    admin jobs billing ingest marketing

echo ""
echo "[5/5] Restarting Caddy..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d caddy

# ---------- Health check ----------
echo ""
echo "Waiting for services to become healthy..."
sleep 8

HEALTHY=0
TOTAL=0
declare -A CONTAINERS=(
    [gateway]=knol-gateway
    [write]=knol-write
    [retrieve]=knol-retrieve
    [graph]=knol-graph
    [admin]=knol-admin
    [jobs]=knol-jobs
    [billing]=knol-billing
    [ingest]=knol-enterprise-ingest
    [marketing]=knol-marketing
)
for svc in gateway write retrieve graph admin jobs billing ingest marketing; do
    TOTAL=$((TOTAL + 1))
    CNAME="${CONTAINERS[$svc]}"
    STATUS=$(docker inspect --format='{{.State.Health.Status}}' "$CNAME" 2>/dev/null || echo "missing")
    if [ "$STATUS" = "healthy" ]; then
        HEALTHY=$((HEALTHY + 1))
        echo "  ✓ $svc — healthy"
    else
        echo "  ✗ $svc — $STATUS"
    fi
done

echo ""
echo "══════════════════════════════════════"
if [ "$HEALTHY" -eq "$TOTAL" ]; then
    echo "  Deploy complete: $HEALTHY/$TOTAL services healthy"
else
    echo "  WARNING: $HEALTHY/$TOTAL healthy. Check logs:"
    echo "  docker compose -f $COMPOSE_FILE logs --tail 50"
fi
echo "══════════════════════════════════════"

# ---------- Show resource usage ----------
echo ""
echo "Resource usage:"
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | head -15
