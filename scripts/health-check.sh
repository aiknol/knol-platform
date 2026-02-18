#!/bin/bash

echo "=== Knol Health Check ==="
echo ""

# Infrastructure
echo "Infrastructure:"
docker exec ml-postgres pg_isready -U memory 2>/dev/null && echo "  Postgres:  OK" || echo "  Postgres:  FAIL"
docker exec ml-redis redis-cli ping 2>/dev/null | grep -q PONG && echo "  Redis:     OK" || echo "  Redis:     FAIL"
curl -sf http://localhost:8222/varz >/dev/null 2>&1 && echo "  NATS:      OK" || echo "  NATS:      FAIL"
echo ""

# Services
echo "Services:"
curl -sf http://localhost:3000/health >/dev/null 2>&1 && echo "  Gateway:   OK (port 3000)" || echo "  Gateway:   FAIL"
echo ""

echo "Done."
