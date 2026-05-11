#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
FRONTEND_DIR="$ROOT_DIR/frontend"

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required"
  exit 1
fi

if [ -z "${CLOUDFLARE_API_TOKEN:-}" ] && [ "${WRANGLER_AUTH_MODE:-}" != "oauth" ]; then
  for candidate in \
    "$ROOT_DIR/../keys/token.txt" \
    "$ROOT_DIR/keys/token.txt" \
    "$ROOT_DIR/../keys/Cloudfare_token.txt" \
    "$ROOT_DIR/../keys/Cloudflare_token.txt" \
  ; do
    if [ -f "$candidate" ]; then
      parsed_token="$(awk -F':' 'tolower($1) ~ /cloud/ {gsub(/^ +/, "", $2); print $2; exit}' "$candidate")"
      if [ -z "$parsed_token" ]; then
        parsed_token="$(head -n 1 "$candidate" | tr -d '\r\n' || true)"
      fi
      if [ -n "$parsed_token" ]; then
        export CLOUDFLARE_API_TOKEN="$parsed_token"
        break
      fi
    fi
  done
fi

if [ -z "${CLOUDFLARE_API_TOKEN:-}" ]; then
  echo "CLOUDFLARE_API_TOKEN not found; continuing with Wrangler's stored OAuth login (run: wrangler login)."
fi

WRANGLER_CMD="${WRANGLER_CMD:-npx --yes wrangler@4.44.0}"
WEB_PROJECT="${WEB_PROJECT:-knol-web}"
ADMIN_PROJECT="${ADMIN_PROJECT:-knol-admin}"
CLOUD_PROJECT="${CLOUD_PROJECT:-knol-cloud}"
DEMO_PROJECT="${DEMO_PROJECT:-knol-demo}"
DOCS_PROJECT="${DOCS_PROJECT:-knol-docs}"

ensure_public_output() {
  local output_dir="$1"
  case "$output_dir" in
    frontend/*) ;;
    *)
      echo "Refusing deploy: output must be under frontend/. Received: $output_dir"
      exit 1
      ;;
  esac

  case "$output_dir" in
    private/*|*/private/*)
      echo "Refusing deploy: private docs are local-only and must not be deployed."
      exit 1
      ;;
  esac
}

export NEXT_PUBLIC_BASE_DOMAIN="${NEXT_PUBLIC_BASE_DOMAIN:-aiknol.com}"
export NEXT_PUBLIC_URL_SCHEME="${NEXT_PUBLIC_URL_SCHEME:-https}"
export NEXT_PUBLIC_SITE_URL="${NEXT_PUBLIC_SITE_URL:-https://aiknol.com}"
export NEXT_PUBLIC_MAIN_HOST="${NEXT_PUBLIC_MAIN_HOST:-aiknol.com}"
export NEXT_PUBLIC_APP_HOST="${NEXT_PUBLIC_APP_HOST:-cloud.aiknol.com}"
export NEXT_PUBLIC_DEMO_HOST="${NEXT_PUBLIC_DEMO_HOST:-demo.aiknol.com}"
export NEXT_PUBLIC_ADMIN_API_HOST="${NEXT_PUBLIC_ADMIN_API_HOST:-api.aiknol.com}"
export NEXT_PUBLIC_ADMIN_API_URL="${NEXT_PUBLIC_ADMIN_API_URL:-https://api.aiknol.com}"
export NEXT_PUBLIC_APP_API_URL="${NEXT_PUBLIC_APP_API_URL:-https://api.aiknol.com}"
export NEXT_PUBLIC_APP_URL="${NEXT_PUBLIC_APP_URL:-https://cloud.aiknol.com}"
export NEXT_PUBLIC_DEMO_URL="${NEXT_PUBLIC_DEMO_URL:-https://demo.aiknol.com}"
export NEXT_PUBLIC_DOCS_URL="${NEXT_PUBLIC_DOCS_URL:-https://docs.aiknol.com}"
export NEXT_PUBLIC_API_BASE_URL="${NEXT_PUBLIC_API_BASE_URL:-https://api.aiknol.com}"
export NEXT_PUBLIC_TENANT_SWAGGER_URL="${NEXT_PUBLIC_TENANT_SWAGGER_URL:-https://api.aiknol.com/docs}"
export NEXT_PUBLIC_GITHUB_REPO_URL="${NEXT_PUBLIC_GITHUB_REPO_URL:-https://github.com/aiknol/knol}"
export NEXT_PUBLIC_APP_SIGNUP_URL="${NEXT_PUBLIC_APP_SIGNUP_URL:-https://cloud.aiknol.com/signup/}"
export NEXT_PUBLIC_APP_LOGIN_URL="${NEXT_PUBLIC_APP_LOGIN_URL:-https://cloud.aiknol.com/login/}"

echo "Building frontend sites..."
(cd "$FRONTEND_DIR/web" && npm run build)
(cd "$FRONTEND_DIR/admin" && npm run build)
(cd "$FRONTEND_DIR/cloud" && npm run build)
(cd "$FRONTEND_DIR/demo" && npm run build)
(cd "$FRONTEND_DIR/docs" && npm run build)

echo "Deploying web -> Cloudflare Pages project: $WEB_PROJECT"
ensure_public_output "frontend/web/out"
(cd "$ROOT_DIR" && $WRANGLER_CMD pages deploy frontend/web/out --project-name="$WEB_PROJECT")

echo "Deploying admin -> Cloudflare Pages project: $ADMIN_PROJECT"
ensure_public_output "frontend/admin/out"
(cd "$ROOT_DIR" && $WRANGLER_CMD pages deploy frontend/admin/out --project-name="$ADMIN_PROJECT")

echo "Deploying cloud -> Cloudflare Pages project: $CLOUD_PROJECT"
ensure_public_output "frontend/cloud/out"
(cd "$ROOT_DIR" && $WRANGLER_CMD pages deploy frontend/cloud/out --project-name="$CLOUD_PROJECT")

echo "Deploying demo -> Cloudflare Pages project: $DEMO_PROJECT"
ensure_public_output "frontend/demo/out"
(cd "$ROOT_DIR" && $WRANGLER_CMD pages deploy frontend/demo/out --project-name="$DEMO_PROJECT")

echo "Deploying docs -> Cloudflare Pages project: $DOCS_PROJECT"
ensure_public_output "frontend/docs/.next-build"
(cd "$ROOT_DIR" && $WRANGLER_CMD pages deploy frontend/docs/.next-build --project-name="$DOCS_PROJECT")

echo "Cloudflare frontend deployment complete."
