#!/bin/sh
# Database restore script for Knol.
# Restores a compressed pg_dump backup into the database.
#
# Usage:
#   ./scripts/db-restore.sh <backup_file>
#   DB_HOST=prod-db ./scripts/db-restore.sh backups/knol_memory_20250101_120000.sql.gz
#
# Environment:
#   DB_HOST       - PostgreSQL host       (default: localhost)
#   DB_PORT       - PostgreSQL port       (default: 5432)
#   DB_USER       - PostgreSQL user       (default: memory)
#   DB_NAME       - PostgreSQL database   (default: memory)
#   PGPASSWORD    - PostgreSQL password   (default: memory_dev)
#   CONFIRM       - Skip confirmation     (set to "yes" to skip)

set -euo pipefail

DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-memory}"
DB_NAME="${DB_NAME:-memory}"
export PGPASSWORD="${PGPASSWORD:-memory_dev}"
CONFIRM="${CONFIRM:-}"

# Validate arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <backup_file>"
    echo ""
    echo "Examples:"
    echo "  $0 backups/knol_memory_20250101_120000.sql.gz"
    echo "  DB_HOST=prod-db $0 /mnt/backups/knol_memory_latest.sql.gz"
    exit 1
fi

BACKUP_FILE="$1"

if [ ! -f "${BACKUP_FILE}" ]; then
    echo "ERROR: Backup file not found: ${BACKUP_FILE}"
    exit 1
fi

SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)

echo "=== Knol Database Restore ==="
echo "Host:     ${DB_HOST}:${DB_PORT}"
echo "Database: ${DB_NAME}"
echo "File:     ${BACKUP_FILE} (${SIZE})"
echo ""

# Safety confirmation
if [ "${CONFIRM}" != "yes" ]; then
    echo "WARNING: This will DROP and recreate all tables in '${DB_NAME}'."
    echo "         All existing data will be replaced with the backup contents."
    echo ""
    printf "Type 'yes' to continue: "
    read -r answer
    if [ "${answer}" != "yes" ]; then
        echo "Aborted."
        exit 0
    fi
fi

# Wait for database to be ready
echo ""
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

echo "Restoring backup..."
START=$(date +%s)

# Decompress and pipe through pg_restore
# The backup was created with pg_dump --format=custom | gzip
# So we gunzip first, then use pg_restore with custom format
gunzip -c "${BACKUP_FILE}" | pg_restore \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d "${DB_NAME}" \
    --clean \
    --if-exists \
    --no-owner \
    --no-privileges \
    --single-transaction \
    --verbose 2>/dev/null || true

END=$(date +%s)
ELAPSED=$((END - START))

echo ""
echo "=== Restore Summary ==="
echo "File:     ${BACKUP_FILE}"
echo "Database: ${DB_NAME}"
echo "Duration: ${ELAPSED}s"
echo ""
echo "Restore complete. Run migrations if needed:"
echo "  ./scripts/db-migrate.sh"
