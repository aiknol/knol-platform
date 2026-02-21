# Knol Enterprise

Enterprise services for Knol.

## Services

- `admin-service`: `http://localhost:3001/health`
- `jobs-service`: background worker (no public HTTP endpoint)
- `billing-service`: `http://localhost:3003/health`
- `ingest-service`: `http://localhost:3004/health`

## Local Run (Preferred)

```bash
cd /Users/dev/projects/knol/memorylayer
export ADMIN_JWT_SECRET='replace-with-random-32-plus-char-secret'
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```

Admin web UI:

- `http://localhost:3006/` (admin website)
- `http://localhost:3007/` (tenant app website)
- `http://localhost:3008/` (demo UI)

## Notes

- `admin-panel-service` is removed from the stack.
- Enterprise crates depend on shared OSS crates in `../knol-oss/crates`.

## License

Commercial license.
