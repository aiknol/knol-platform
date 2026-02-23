# Frontend Workspace

This repository uses npm workspaces for frontend apps:

- `frontend/web`
- `frontend/admin`
- `frontend/cloud`
- `frontend/demo`
- `frontend/docs`

Local-only private documentation site is outside this workspace:

- `private/docs`

## Dependency layout

Use a single shared dependency install at:

- `frontend/node_modules`

Do not run `npm install` inside individual app directories.

## Install

```bash
cd frontend
npm install
```

## Run app scripts

From `frontend/`:

```bash
npm run build:web
npm run build:admin
npm run build:app
npm run build:demo
npm run build:docs
```

Or from each app directory with shared workspace dependencies already installed.
