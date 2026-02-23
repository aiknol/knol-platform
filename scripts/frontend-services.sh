#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
FRONTEND_DIR="$ROOT_DIR/frontend"
MAIN_WEB_DIR="$FRONTEND_DIR/web"
ADMIN_WEB_DIR="$FRONTEND_DIR/admin"
APP_WEB_DIR="$FRONTEND_DIR/app"
DEMO_DIR="$FRONTEND_DIR/demo"
DOCS_DIR="$FRONTEND_DIR/docs"

# Prefer the renamed tenant site directory (`cloud`), but keep
# backward compatibility if a standalone `app` directory exists.
if [ -f "$FRONTEND_DIR/cloud/package.json" ]; then
  APP_WEB_DIR="$FRONTEND_DIR/cloud"
fi

LOCAL_BASE_DOMAIN="${NEXT_PUBLIC_BASE_DOMAIN:-localhost}"
LOCAL_URL_SCHEME="${NEXT_PUBLIC_URL_SCHEME:-http}"
LOCAL_SITE_URL="${NEXT_PUBLIC_SITE_URL:-http://localhost:3005}"
LOCAL_MAIN_HOST="${NEXT_PUBLIC_MAIN_HOST:-localhost}"
LOCAL_MAIN_PORT="${NEXT_PUBLIC_MAIN_PORT:-3005}"
LOCAL_APP_HOST="${NEXT_PUBLIC_APP_HOST:-localhost}"
LOCAL_APP_PORT="${NEXT_PUBLIC_APP_PORT:-3007}"
LOCAL_DEMO_HOST="${NEXT_PUBLIC_DEMO_HOST:-localhost}"
LOCAL_DEMO_PORT="${NEXT_PUBLIC_DEMO_PORT:-3008}"
LOCAL_DOCS_HOST="${NEXT_PUBLIC_DOCS_HOST:-localhost}"
LOCAL_DOCS_PORT="${NEXT_PUBLIC_DOCS_PORT:-3009}"
LOCAL_DOCS_URL="${NEXT_PUBLIC_DOCS_URL:-http://${LOCAL_DOCS_HOST}:${LOCAL_DOCS_PORT}}"
LOCAL_API_BASE_URL="${NEXT_PUBLIC_API_BASE_URL:-http://localhost:3000}"
LOCAL_TENANT_SWAGGER_URL="${NEXT_PUBLIC_TENANT_SWAGGER_URL:-http://localhost:3002/docs}"
LOCAL_GITHUB_REPO_URL="${NEXT_PUBLIC_GITHUB_REPO_URL:-https://github.com/aiknol/knol}"
LOCAL_ADMIN_API_HOST="${NEXT_PUBLIC_ADMIN_API_HOST:-localhost}"
LOCAL_ADMIN_API_PORT="${NEXT_PUBLIC_ADMIN_API_PORT:-3001}"
LOCAL_ADMIN_API_URL="${NEXT_PUBLIC_ADMIN_API_URL:-http://localhost:3001}"
LOCAL_APP_API_URL="${NEXT_PUBLIC_APP_API_URL:-$LOCAL_ADMIN_API_URL}"

COMMON_NEXT_ENV=(
  "NEXT_PUBLIC_BASE_DOMAIN=$LOCAL_BASE_DOMAIN"
  "NEXT_PUBLIC_URL_SCHEME=$LOCAL_URL_SCHEME"
  "NEXT_PUBLIC_SITE_URL=$LOCAL_SITE_URL"
  "NEXT_PUBLIC_MAIN_HOST=$LOCAL_MAIN_HOST"
  "NEXT_PUBLIC_MAIN_PORT=$LOCAL_MAIN_PORT"
  "NEXT_PUBLIC_APP_HOST=$LOCAL_APP_HOST"
  "NEXT_PUBLIC_APP_PORT=$LOCAL_APP_PORT"
  "NEXT_PUBLIC_DEMO_HOST=$LOCAL_DEMO_HOST"
  "NEXT_PUBLIC_DEMO_PORT=$LOCAL_DEMO_PORT"
  "NEXT_PUBLIC_DOCS_HOST=$LOCAL_DOCS_HOST"
  "NEXT_PUBLIC_DOCS_PORT=$LOCAL_DOCS_PORT"
  "NEXT_PUBLIC_DOCS_URL=$LOCAL_DOCS_URL"
  "NEXT_PUBLIC_API_BASE_URL=$LOCAL_API_BASE_URL"
  "NEXT_PUBLIC_TENANT_SWAGGER_URL=$LOCAL_TENANT_SWAGGER_URL"
  "NEXT_PUBLIC_GITHUB_REPO_URL=$LOCAL_GITHUB_REPO_URL"
  "NEXT_PUBLIC_ADMIN_API_HOST=$LOCAL_ADMIN_API_HOST"
  "NEXT_PUBLIC_ADMIN_API_PORT=$LOCAL_ADMIN_API_PORT"
  "NEXT_PUBLIC_ADMIN_API_URL=$LOCAL_ADMIN_API_URL"
  "NEXT_PUBLIC_APP_API_URL=$LOCAL_APP_API_URL"
)

usage() {
  cat <<'EOF'
Usage: ./scripts/frontend-services.sh <start|stop|restart|status>

Commands:
  start    Start all frontend services (main, admin, app, demo, docs).
  stop     Stop all frontend services.
  restart  Restart all frontend services.
  status   Show current status of all frontend services.
EOF
}

