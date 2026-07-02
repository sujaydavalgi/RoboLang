#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
source "$ROOT/scripts/lib/control_center_smoke_lib.sh"
PROGRAM="$ROOT/examples/showcase/forecast/degradation.sd"
SOAK_FILE="${SPANDA_FORECAST_FIELD_SOAK_START_FILE:-$ROOT/.spanda/forecast-field-soak-start.txt}"
MIN_DAYS="${SPANDA_FORECAST_FIELD_SOAK_MIN_DAYS:-30}"

echo "== Readiness forecast stable promotion gate =="
if [[ "${SPANDA_FORECAST_SKIP_SOAK:-1}" != "1" ]]; then
  [[ -f "$SOAK_FILE" ]] || { echo "missing $SOAK_FILE" >&2; exit 1; }
  START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
  if date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
    START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
  else
    START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
  fi
  ELAPSED_DAYS=$(( ($(date -u "+%s") - START_EPOCH) / 86400 ))
  (( ELAPSED_DAYS >= MIN_DAYS )) || { echo "Forecast soak incomplete" >&2; exit 1; }
else
  echo "Skipping field soak (SPANDA_FORECAST_SKIP_SOAK=1)"
fi

if [[ "${SPANDA_FORECAST_SKIP_SMOKE:-0}" != "1" ]]; then
  "$ROOT/scripts/readiness_forecast_smoke.sh"
else
  echo "Skipping smoke (SPANDA_FORECAST_SKIP_SMOKE=1)"
fi

cargo test -p spanda-readiness --test forecast_tests --quiet

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
run_spanda control-center serve --bind "$BIND" --program "$PROGRAM" &
sleep 2
body="$(curl -sf --max-time 15 "http://${BIND}/v1/analytics/readiness-forecast?all=1")"
echo "$body" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("version")=="v1", d'
echo "Readiness forecast stable promotion gate passed."
