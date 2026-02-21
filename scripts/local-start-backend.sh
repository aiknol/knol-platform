#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required."
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "Docker daemon is not running. Start Docker and retry."
  exit 1
fi

cd "$ROOT_DIR"

echo "Starting backend services (Docker)..."
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d \
  postgres redis nats minio db-migrate \
  write-service retrieve-service graph-service gateway \
  admin-service jobs-service billing-service ingest-service

wait_for_http() {
  local url="$1"
  local attempts=60
  local i
  for ((i=1; i<=attempts; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

echo "Waiting for backend health endpoints..."
wait_for_http "http://127.0.0.1:3000/health" || { echo "Gateway failed (3000)."; exit 1; }
wait_for_http "http://127.0.0.1:3001/health" || { echo "Admin service failed (3001)."; exit 1; }
wait_for_http "http://127.0.0.1:3003/health" || { echo "Billing service failed (3003)."; exit 1; }
wait_for_http "http://127.0.0.1:3004/health" || { echo "Ingest service failed (3004)."; exit 1; }

echo "Backend services are up:"
echo "  Gateway      : http://localhost:3000/health"
echo "  Admin API    : http://localhost:3001/health"
echo "  Billing      : http://localhost:3003/health"
echo "  Ingest       : http://localhost:3004/health"
echo "  PostgreSQL   : localhost:5432"
echo "  Redis        : localhost:6379"
echo "  NATS         : localhost:4222"
