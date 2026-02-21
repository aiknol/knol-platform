# demo

Next.js TypeScript demo site deployed as `demo.aiknol.com`.

## Run

```bash
cd /Users/dev/projects/knol/memorylayer/frontend/demo
npm run dev
```

Local URL:

- `http://localhost:3008`

## Build

```bash
cd /Users/dev/projects/knol/memorylayer/frontend/demo
npm run build
```

Static output is generated in `out/`.

## Environment

- `NEXT_PUBLIC_ADMIN_API_URL` (recommended): explicit admin API base URL.
- Or set `NEXT_PUBLIC_ADMIN_API_HOST` + `NEXT_PUBLIC_ADMIN_API_PORT` (+ optional `NEXT_PUBLIC_URL_SCHEME`) to derive it.
