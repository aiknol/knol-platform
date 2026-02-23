# Knol Public Docs Site

Public docs website for `docs.aiknol.com`.

Scope of this site:

- Knol OSS documentation
- Full tenant service documentation for cloud users

Platform-internal documentation is intentionally excluded from this public site and is available in `private/docs` for local usage.

## Development

```bash
cd frontend/docs
npm install
npm run dev
```

Default URL: `http://localhost:3009`

## Build

```bash
npm run build
```

Static output is generated in `frontend/docs/out`.

Build artifacts are isolated to avoid `next dev` and `next build` cache conflicts:

- Dev cache: `.next-dev/`
- Build cache: `.next-build/`

## URL Configuration

All server-related URLs in the docs site are centralized in:

- `frontend/docs/src/config/site.ts`

Override at build time with env vars:

- `NEXT_PUBLIC_DOCS_URL`
- `NEXT_PUBLIC_API_BASE_URL`
- `NEXT_PUBLIC_TENANT_SWAGGER_URL`
- `NEXT_PUBLIC_GITHUB_REPO_URL`

## Troubleshooting

If you see an error like `Cannot find module './237.js'`, clear docs build caches and restart:

```bash
rm -rf .next .next-dev .next-build
npm run dev
```
