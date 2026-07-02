#!/usr/bin/env bash
# Smoke LATER differentiation Control Center REST analytics endpoints.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
# shellcheck source=scripts/lib/control_center_smoke_lib.sh
source "$ROOT/scripts/lib/control_center_smoke_lib.sh"

PROGRAM="$ROOT/examples/showcase/mission_twin/patrol.sd"
TRACE_PROGRAM="$ROOT/examples/showcase/differentiation/decision_trail/main.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
BIND="127.0.0.1:${PORT}"
cleanup() { cc_smoke_stop_listener; }
cc_smoke_trap cleanup
CC_SMOKE_BIND="$BIND"

echo "== LATER analytics REST probes on ${BIND} =="
run_spanda control-center serve --bind "$BIND" --program "$PROGRAM" &
sleep 2

fetch() {
  local path="$1"
  curl -sf --max-time 15 "http://${BIND}${path}"
}

for path in \
  "/v1/analytics/mission-twin" \
  "/v1/analytics/certification-pack" \
  "/v1/analytics/human-teaming" \
  "/v1/analytics/governance"
do
  body="$(fetch "$path")"
  echo "$body" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("version")=="v1", d'
done

cc_smoke_stop_listener
sleep 1

PORT2=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
BIND2="127.0.0.1:${PORT2}"
CC_SMOKE_BIND="$BIND2"
run_spanda control-center serve --bind "$BIND2" --program "$TRACE_PROGRAM" &
sleep 2

body="$(fetch "/v1/analytics/time-travel?at=T%2B00%3A01&inspect=decisions")"
echo "$body" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("version")=="v1" and "time_travel" in d, d'

echo "Later analytics smoke OK"
