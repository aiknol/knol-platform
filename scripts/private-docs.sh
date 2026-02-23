#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
PRIVATE_DOCS_DIR="$ROOT_DIR/private/docs"
PID_FILE="/tmp/private-docs-3010.pid"
LOG_FILE="/tmp/private-docs-3010.log"
PORT="${PORT:-3010}"

usage() {
  cat <<'EOF'
Usage: ./scripts/private-docs.sh <start|stop|restart|status>

Commands:
  start    Start private docs website on localhost.
  stop     Stop private docs website.
  restart  Restart private docs website.
  status   Show private docs website status.
EOF
}

is_listening() {
  lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1
}

ensure_dependencies() {
  if [ ! -x "$PRIVATE_DOCS_DIR/node_modules/.bin/next" ]; then
    (cd "$PRIVATE_DOCS_DIR" && npm install --no-audit --no-fund)
  fi
}

start_private_docs() {
  if is_listening; then
    echo "Private docs already running at http://localhost:$PORT"
    return
  fi

  ensure_dependencies

  (
    cd "$PRIVATE_DOCS_DIR"
    nohup env \
      HOSTNAME=0.0.0.0 \
      PORT="$PORT" \
      NEXT_PUBLIC_PRIVATE_DOCS_URL="http://localhost:$PORT" \
      NEXT_PUBLIC_API_BASE_URL="${NEXT_PUBLIC_API_BASE_URL:-http://localhost:3000}" \
      NEXT_PUBLIC_TENANT_SWAGGER_URL="${NEXT_PUBLIC_TENANT_SWAGGER_URL:-http://localhost:3002/docs}" \
      NEXT_PUBLIC_GITHUB_REPO_URL="${NEXT_PUBLIC_GITHUB_REPO_URL:-https://github.com/aiknol/knol}" \
      npm run dev >"$LOG_FILE" 2>&1 &
    echo $! >"$PID_FILE"
  )

  for _ in {1..80}; do
    if curl -fsS "http://127.0.0.1:$PORT/" >/dev/null 2>&1; then
      echo "Private docs running at http://localhost:$PORT"
      return
    fi
    sleep 0.4
  done

  echo "Private docs failed to start. See $LOG_FILE"
  exit 1
}

stop_private_docs() {
  if [ -f "$PID_FILE" ]; then
    kill "$(cat "$PID_FILE")" 2>/dev/null || true
    rm -f "$PID_FILE"
  fi
  lsof -tiTCP:"$PORT" -sTCP:LISTEN 2>/dev/null | xargs -r kill 2>/dev/null || true
  echo "Private docs stopped."
}

status_private_docs() {
  if is_listening; then
    code="$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$PORT/" || true)"
    echo "Private docs: http://localhost:$PORT (up, HTTP ${code:-n/a})"
  else
    echo "Private docs: http://localhost:$PORT (down)"
  fi
}

cmd="${1:-}"
case "$cmd" in
  start)
    start_private_docs
    ;;
  stop)
    stop_private_docs
    ;;
  restart)
    stop_private_docs
    start_private_docs
    ;;
  status)
    status_private_docs
    ;;
  *)
    usage
    exit 1
    ;;
esac
