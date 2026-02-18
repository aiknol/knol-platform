#!/bin/sh
set -eu

DB_HOST="${DB_HOST:-postgres}"
DB_USER="${DB_USER:-memory}"
DB_NAME="${DB_NAME:-memory}"

echo "Waiting for postgres at ${DB_HOST}..."
until pg_isready -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" >/dev/null 2>&1; do
  sleep 1
done

echo "Ensuring migration tracking table exists..."
psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 <<'SQL'
CREATE TABLE IF NOT EXISTS schema_migrations (
  filename   TEXT PRIMARY KEY,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
SQL

bootstrap_existing_schema() {
  migration_count="$(psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -tA -c "SELECT COUNT(*) FROM schema_migrations" | tr -d '[:space:]')"
  tenants_exists="$(psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -tA -c "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'tenants')" | tr -d '[:space:]')"

  if [ "${migration_count}" = "0" ] && [ "${tenants_exists}" = "t" ]; then
    echo "Existing schema detected without migration history; bootstrapping schema_migrations..."
    for dir in /migrations/oss /migrations/enterprise; do
      find "${dir}" -maxdepth 1 -type f -name '*.sql' | sort | while IFS= read -r file; do
        prefix="$(basename "${dir}")"
        base="$(basename "${file}")"
        key="${prefix}/${base}"
        sql_key="$(printf "%s" "${key}" | sed "s/'/''/g")"
        psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -c "INSERT INTO schema_migrations(filename) VALUES ('${sql_key}') ON CONFLICT DO NOTHING" >/dev/null
      done
    done
  fi
}

mark_consolidated_if_legacy_applied() {
  baseline_key="$1"
  legacy_glob="$2"

  baseline_sql_key="$(printf "%s" "${baseline_key}" | sed "s/'/''/g")"
  baseline_applied="$(psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -tA -c "SELECT 1 FROM schema_migrations WHERE filename = '${baseline_sql_key}'" | tr -d '[:space:]')"
  if [ "${baseline_applied}" = "1" ]; then
    return
  fi

  legacy_sql_glob="$(printf "%s" "${legacy_glob}" | sed "s/'/''/g")"
  legacy_applied="$(psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -tA -c "SELECT 1 FROM schema_migrations WHERE filename LIKE '${legacy_sql_glob}' LIMIT 1" | tr -d '[:space:]')"
  if [ "${legacy_applied}" = "1" ]; then
    echo "Marking consolidated migration as applied: ${baseline_key}"
    psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -c "INSERT INTO schema_migrations(filename) VALUES ('${baseline_sql_key}') ON CONFLICT DO NOTHING" >/dev/null
  fi
}

apply_dir() {
  dir="$1"
  prefix="$2"

  find "${dir}" -maxdepth 1 -type f -name '*.sql' | sort | while IFS= read -r file; do
    base="$(basename "${file}")"
    key="${prefix}/${base}"
    sql_key="$(printf "%s" "${key}" | sed "s/'/''/g")"

    applied="$(psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -tA -c "SELECT 1 FROM schema_migrations WHERE filename = '${sql_key}'")"
    if [ "${applied}" = "1" ]; then
      echo "Skipping already applied migration: ${key}"
      continue
    fi

    echo "Applying migration: ${key}"
    psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -f "${file}"
    psql -h "${DB_HOST}" -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 -c "INSERT INTO schema_migrations(filename) VALUES ('${sql_key}')"
  done
}

bootstrap_existing_schema
mark_consolidated_if_legacy_applied "oss/001_baseline.sql" "oss/00%\\_%.sql"
mark_consolidated_if_legacy_applied "enterprise/006_baseline.sql" "enterprise/0%\\_%.sql"
apply_dir "/migrations/oss" "oss"
apply_dir "/migrations/enterprise" "enterprise"

echo "Migrations complete."
