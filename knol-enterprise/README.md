# Knol Enterprise

Enterprise extensions for [Knol](https://github.com/aiknol/knol-platform) — the context engineering platform for AI applications.

## Services

| Service | Port | Description |
|---------|------|-------------|
| `admin-service` | 8084 | Admin API, demo endpoints, system config |
| `tenant-service` | 8085 | Multi-tenant workspace management |
| `billing-service` | 8086 | Usage tracking, plan enforcement, Stripe |
| `jobs-service` | — | Background job processing (NATS consumer) |
| `ingest-service` | 8087 | Bulk memory ingestion pipeline |
| `marketing-service` | 8088 | Marketing automation and campaigns |

## Local Development

```bash
# From the repository root
cp .env.example .env
# Edit .env with your secrets

docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```

Frontend:

- Admin Panel: `http://localhost:3006/`
- Cloud Dashboard: `http://localhost:3007/`
- Demo UI: `http://localhost:3008/`

## Architecture

Enterprise crates depend on shared OSS crates in `../knol-oss/crates/` for database access, caching, queueing, and common types.

## License

Source-available. See [LICENSE](LICENSE) for terms.
