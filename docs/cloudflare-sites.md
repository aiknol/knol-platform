# Cloudflare Frontend Deployment

Deploy each frontend from its own directory as an independent Cloudflare Pages project.

## Projects

| Cloudflare Pages Project | Repository Directory | Custom Domain |
|---|---|---|
| `knol-web` | `frontend/web/` | `cortex.doaide.com` |
| `knol-admin` | `frontend/admin/` | `admin.cortex.doaide.com` |
| `knol-cloud` | `frontend/cloud/` | `cloud.cortex.doaide.com` |
| `knol-demo` | `frontend/demo/` | `demo.cortex.doaide.com` |
| `cortex-docs` | `frontend/docs/` | `docs.cortex.doaide.com` |

## Build Commands

- Install frontend workspace deps once: `cd frontend && npm install --no-audit --no-fund`
- `frontend/web/`: `npm run build`
- `frontend/admin/`: `npm run build`
- `frontend/cloud/`: `npm run build`
- `frontend/demo/`: `npm run build`
- `frontend/docs/`: `npm run build`

## Output Directories

- `frontend/web/out`
- `frontend/admin/out`
- `frontend/cloud/out`
- `frontend/demo/out`
- `frontend/docs/out`

## Notes

- `frontend/web/`, `frontend/admin/`, `frontend/cloud/`, `frontend/demo/`, and `frontend/docs/` are standalone Next.js apps.
- `private/docs/` is local-only documentation and is intentionally excluded from Cloudflare deployment.
- Set domain env vars per project at build time:
  - `NEXT_PUBLIC_BASE_DOMAIN=cortex.doaide.com`
  - `NEXT_PUBLIC_URL_SCHEME=https`
- Docs site server URL env vars:
  - `NEXT_PUBLIC_DOCS_URL=https://docs.cortex.doaide.com`
  - `NEXT_PUBLIC_API_BASE_URL=https://api.cortex.doaide.com`
  - `NEXT_PUBLIC_TENANT_SWAGGER_URL=https://api.cortex.doaide.com/docs`
- Run local frontend smoke checks with `./scripts/frontend-smoke.sh`.
- For server-side access control on static deployments, enforce Cloudflare Access policy on `admin.cortex.doaide.com` and `cloud.cortex.doaide.com`.