require_bin() {
  local bin="$1"
  command -v "$bin" >/dev/null 2>&1 || {
    echo "$bin is required"
    exit 1
  }
}

wait_for_http() {
  local url="$1"
  local attempts=60
  local i
  for ((i=1; i<=attempts; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.4
  done
  return 1
}

is_listening() {
  local port="$1"
  lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
}

has_next_binary() {
  [ -x "$FRONTEND_DIR/node_modules/.bin/next" ]
}

ensure_frontend_dependencies() {
  if has_next_binary; then
    return 0
  fi
  if [ ! -f "$FRONTEND_DIR/package.json" ]; then
    echo "Missing frontend workspace package: $FRONTEND_DIR/package.json"
    exit 1
  fi
  echo "Installing frontend workspace dependencies..."
  (cd "$FRONTEND_DIR" && npm install --no-audit --no-fund >/tmp/frontend-install.log 2>&1)
  if ! has_next_binary; then
    echo "Frontend dependencies did not install correctly. See /tmp/frontend-install.log"
    exit 1
  fi
}

start_service() {
  local name="$1"
  local service_dir="$2"
  local port="$3"
  local logfile="/tmp/${name}-${port}.log"
  local pidfile="/tmp/${name}-${port}.pid"

  if [ ! -d "$service_dir" ]; then
    echo "Missing website directory: $service_dir"
    exit 1
  fi

  if [ ! -f "$service_dir/package.json" ]; then
    echo "Missing package.json in: $service_dir"
    exit 1
  fi

  if is_listening "$port"; then
    echo "Port $port already in use, skipping start for $name"
    return
  fi

  (
    cd "$service_dir"
    nohup env "${COMMON_NEXT_ENV[@]}" HOSTNAME=0.0.0.0 PORT="$port" npm run dev >"$logfile" 2>&1 &
    echo $! >"$pidfile"
  )
}

stop_frontends() {
  for pid_file in /tmp/web-main-3005.pid /tmp/admin-3006.pid /tmp/app-3007.pid /tmp/demo-3008.pid /tmp/docs-3009.pid; do
    if [ -f "$pid_file" ]; then
      kill "$(cat "$pid_file")" 2>/dev/null || true
      rm -f "$pid_file"
    fi
  done

  for port in 3005 3006 3007 3008 3009; do
    lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null | xargs -r kill 2>/dev/null || true
  done
}

show_status() {
  local ports=(3005 3006 3007 3008 3009)
  local labels=("Main" "Admin" "App" "Demo" "Docs")
  local i
  echo "Frontend status:"
  for i in "${!ports[@]}"; do
    local port="${ports[$i]}"
    local label="${labels[$i]}"
    local code
    code="$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:${port}/" || true)"
    if is_listening "$port"; then
      echo "  ${label}: http://localhost:${port} (up, HTTP ${code:-n/a})"
    else
      echo "  ${label}: http://localhost:${port} (down)"
    fi
  done
}

clean_next_caches() {
  local dirs=("$MAIN_WEB_DIR" "$ADMIN_WEB_DIR" "$APP_WEB_DIR" "$DEMO_DIR" "$DOCS_DIR")
  local dir
  for dir in "${dirs[@]}"; do
    rm -rf "$dir/.next" "$dir/.next-dev" "$dir/.next-build"
  done
}

start_all() {
  require_bin npm
  ensure_frontend_dependencies

  # Prevent stale Next dev cache issues after refactors/renames.
  clean_next_caches

  echo "Starting frontend services..."
  start_service web-main "$MAIN_WEB_DIR" 3005
  start_service admin "$ADMIN_WEB_DIR" 3006
  start_service app "$APP_WEB_DIR" 3007
  start_service demo "$DEMO_DIR" 3008
  start_service docs "$DOCS_DIR" 3009

  echo "Waiting for services to become reachable..."
  wait_for_http "http://127.0.0.1:3005/" || { echo "Main site failed (3005). See /tmp/web-main-3005.log"; exit 1; }
  wait_for_http "http://127.0.0.1:3006/" || { echo "Admin site failed (3006). See /tmp/admin-3006.log"; exit 1; }
  wait_for_http "http://127.0.0.1:3007/" || { echo "App site failed (3007). See /tmp/app-3007.log"; exit 1; }
  wait_for_http "http://127.0.0.1:3008/" || { echo "Demo site failed (3008). See /tmp/demo-3008.log"; exit 1; }
  wait_for_http "http://127.0.0.1:3009/" || { echo "Docs site failed (3009). See /tmp/docs-3009.log"; exit 1; }

  echo "All frontend services are up:"
  echo "  Main : http://localhost:3005"
  echo "  Admin: http://localhost:3006"
  echo "  App  : http://localhost:3007"
  echo "  Demo : http://localhost:3008"
  echo "  Docs : http://localhost:3009"
}

cmd="${1:-}"
case "$cmd" in
  start)
    start_all
    ;;
  stop)
    echo "Stopping frontend services..."
    stop_frontends
    echo "Frontend services stopped."
    ;;
  restart)
    echo "Restarting frontend services..."
    stop_frontends
    start_all
    ;;
  status)
    show_status
    ;;
  *)
    usage
    exit 1
    ;;
esac
