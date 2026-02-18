#!/bin/sh
# Database backup script for Knol.
# Creates a compressed pg_dump of the entire database.
#
# Usage:
#   ./scripts/db-backup.sh                     # Uses defaults
#   DB_HOST=prod-db ./scripts/db-backup.sh     # Custom host
#   BACKUP_DIR=/mnt/backups ./scripts/db-backup.sh
#
# Environment:
#   DB_HOST       - PostgreSQL host       (default: localhost)
#   DB_PORT       - PostgreSQL port       (default: 5432)
#   DB_USER       - PostgreSQL user       (default: memory)
#   DB_NAME       - PostgreSQL database   (default: memory)
#   PGPASSWORD    - PostgreSQL password   (default: memory_dev)
#   BACKUP_DIR    - Output directory      (default: ./backups)
#   BACKUP_RETAIN - Days to keep backups  (default: 30)

set -euo pipefail

DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-memory}"
DB_NAME="${DB_NAME:-memory}"
export PGPASSWORD="${PGPASSWORD:-memory_dev}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
BACKUP_RETAIN="${BACKUP_RETAIN:-30}"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/knol_${DB_NAME}_${TIMESTAMP}.sql.gz"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

echo "=== Knol Database Backup ==="
echo "Host:      ${DB_HOST}:${DB_PORT}"
echo "Database:  ${DB_NAME}"
echo "Output:    ${BACKUP_FILE}"
echo "Retention: ${BACKUP_RETAIN} days"
echo ""

# Wait for database to be ready
echo "Checking database connectivity..."
for i in $(seq 1 10); do
    if pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" > /dev/null 2>&1; then
        break
    fi
    if [ "$i" -eq 10 ]; then
        echo "ERROR: Database not reachable after 10 attempts"
        exit 1
    fi
    echo "  Waiting for database... (attempt $i/10)"
    sleep 2
done

# Create backup with compression
echo "Creating backup..."
START=$(date +%s)

pg_dump \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d "${DB_NAME}" \
    --format=custom \
    --compress=6 \
    --no-owner \
    --no-privileges \
    --verbose 2>/dev/null \
| gzip > "${BACKUP_FILE}"

END=$(date +%s)
ELAPSED=$((END - START))
SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)

echo "Backup complete: ${BACKUP_FILE} (${SIZE}, ${ELAPSED}s)"

# Clean up old backups
if [ "${BACKUP_RETAIN}" -gt 0 ]; then
    DELETED=$(find "${BACKUP_DIR}" -name "knol_*.sql.gz" -mtime "+${BACKUP_RETAIN}" -delete -print | wc -l)
    if [ "${DELETED}" -gt 0 ]; then
        echo "Cleaned up ${DELETED} backup(s) older than ${BACKUP_RETAIN} days"
    fi
fi

echo ""
echo "=== Backup Summary ==="
echo "File:     ${BACKUP_FILE}"
echo "Size:     ${SIZE}"
echo "Duration: ${ELAPSED}s"
echo "Total backups: $(find "${BACKUP_DIR}" -name "knol_*.sql.gz" | wc -l)"
