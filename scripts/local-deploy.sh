#!/bin/bash
set -euo pipefail

echo "=== Knol Local Deployment ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Start infrastructure
echo "Starting infrastructure..."
cd "$ROOT_DIR/knol-oss"
docker compose up -d

echo "Waiting for Postgres..."
until docker exec ml-postgres pg_isready -U memory 2>/dev/null; do sleep 1; done
echo "Postgres ready."

echo "Waiting for Redis..."
until docker exec ml-redis redis-cli ping 2>/dev/null | grep -q PONG; do sleep 1; done
echo "Redis ready."

echo "Waiting for NATS..."
sleep 3
echo "NATS ready."

# Build OSS
echo ""
echo "Building OSS services..."
cd "$ROOT_DIR/knol-oss"
cargo build --workspace --release 2>&1 | tail -3

# Start OSS services
echo "Starting OSS services..."
RUST_LOG=info cargo run --release --bin service-gateway &
GATEWAY_PID=$!
sleep 1

RUST_LOG=info cargo run --release --bin service-write &
WRITE_PID=$!

RUST_LOG=info cargo run --release --bin service-retrieve &
RETRIEVE_PID=$!

RUST_LOG=info cargo run --release --bin service-graph &
GRAPH_PID=$!

echo ""
echo "=== All Services Running ==="
echo "Gateway:  http://localhost:3000 (PID: $GATEWAY_PID)"
echo "Write:    (PID: $WRITE_PID)"
echo "Retrieve: (PID: $RETRIEVE_PID)"
echo "Graph:    (PID: $GRAPH_PID)"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for any process to exit
wait
