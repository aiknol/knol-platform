#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

strip_trailing_slash() {
  local value="$1"
  while [[ "$value" == */ ]]; do
    value="${value%/}"
  done
  echo "$value"
}

ensure_trailing_slash() {
  local value
  value="$(strip_trailing_slash "$1")"
  echo "${value}/"
}

ensure_signup_url() {
  local normalized
  normalized="$(strip_trailing_slash "$1")"
  if [[ "$normalized" == */signup ]]; then
    echo "${normalized}/"
    return
  fi
  echo "${normalized}/signup/"
}

compute_app_origin() {
  local explicit_app_url="${NEXT_PUBLIC_APP_URL:-}"
  if [[ -n "$explicit_app_url" ]]; then
    strip_trailing_slash "$explicit_app_url"
    return
  fi

  local host="${NEXT_PUBLIC_APP_HOST:-localhost}"
  local port="${NEXT_PUBLIC_APP_PORT:-3007}"
  local scheme="${NEXT_PUBLIC_URL_SCHEME:-http}"
  if [[ "$host" == "localhost" || "$host" == "127.0.0.1" ]]; then
    scheme="http"
  fi
  echo "${scheme}://${host}:${port}"
}

assert_redirect() {
  local path="$1"
  local expected="$2"
  local actual
  actual="$(curl -sI "http://127.0.0.1:3005${path}" | tr -d '\r' | awk -F': ' 'tolower($1)=="location"{print $2}' | head -n1)"
  if [[ -z "$actual" ]]; then
    echo "Smoke check failed: missing redirect location for ${path}"
    exit 1
  fi
  if [[ "$(strip_trailing_slash "$actual")" != "$(strip_trailing_slash "$expected")" ]]; then
    echo "Smoke check failed: ${path} redirects to ${actual}, expected ${expected}"
    exit 1
  fi
}

assert_contains() {
  local url="$1"
  local pattern="$2"
  local response
  response="$(curl -fsS "$url")"
  if ! echo "$response" | grep -q "$pattern"; then
    echo "Smoke check failed: ${url} does not contain expected text: ${pattern}"
    exit 1
  fi
}

assert_http_200() {
  local url="$1"
  local code
  code="$(curl -s -o /dev/null -w "%{http_code}" "$url")"
  if [[ "$code" != "200" ]]; then
    echo "Smoke check failed: ${url} returned HTTP ${code}"
    exit 1
  fi
}

expected_signup="${NEXT_PUBLIC_APP_SIGNUP_URL:-}"
if [[ -z "$expected_signup" ]]; then
  expected_signup="$(ensure_signup_url "$(compute_app_origin)")"
else
  expected_signup="$(ensure_signup_url "$expected_signup")"
fi

expected_login="${NEXT_PUBLIC_APP_LOGIN_URL:-}"
if [[ -z "$expected_login" ]]; then
  expected_login="$(ensure_trailing_slash "$(strip_trailing_slash "$(compute_app_origin)")/login")"
else
  expected_login="$(ensure_trailing_slash "$expected_login")"
fi

"$SCRIPT_DIR/frontend-services.sh" start >/tmp/frontend-smoke-start.log 2>&1
trap '"$SCRIPT_DIR/frontend-services.sh" stop >/tmp/frontend-smoke-stop.log 2>&1 || true' EXIT

assert_redirect "/signup/" "$expected_signup"
assert_redirect "/login/" "$expected_login"

assert_http_200 "http://127.0.0.1:3006/login/"
assert_http_200 "http://127.0.0.1:3006/dashboard/"
assert_http_200 "http://127.0.0.1:3007/signup/"
assert_http_200 "http://127.0.0.1:3007/dashboard/"
assert_http_200 "http://127.0.0.1:3008/"
assert_http_200 "http://127.0.0.1:3009/"

assert_contains "http://127.0.0.1:3006/login/" "Sign in to admin"
assert_contains "http://127.0.0.1:3007/signup/" "Create Your Free Workspace"
assert_contains "http://127.0.0.1:3008/" "<title>"
assert_contains "http://127.0.0.1:3009/" "Knol Docs"

echo "Frontend smoke checks passed."
