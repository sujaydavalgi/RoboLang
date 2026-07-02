#!/usr/bin/env bash
# Twin Cloud SaaS golden path — Control Center backend + CLI push/pull/list.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
# shellcheck source=scripts/lib/control_center_smoke_lib.sh
source "$ROOT/scripts/lib/control_center_smoke_lib.sh"

PROGRAM="$ROOT/examples/showcase/mission_twin/patrol.sd"
PULL_FILE="${TMPDIR:-/tmp}/spanda-twin-cloud-pull.json"
STATE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/spanda-twin-cloud-state.XXXXXX")"
export SPANDA_CONTROL_CENTER_STATE_DIR="$STATE_DIR"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
BIND="127.0.0.1:${PORT}"
BASE="http://${BIND}"
cleanup() {
  cc_smoke_stop_listener
  rm -rf "$STATE_DIR"
}
cc_smoke_trap cleanup
CC_SMOKE_BIND="$BIND"

echo "== Twin Cloud SaaS backend (Control Center) =="
run_spanda control-center serve --bind "$BIND" --program "$PROGRAM" &
CC_SMOKE_WRAPPER_PID=$!

echo "== wait for /v1/health =="
cc_smoke_wait_for_health

export SPANDA_TWIN_CLOUD_URL="$BASE"

echo "== CLI push mission twin snapshot =="
run_spanda twin cloud push "$PROGRAM" --json >/tmp/spanda-twin-cloud-push.json
grep -q '"twin_id"' /tmp/spanda-twin-cloud-push.json

echo "== CLI list twins =="
run_spanda twin cloud list --json | grep -q '"patrol"'

echo "== CLI pull latest snapshot =="
rm -f "$PULL_FILE"
run_spanda twin cloud pull patrol --out "$PULL_FILE"
test -s "$PULL_FILE"
grep -q '"mission_twin"' "$PULL_FILE"

echo "== REST sync + get =="
curl -sf -X POST "$BASE/v1/twins/sync" -H 'Content-Type: application/json' -d '{}' >/dev/null
curl -sf "$BASE/v1/twins/patrol" | grep -q '"mission_twin"'

echo "== Restart Control Center and verify persisted twin =="
cc_smoke_stop_listener
run_spanda control-center serve --bind "$BIND" --program "$PROGRAM" &
CC_SMOKE_WRAPPER_PID=$!
cc_smoke_wait_for_health
run_spanda twin cloud pull patrol --out "$PULL_FILE"
grep -q '"mission_twin"' "$PULL_FILE"

echo "Twin Cloud SaaS golden path OK"
