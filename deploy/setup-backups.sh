#!/usr/bin/env bash
# =============================================================================
# Knol — Automated Backup Setup
# Sets up daily PostgreSQL backups via cron on the production VPS
# and uploads backups to off-host object storage (S3-compatible).
# =============================================================================
# Usage:
#   sudo ./deploy/setup-backups.sh
# =============================================================================

set -euo pipefail

BACKUP_DIR="/opt/knol/backups"
BACKUP_SCRIPT="/opt/knol/db-backup-prod.sh"
LOG_DIR="/var/log/knol"
CRON_USER="knol"

echo "=== Knol Backup Setup ==="

# Ensure required tools are installed
NEED_APT_UPDATE=0
if ! command -v pg_dump >/dev/null 2>&1; then
    NEED_APT_UPDATE=1
fi
if ! command -v aws >/dev/null 2>&1; then
    NEED_APT_UPDATE=1
fi
if [ "$NEED_APT_UPDATE" -eq 1 ]; then
    apt-get update -qq
fi
if ! command -v pg_dump >/dev/null 2>&1; then
    apt-get install -y -qq postgresql-client
fi
if ! command -v aws >/dev/null 2>&1; then
    apt-get install -y -qq awscli
fi

# Create directories
mkdir -p "$BACKUP_DIR" "$LOG_DIR"
chown "$CRON_USER":"$CRON_USER" "$BACKUP_DIR" "$LOG_DIR"

# Create the production backup wrapper script
cat > "$BACKUP_SCRIPT" <<'SCRIPT'
#!/bin/sh
# Production backup wrapper — reads /opt/knol/.env.production
set -eu

DEPLOY_DIR="/opt/knol"
ENV_FILE="$DEPLOY_DIR/.env.production"
BACKUP_DIR="$DEPLOY_DIR/backups"
LOG_FILE="/var/log/knol/backup.log"
umask 077

log() {
    echo "$(date -Iseconds) $1" >> "$LOG_FILE"
}

if [ ! -f "$ENV_FILE" ]; then
    log "ERROR: $ENV_FILE not found"
    exit 1
fi

env_get() {
    key="$1"
    line="$(grep -E "^${key}=" "$ENV_FILE" | tail -n 1 || true)"
    if [ -z "$line" ]; then
        echo ""
    else
        echo "${line#*=}"
    fi
}

# Core DB config
DATABASE_URL="$(env_get DATABASE_URL)"
if [ -z "$DATABASE_URL" ]; then
    log "ERROR: DATABASE_URL not found in $ENV_FILE"
    exit 1
fi

# Off-host backup config (S3-compatible storage)
BACKUP_REMOTE_ENABLED="$(env_get BACKUP_REMOTE_ENABLED)"
BACKUP_REMOTE_REQUIRED="$(env_get BACKUP_REMOTE_REQUIRED)"
BACKUP_S3_BUCKET="$(env_get BACKUP_S3_BUCKET)"
BACKUP_S3_REGION="$(env_get BACKUP_S3_REGION)"
BACKUP_S3_ENDPOINT="$(env_get BACKUP_S3_ENDPOINT)"
BACKUP_S3_PREFIX="$(env_get BACKUP_S3_PREFIX)"
BACKUP_S3_ACCESS_KEY_ID="$(env_get BACKUP_S3_ACCESS_KEY_ID)"
BACKUP_S3_SECRET_ACCESS_KEY="$(env_get BACKUP_S3_SECRET_ACCESS_KEY)"

[ -z "$BACKUP_REMOTE_ENABLED" ] && BACKUP_REMOTE_ENABLED="true"
[ -z "$BACKUP_REMOTE_REQUIRED" ] && BACKUP_REMOTE_REQUIRED="true"

if [ "$BACKUP_REMOTE_ENABLED" = "true" ]; then
    missing=""
    [ -z "$BACKUP_S3_BUCKET" ] && missing="${missing} BACKUP_S3_BUCKET"
    [ -z "$BACKUP_S3_ACCESS_KEY_ID" ] && missing="${missing} BACKUP_S3_ACCESS_KEY_ID"
    [ -z "$BACKUP_S3_SECRET_ACCESS_KEY" ] && missing="${missing} BACKUP_S3_SECRET_ACCESS_KEY"
    if [ -n "$missing" ]; then
        log "ERROR: missing required backup env vars:${missing}"
        exit 1
    fi
