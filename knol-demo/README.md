# Knol Demo

Static demo UI used with the Knol stack.

## Run

```bash
cd /Users/dev/projects/knol/memorylayer
export ADMIN_JWT_SECRET='replace-with-random-32-plus-char-secret'
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```

Demo URL:

- `http://localhost:8080`

Optional app URLs:

- Gateway: `http://localhost:3000`
- Website: `http://localhost:3005`
