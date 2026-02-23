# Tenant Service Guide

Self-service tenant API and workspace features for `cloud.aiknol.com`.

## Purpose

The tenant service powers signup/login, workspace management, API keys, billing, team invites, and profile/settings for each company tenant.

## Base URLs

- API base: `https://api.aiknol.com/app`
- OpenAPI / Swagger UI: `https://api.aiknol.com/docs`
- Tenant app website: `https://cloud.aiknol.com`

## Authentication Model

- Session cookie flow (browser app):
  - `POST /app/auth/signup`
  - `POST /app/auth/login`
  - `POST /app/auth/logout`
  - `GET /app/auth/me`
- Bearer token flow (API clients):
  - `Authorization: Bearer <token>`
- Tenant scoping:
  - all protected endpoints are tenant-scoped through authenticated claims.

## Endpoint Groups

### Auth

- `POST /app/auth/signup`
- `POST /app/auth/login`
- `POST /app/auth/logout`
- `GET /app/auth/me`
- `POST /app/auth/accept-invite`

### Workspace and Users

- `GET /app/tenant`
- `GET /app/users`
- `POST /app/users`
- `PUT /app/users/{id}`

### API Keys

- `GET /app/api-keys`
- `POST /app/api-keys`
- `DELETE /app/api-keys/{id}`

### Team Invites

- `POST /app/invites`
- `GET /app/invites`
- `DELETE /app/invites/{id}`
- `POST /app/auth/accept-invite`

### Billing and Usage

- `POST /app/billing/checkout`
- `POST /app/billing/portal`
- `GET /app/billing/subscription`
- `POST /app/billing/cancel`
- `POST /app/billing/reactivate`
- `GET /app/billing/invoices`
- `GET /app/billing/invoices/upcoming`
- `GET /app/billing/usage`
- `GET /app/billing/usage/history`
- `POST /app/billing/stripe/webhook`

### Settings

- `PUT /app/settings/tenant`
- `PUT /app/settings/profile`
- `POST /app/settings/change-password`

## Role Model

Workspace user roles:

- `owner`
- `admin`
- `developer`
- `viewer`

API key roles:

- `admin`
- `developer`
- `read_only`

## Reference Sources

- Tenant service code: `knol-enterprise/crates/service-tenant/`
- OpenAPI definition: `knol-enterprise/crates/service-tenant/src/openapi.rs`
- Integration tests covering endpoint behavior: `knol-enterprise/crates/service-tenant/tests/tenant_api_test.rs`