elif [ "$BACKUP_REMOTE_REQUIRED" = "true" ]; then
    log "ERROR: BACKUP_REMOTE_REQUIRED=true but BACKUP_REMOTE_ENABLED is not true"
    exit 1
fi

mkdir -p "$BACKUP_DIR"

TIMESTAMP="$(date -u +%Y%m%d_%H%M%S)"
BACKUP_FILE="${BACKUP_DIR}/knol_${TIMESTAMP}.dump"

log "Starting PostgreSQL backup"

if ! pg_dump \
    --dbname="$DATABASE_URL" \
    --format=custom \
    --compress=6 \
    --no-owner \
    --no-privileges \
    --file="$BACKUP_FILE" \
    2>>"$LOG_FILE"; then
    log "ERROR: pg_dump failed"
    exit 1
fi

SIZE="$(du -h "$BACKUP_FILE" | cut -f1)"
log "Backup complete: $BACKUP_FILE ($SIZE)"

if [ "$BACKUP_REMOTE_ENABLED" = "true" ]; then
    export AWS_ACCESS_KEY_ID="$BACKUP_S3_ACCESS_KEY_ID"
    export AWS_SECRET_ACCESS_KEY="$BACKUP_S3_SECRET_ACCESS_KEY"
    if [ -n "$BACKUP_S3_REGION" ]; then
        export AWS_DEFAULT_REGION="$BACKUP_S3_REGION"
    fi

    REMOTE_KEY="$(basename "$BACKUP_FILE")"
    if [ -n "$BACKUP_S3_PREFIX" ]; then
        REMOTE_KEY="${BACKUP_S3_PREFIX%/}/${REMOTE_KEY}"
    fi
    REMOTE_URI="s3://${BACKUP_S3_BUCKET}/${REMOTE_KEY}"

    log "Uploading backup to remote storage: $REMOTE_URI"
    if [ -n "$BACKUP_S3_ENDPOINT" ]; then
        aws --endpoint-url "$BACKUP_S3_ENDPOINT" s3 cp "$BACKUP_FILE" "$REMOTE_URI" --only-show-errors 2>>"$LOG_FILE"
        aws --endpoint-url "$BACKUP_S3_ENDPOINT" s3 ls "$REMOTE_URI" >/dev/null 2>>"$LOG_FILE"
    else
        aws s3 cp "$BACKUP_FILE" "$REMOTE_URI" --only-show-errors 2>>"$LOG_FILE"
        aws s3 ls "$REMOTE_URI" >/dev/null 2>>"$LOG_FILE"
    fi
    log "Remote upload verified: $REMOTE_URI"
fi

# Retain last 30 days of backups
DELETED="$(find "$BACKUP_DIR" -name "knol_*.dump" -mtime +30 -delete -print 2>/dev/null | wc -l)"
if [ "$DELETED" -gt 0 ]; then
    log "Cleaned up $DELETED old local backup(s)"
fi
SCRIPT

chmod +x "$BACKUP_SCRIPT"
chown "$CRON_USER":"$CRON_USER" "$BACKUP_SCRIPT"

# Install cron job — daily at 3:00 AM UTC
CRON_LINE="0 3 * * * $BACKUP_SCRIPT"
(crontab -u "$CRON_USER" -l 2>/dev/null | grep -v "$BACKUP_SCRIPT"; echo "$CRON_LINE") | crontab -u "$CRON_USER" -

echo ""
echo "Backup setup complete:"
echo "  Script:    $BACKUP_SCRIPT"
echo "  Backups:   $BACKUP_DIR"
echo "  Logs:      $LOG_DIR/backup.log"
echo "  Schedule:  Daily at 03:00 UTC"
echo "  Retention: 30 days (local)"
echo "  Remote:    S3-compatible upload enabled by .env.production"
echo ""
echo "Test with: sudo -u $CRON_USER $BACKUP_SCRIPT"
