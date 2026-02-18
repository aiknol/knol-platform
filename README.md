# Knol OSS

Open-source memory infrastructure services for Knol.

## Services

- `gateway` (API): `http://localhost:3000`
- `write-service`: `http://localhost:8081/health`
- `retrieve-service`: `http://localhost:8082/health`
- `graph-service`: `http://localhost:8083/health`

## Local Run (Preferred)

Use the root stack:

```bash
cd /Users/dev/projects/knol/memorylayer
docker compose -f docker-compose.oss.yml up -d --build
```

## Notes

- Services run as prebuilt runtime images in the stack.
- Database pool sizing can be tuned with `DB_MAX_CONNECTIONS` and `DB_MIN_CONNECTIONS`.

## License

Apache License 2.0.
