#!/usr/bin/env bash
# Phase E1 smoke — Control Center API and device pool lifecycle.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_CONTROL_CENTER_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"
export SPANDA_API_KEY="enterprise-ops-smoke-key"

echo "== start control-center on ${BIND} =="
run_spanda control-center serve --bind "$BIND" &
SERVER_PID=$!
sleep 2

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
}
trap cleanup EXIT

fetch() {
  local path="$1"
  local attempt=0
  while [[ $attempt -lt 30 ]]; do
    if curl -sf "http://${BIND}${path}"; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 0.2
  done
  echo "failed to fetch ${path}" >&2
  return 1
}

echo "== GET /v1/health =="
fetch /v1/health | grep -q spanda-control-center

echo "== GET /v1/dashboard =="
fetch /v1/dashboard | grep -q device_pool

echo "== GET /v1/devices =="
fetch /v1/devices | grep -q '"devices"'

echo "== GET /v1/fleet/agents =="
fetch /v1/fleet/agents | grep -q '"agents"'

echo "== GET /v1/rbac/matrix =="
fetch /v1/rbac/matrix | grep -q Administrator

echo "== POST /v1/alerts/test (authenticated) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/alerts/test" | grep -q '"ok":true'

echo "== GET /v1/alerts =="
fetch /v1/alerts | grep -q Control

echo "== GET / (Control Center UI) =="
curl -sf "http://${BIND}/" | grep -q "Spanda Control Center"

echo "Enterprise operations smoke OK"
