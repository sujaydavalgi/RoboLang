#!/usr/bin/env bash
# Twin Cloud SaaS Stable tier promotion gate (Experimental → Stable).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
source "$ROOT/scripts/lib/control_center_smoke_lib.sh"
PROGRAM="$ROOT/examples/showcase/mission_twin/patrol.sd"
SOAK_FILE="${SPANDA_TWIN_CLOUD_FIELD_SOAK_START_FILE:-$ROOT/.spanda/twin-cloud-field-soak-start.txt}"
MIN_DAYS="${SPANDA_TWIN_CLOUD_FIELD_SOAK_MIN_DAYS:-30}"
HARDENING="$ROOT/docs/stable-hardening-twin-cloud-saas.md"

echo "== Twin Cloud SaaS stable promotion gate =="

if [[ ! -f "$HARDENING" ]]; then
  echo "missing $HARDENING" >&2
  exit 1
fi
grep -q "Stable Hardening" "$HARDENING"

if [[ "${SPANDA_TWIN_CLOUD_SKIP_SOAK:-1}" != "1" ]]; then
  [[ -f "$SOAK_FILE" ]] || { echo "missing $SOAK_FILE — run scripts/twin_cloud_field_soak_init.sh" >&2; exit 1; }
  START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
  if date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
    START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
  else
    START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
  fi
  ELAPSED_DAYS=$(( ($(date -u "+%s") - START_EPOCH) / 86400 ))
  (( ELAPSED_DAYS >= MIN_DAYS )) || { echo "Twin Cloud soak incomplete ($ELAPSED_DAYS / $MIN_DAYS days)" >&2; exit 1; }
else
  echo "Skipping field soak (SPANDA_TWIN_CLOUD_SKIP_SOAK=1)"
fi

if [[ "${SPANDA_TWIN_CLOUD_SKIP_SMOKE:-0}" != "1" ]]; then
  "$ROOT/scripts/twin_cloud_unified_path.sh"
else
  echo "Skipping smoke (SPANDA_TWIN_CLOUD_SKIP_SMOKE=1)"
fi

cargo test -p spanda-twin-cloud --quiet
cargo test -p spanda-api twin_cloud --quiet

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
BIND="127.0.0.1:${PORT}"
BASE="http://${BIND}"
cleanup() { cc_smoke_stop_listener; }
cc_smoke_trap cleanup
CC_SMOKE_BIND="$BIND"
export SPANDA_API_KEY="${SPANDA_API_KEY:-twin-cloud-gate-key}"
run_spanda control-center serve --bind "$BIND" --program "$PROGRAM" &
sleep 2
body="$(curl -sf --max-time 15 -H "Authorization: Bearer ${SPANDA_API_KEY}" "$BASE/v1/twins")"
echo "$body" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert "twins" in d, d'
history="$(curl -sf -X POST "$BASE/v1/twins/sync" -H 'Content-Type: application/json' -H "Authorization: Bearer ${SPANDA_API_KEY}" -d '{}')"
echo "$history" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d.get("twin_id"), d'

echo "Twin Cloud SaaS stable promotion gate passed."
