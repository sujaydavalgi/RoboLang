#!/usr/bin/env bash
# Mission Risk Analysis Stable tier promotion gate.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
# shellcheck source=scripts/lib/control_center_smoke_lib.sh
source "$ROOT/scripts/lib/control_center_smoke_lib.sh"

SOAK_FILE="${SPANDA_RISK_FIELD_SOAK_START_FILE:-$ROOT/.spanda/risk-field-soak-start.txt}"
MIN_DAYS="${SPANDA_RISK_FIELD_SOAK_MIN_DAYS:-30}"
PROGRAM="$ROOT/examples/showcase/risk/deployment_risk.sd"

echo "== Mission risk stable promotion gate =="

if [[ "${SPANDA_RISK_SKIP_SOAK:-1}" != "1" ]]; then
  if [[ ! -f "$SOAK_FILE" ]]; then
    echo "missing soak file: $SOAK_FILE — run ./scripts/risk_field_soak_init.sh" >&2
    exit 1
  fi
  START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
  if date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
    START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
  else
    START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
  fi
  ELAPSED_DAYS=$(( ($(date -u "+%s") - START_EPOCH) / 86400 ))
  if (( ELAPSED_DAYS < MIN_DAYS )); then
    echo "Risk field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
    exit 1
  fi
else
  echo "Skipping field soak (SPANDA_RISK_SKIP_SOAK=1)"
fi

if [[ "${SPANDA_RISK_SKIP_SMOKE:-0}" != "1" ]]; then
  "$ROOT/scripts/risk_smoke.sh"
else
  echo "Skipping smoke (SPANDA_RISK_SKIP_SMOKE=1)"
fi

cargo test -p spanda-risk --quiet

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

body="$(curl -sf --max-time 15 "http://${BIND}/v1/analytics/mission-risk")"
echo "$body" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("version")=="v1", d'

echo "Mission risk stable promotion gate passed."
