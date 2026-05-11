#!/usr/bin/env bash
# =============================================================================
# Knol — Production Deploy Script
# Run on the VPS to pull latest images and restart services
# =============================================================================
# Usage:
#   ./deploy.sh v0.2.1       # Deploy specific immutable tag
# =============================================================================

set -euo pipefail

DEPLOY_DIR="/opt/knol"
COMPOSE_FILE="$DEPLOY_DIR/docker-compose.prod.yml"
ENV_FILE="$DEPLOY_DIR/.env.production"
TAG="${1:-}"

cd "$DEPLOY_DIR"

echo "╔══════════════════════════════════════╗"
echo "║  Knol Deploy — tag: $TAG"
echo "╚══════════════════════════════════════╝"

if [ -z "$TAG" ]; then
    echo "ERROR: image tag is required."
    echo "Usage: ./deploy.sh <image-tag>"
    exit 1
fi
if [ "$TAG" = "latest" ]; then
    echo "ERROR: IMAGE_TAG must be an immutable release tag (not 'latest')."
    exit 1
fi

# Validate env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo "ERROR: $ENV_FILE not found."
    echo "Copy .env.production.example and fill in real values."
    exit 1
fi

get_env_value() {
    local key="$1"
    local line
    line="$(grep -E "^${key}=" "$ENV_FILE" | tail -n 1 || true)"
    if [ -z "$line" ]; then
        echo ""
    else
        echo "${line#*=}"
    fi
}

require_non_placeholder() {
    local key="$1"
    local value
    value="$(get_env_value "$key")"
    if [ -z "$value" ]; then
        echo "ERROR: $key is missing in $ENV_FILE"
        exit 1
    fi
    if [[ "$value" == replace-with-* ]] || [[ "$value" == *"USER:PASSWORD@HOST"* ]]; then
        echo "ERROR: $key appears to be a placeholder value in $ENV_FILE"
        exit 1
    fi
}

require_non_placeholder "DATABASE_URL"
require_non_placeholder "REDIS_URL"
require_non_placeholder "ADMIN_JWT_SECRET"
require_non_placeholder "ADMIN_ENCRYPTION_KEY"
require_non_placeholder "ADMIN_INITIAL_PASSWORD"
require_non_placeholder "BACKUP_REMOTE_ENABLED"
require_non_placeholder "BACKUP_REMOTE_REQUIRED"
require_non_placeholder "BACKUP_S3_BUCKET"
require_non_placeholder "BACKUP_S3_ACCESS_KEY_ID"
require_non_placeholder "BACKUP_S3_SECRET_ACCESS_KEY"
require_non_placeholder "BACKUP_S3_PREFIX"

admin_jwt_secret="$(get_env_value ADMIN_JWT_SECRET)"
if [ "${#admin_jwt_secret}" -lt 32 ]; then
    echo "ERROR: ADMIN_JWT_SECRET must be at least 32 characters"
    exit 1
fi

admin_encryption_key="$(get_env_value ADMIN_ENCRYPTION_KEY)"
decoded_len="$(printf '%s' "$admin_encryption_key" | base64 --decode 2>/dev/null | wc -c | tr -d ' ')"
if [ "$decoded_len" != "32" ]; then
    echo "ERROR: ADMIN_ENCRYPTION_KEY must be base64 for exactly 32 bytes."
    echo "Generate with: openssl rand -base64 32"
    exit 1
fi

backup_remote_enabled="$(get_env_value BACKUP_REMOTE_ENABLED)"
backup_remote_required="$(get_env_value BACKUP_REMOTE_REQUIRED)"
if [ "$backup_remote_enabled" != "true" ]; then
    echo "ERROR: BACKUP_REMOTE_ENABLED must be true in production"
    exit 1
fi
if [ "$backup_remote_required" != "true" ]; then
    echo "ERROR: BACKUP_REMOTE_REQUIRED must be true in production"
    exit 1
fi

if [ ! -x "$DEPLOY_DIR/db-backup-prod.sh" ]; then
    echo "ERROR: Backup script $DEPLOY_DIR/db-backup-prod.sh not found."
    echo "Run: sudo ./setup-backups.sh"
    exit 1
fi
backup_cron_present=0
if crontab -l 2>/dev/null | grep -q "$DEPLOY_DIR/db-backup-prod.sh"; then
    backup_cron_present=1
fi
if [ "$backup_cron_present" -eq 0 ] && crontab -u knol -l 2>/dev/null | grep -q "$DEPLOY_DIR/db-backup-prod.sh"; then
    backup_cron_present=1
fi
if [ "$backup_cron_present" -eq 0 ]; then
    echo "ERROR: Backup cron job is not configured for current user."
    echo "Run: sudo ./setup-backups.sh"
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
# Ensure NATS is running first (some services depend on it)
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d nats
sleep 2
# Run the gateway container to apply OSS migrations baked into /app/migrations/
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" \
    run --rm gateway \
    sh -c '
      if [ -d /app/migrations ] && ls /app/migrations/*.sql >/dev/null 2>&1; then
        echo "  Found migrations in /app/migrations/, ready for application."
        echo "  (Migrations are applied automatically on service startup via sqlx)"
      else
        echo "  No SQL migrations found, skipping."
      fi
    ' || echo "WARNING: Migration check failed, continuing..."

# ---------- Rolling restart ----------
echo ""
echo "[3/5] Restarting infrastructure (nats, minio)..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d nats minio
sleep 3

echo ""
echo "[4/5] Restarting application services..."
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d --force-recreate \
    gateway write retrieve graph \
    admin tenant jobs billing ingest marketing

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
    [tenant]=knol-tenant
    [jobs]=knol-jobs
    [billing]=knol-billing
    [ingest]=knol-enterprise-ingest
    [marketing]=knol-marketing
)
for svc in gateway write retrieve graph admin tenant jobs billing ingest marketing; do
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
