#!/bin/bash
set -euo pipefail

echo "=== Stopping Knol Services ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Stop Rust services
echo "Stopping service binaries..."
pkill -f "service-gateway" 2>/dev/null || true
pkill -f "service-write" 2>/dev/null || true
pkill -f "service-retrieve" 2>/dev/null || true
pkill -f "service-graph" 2>/dev/null || true
pkill -f "admin-service" 2>/dev/null || true
pkill -f "jobs-service" 2>/dev/null || true
pkill -f "billing-service" 2>/dev/null || true
pkill -f "ingest-service" 2>/dev/null || true

echo "Stopping frontend web surfaces..."
"$SCRIPT_DIR/frontend-services.sh" stop || true

# Stop infrastructure/backend
echo "Stopping Docker containers..."
cd "$ROOT_DIR"
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml down 2>/dev/null || true

echo "All services stopped."
