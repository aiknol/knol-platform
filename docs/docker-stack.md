# Docker Stack Guide

This guide covers local startup with separated OSS and proprietary compose files.

## Start

OSS only:

```bash
cd /Users/dev/projects/knol/memorylayer
docker compose -f docker-compose.oss.yml up -d --build
```

OSS + proprietary overlay:

```bash
export ADMIN_JWT_SECRET='replace-with-random-32-plus-char-secret'
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```

Start only proprietary services (requires OSS already up):

```bash
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d admin-service jobs-service billing-service ingest-service admin-website website demo-ui
```

## Endpoints

- Gateway: `http://localhost:3000`
- Write Service: `http://localhost:8081/health`
- Retrieve Service: `http://localhost:8082/health`
- Graph Service: `http://localhost:8083/health`
- Admin Service: `http://localhost:3001/health`
- Admin Website: `http://localhost:3006/` (redirects to `/admin/`)
- Billing Service: `http://localhost:3003/health`
- Ingest Service: `http://localhost:3004/health`
- Website: `http://localhost:3005`
- Demo UI: `http://localhost:8080`
- NATS monitor: `http://localhost:8222/healthz`

## Notes

- `db-migrate` is one-shot and should exit with code `0`.
- Rust services run from prebuilt multi-stage runtime images (`knol-oss-runtime:local`, `knol-enterprise-runtime:local`).
- First `--build` may take several minutes (dependency compile), then restarts are fast.
- Current optimized runtime image sizes are approximately: OSS `142MB`, Enterprise `135MB`.
- Services are memory-capped via `mem_limit` in compose files.
- DB pools are memory-tuned with:
  - `DB_MAX_CONNECTIONS` (service max pool size)
  - `DB_MIN_CONNECTIONS` (service min pool size; default `1`)
- If ports are already in use (for example older `memory-*` containers), stop conflicting containers before startup.
- Admin web UI expects `NEXT_PUBLIC_ADMIN_API_URL` (defaults to `http://localhost:3001` in this stack).
- `ADMIN_JWT_SECRET` is required for `admin-service` and must be at least 32 characters.
- Admin CORS is restricted by `ADMIN_CORS_ORIGIN` (default `http://localhost:3006`).

## Verify

```bash
docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}' | rg '^knol-'
```

## Stop

```bash
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml down
```
