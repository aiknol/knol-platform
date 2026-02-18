# Admin Integration

## Environment

Set API URL for the admin UI:

```bash
NEXT_PUBLIC_ADMIN_API_URL=http://localhost:3001
```

## Local Access

- Admin site: `http://localhost:3006/` (redirects to `/admin/`)
- Admin login: `http://localhost:3006/admin/login`
- Admin API health: `http://localhost:3001/health`

## Auth Model

- Login endpoint: `POST /admin/auth/login`
- JWT token stored in browser local storage
- Protected admin endpoints require `Authorization: Bearer <token>`

## Reference

- See `ADMIN_PANEL.md` for routes and page structure.
