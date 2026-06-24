#!/usr/bin/env bash
# Golden path for persistent telemetry store (sim + query CLI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

STORE_DIR="$(mktemp -d)"
trap 'rm -rf "${STORE_DIR}"' EXIT
export SPANDA_TELEMETRY_STORE_PATH="${STORE_DIR}/telemetry.jsonl"
export SPANDA_TELEMETRY_HEARTBEAT_PATH="${STORE_DIR}/heartbeats.json"

SPANDA=(cargo run --quiet -p spanda --)

echo "== sim with --persist-telemetry =="
"${SPANDA[@]}" sim examples/end_to_end/validated_telemetry.sd --persist-telemetry >/dev/null

echo "== telemetry stats =="
STATS="$("${SPANDA[@]}" telemetry stats)"
echo "${STATS}"
echo "${STATS}" | grep -q "Sensor events:"
echo "${STATS}" | grep -q "Device events:"

echo "== telemetry list sensor events =="
LIST="$("${SPANDA[@]}" telemetry list --kind sensor --limit 3)"
echo "${LIST}"
echo "${LIST}" | grep -q '\[sensor\]'

echo "== telemetry session + runtime metrics =="
SESSIONS="$("${SPANDA[@]}" telemetry list --kind session --limit 2)"
echo "${SESSIONS}"
echo "${SESSIONS}" | grep -q '\[session\]'
SESSION_ID="$(echo "${SESSIONS}" | sed -n 's/.*\[session\].* \([^ ]*\) phase=start.*/\1/p' | head -1)"
FILTERED="$("${SPANDA[@]}" telemetry list --session "${SESSION_ID}" --kind sensor --limit 1)"
echo "${FILTERED}"
echo "${FILTERED}" | grep -q "session=${SESSION_ID}"
METRICS="$("${SPANDA[@]}" telemetry list --kind runtime_metrics --limit 1)"
echo "${METRICS}"
echo "${METRICS}" | grep -q '\[runtime_metrics\]'

echo "== telemetry prometheus =="
PROM="$("${SPANDA[@]}" telemetry prometheus)"
echo "${PROM}" | head -5
echo "${PROM}" | grep -q 'spanda_telemetry_events_total'

echo "== telemetry otlp =="
OTLP="$("${SPANDA[@]}" telemetry otlp)"
echo "${OTLP}" | head -8
echo "${OTLP}" | grep -q 'resourceMetrics'

echo "== telemetry latest device publish =="
LATEST="$("${SPANDA[@]}" telemetry latest --device TelemetryRover --metric /telemetry)"
echo "${LATEST}"
echo "${LATEST}" | grep -q 'TelemetryRover'

echo "== sqlite backend migrates JSONL history =="
SQLITE_DIR="$(mktemp -d)"
JSONL_PATH="${SQLITE_DIR}/telemetry-store.jsonl"
DB_PATH="${SQLITE_DIR}/telemetry-store.db"
cp "${SPANDA_TELEMETRY_STORE_PATH}" "${JSONL_PATH}"
if [[ -f "${SPANDA_TELEMETRY_HEARTBEAT_PATH}" ]]; then
  cp "${SPANDA_TELEMETRY_HEARTBEAT_PATH}" "${SQLITE_DIR}/telemetry-heartbeats.json"
fi
unset SPANDA_TELEMETRY_HEARTBEAT_PATH
export SPANDA_TELEMETRY_BACKEND=sqlite
export SPANDA_TELEMETRY_STORE_PATH="${DB_PATH}"
SQLITE_STATS="$("${SPANDA[@]}" telemetry stats)"
echo "${SQLITE_STATS}"
echo "${SQLITE_STATS}" | grep -q "Sensor events:"
echo "${SQLITE_STATS}" | grep -q "Device events:"
test ! -f "${JSONL_PATH}"
test -f "${SQLITE_DIR}/telemetry-store.jsonl.bak"
unset SPANDA_TELEMETRY_BACKEND

echo "Telemetry store golden path complete."
