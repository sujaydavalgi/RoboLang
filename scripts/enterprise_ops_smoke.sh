#!/usr/bin/env bash
# Phase E1+E2+E3 smoke — Control Center API, provisioning, drift, OTA, SDK paths.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
CONFIG="${ROOT}/crates/spanda-config/tests/fixtures/warehouse/spanda.toml"

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

echo "== start control-center on ${BIND} (warehouse config) =="
run_spanda control-center serve --bind "$BIND" --config "$CONFIG" &
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

echo "== E2 GET /v1/discovery?transport=mdns =="
fetch "/v1/discovery?transport=mdns" | grep -q mdns-stub-robot

echo "== E2 GET /v1/health/summary =="
fetch /v1/health/summary | grep -q overall_status

echo "== E2 GET /v1/assurance/summary =="
fetch /v1/assurance/summary | grep -q '"loaded":true'

echo "== E2 POST /v1/provision (expect readiness alert) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"device_id":"lidar-front","robot_id":"rover-001"}' \
  "http://${BIND}/v1/provision" | grep -q '"ok":false'

echo "== E2 GET /v1/alerts (provisioning failure) =="
fetch /v1/alerts | grep -q readiness_failed

echo "== E2 POST /v1/config/snapshots =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"label":"smoke-baseline"}' \
  "http://${BIND}/v1/config/snapshots" | grep -q '"ok":true'

echo "== E2 GET /v1/config/snapshots =="
SNAPSHOT_JSON=$(fetch /v1/config/snapshots)
echo "$SNAPSHOT_JSON" | grep -q smoke-baseline
BASELINE_ID=$(echo "$SNAPSHOT_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["snapshots"][0]["id"])')

echo "== E3 GET /v1/openapi.json =="
fetch /v1/openapi.json | grep -q Spanda

echo "== E3 GET /v1/drift?baseline_id =="
fetch "/v1/drift?baseline_id=${BASELINE_ID}" | grep -q dimensions_checked

echo "== E3 POST /v1/ota/plan (canary dry-run) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"strategy":"canary","version":"1.2.3","canary_percent":20,"dry_run":true,"assignments":[{"robot_name":"rover-001","hardware":"jetson"}]}' \
  "http://${BIND}/v1/ota/plan" | grep -q '"strategy":"canary"'

echo "== E3 GET /v1/trust/package?name=spanda-mqtt =="
fetch "/v1/trust/package?name=spanda-mqtt" | grep -q trust

echo "== E3 GET /v1/sre/summary =="
fetch /v1/sre/summary | grep -q availability_percent

echo "== E3 GET /v1/observability/traces (correlation IDs) =="
curl -sf -H "X-Correlation-ID: smoke-trace-1" "http://${BIND}/v1/health" >/dev/null
fetch /v1/observability/traces | grep -q smoke-trace-1

echo "== E3 POST /v1/rpc (gRPC gateway) =="
curl -sf -X POST \
  -H "Content-Type: application/json" \
  -d '{"method":"spanda.v1.SpandaService/GetHealth"}' \
  "http://${BIND}/v1/rpc" | grep -q spanda-control-center

echo "== E3 Python SDK health =="
PYTHONPATH="${ROOT}/packages/sdk-python/src:${PYTHONPATH:-}" \
  SPANDA_CONTROL_CENTER_URL="http://${BIND}" SPANDA_API_KEY="${SPANDA_API_KEY}" \
  python3 -c "from spanda_sdk import ControlCenterClient; c=ControlCenterClient(); assert c.health()['service']=='spanda-control-center'"

echo "Enterprise operations smoke OK"
