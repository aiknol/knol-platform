# Cloudflare Frontend Deployment

Deploy each frontend from its own directory as an independent Cloudflare Pages project.

## Projects

| Cloudflare Pages Project | Repository Directory | Custom Domain |
|---|---|---|
| `knol-web` | `frontend/web/` | `aiknol.com` |
| `knol-admin` | `frontend/admin/` | `admin.aiknol.com` |
| `knol-cloud` | `frontend/cloud/` | `cloud.aiknol.com` |
| `knol-demo` | `frontend/demo/` | `demo.aiknol.com` |
| `knol-docs` | `frontend/docs/` | `docs.aiknol.com` |

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
  - `NEXT_PUBLIC_BASE_DOMAIN=aiknol.com`
  - `NEXT_PUBLIC_URL_SCHEME=https`
- Docs site server URL env vars:
  - `NEXT_PUBLIC_DOCS_URL=https://docs.aiknol.com`
  - `NEXT_PUBLIC_API_BASE_URL=https://api.aiknol.com`
  - `NEXT_PUBLIC_TENANT_SWAGGER_URL=https://api.aiknol.com/docs`
- Run local frontend smoke checks with `./scripts/frontend-smoke.sh`.
- For server-side access control on static deployments, enforce Cloudflare Access policy on `admin.aiknol.com` and `cloud.aiknol.com`.
