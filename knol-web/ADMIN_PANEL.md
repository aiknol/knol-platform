# Knol Admin Panel

## Purpose

Web UI for platform administration backed by `admin-service`.

## URLs

- Admin web: `http://localhost:3006/`
- Admin pages base: `http://localhost:3006/admin/`
- Admin API: `http://localhost:3001`

## Core Routes

- `/admin/login`
- `/admin`
- `/admin/config`
- `/admin/credentials`
- `/admin/campaigns`
- `/admin/tenants`
- `/admin/users`
- `/admin/audit`

## Config UX

- Typed config editor (string, number, boolean, json, string_array)
- Search + category filter for faster updates
- Dedicated default LLM provider control in `/admin/config` that updates `llm.provider`

## Backend Endpoints (Required)

- `POST /admin/auth/login`
- `POST /admin/auth/logout`
- `POST /admin/auth/change-password`
- `GET/PUT/DELETE /admin/config/*`
- `GET/PUT/DELETE /admin/credentials/*`
- `GET/PUT /admin/campaigns/*`
- `GET/PUT /admin/tenants/*`
- `GET/POST/PUT/DELETE /admin/users/*`
- `GET /admin/audit`
- `GET /admin/status`

## Local Run

```bash
cd /Users/dev/projects/knol/memorylayer
export ADMIN_JWT_SECRET='replace-with-random-32-plus-char-secret'
docker compose -f docker-compose.oss.yml -f docker-compose.proprietary.yml up -d --build
```
