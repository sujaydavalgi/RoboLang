#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${CARGO_TARGET_DIR:-target}/debug/spanda"
if [[ ! -x "$BIN" ]]; then
  cargo build -p spanda --quiet
fi

echo "== package tampering: approved passes =="
"$BIN" tamper-check examples/showcase/package_tampering/approved.sd | grep -q "PASS"

echo "== package tampering: tampered has extra package finding =="
APPROVED_JSON="$("$BIN" tamper-check examples/showcase/package_tampering/approved.sd --json 2>/dev/null)"
TAMPERED_JSON="$("$BIN" tamper-check examples/showcase/package_tampering/tampered.sd --json 2>/dev/null || true)"
APPROVED_SCORE="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["trust_score"])' "$APPROVED_JSON")"
TAMPERED_SCORE="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["trust_score"])' "$TAMPERED_JSON")"
if [[ "$TAMPERED_SCORE" -ge "$APPROVED_SCORE" ]]; then
  echo "expected tampered trust score ($TAMPERED_SCORE) below approved ($APPROVED_SCORE)" >&2
  exit 1
fi

echo "== mission tampering: baseline integrity =="
"$BIN" integrity examples/showcase/mission_tampering/approved.sd --baseline examples/showcase/mission_tampering/approved.sd | grep -q "PASS"

echo "== mission tampering: modified fails integrity =="
if "$BIN" integrity examples/showcase/mission_tampering/modified.sd --baseline examples/showcase/mission_tampering/approved.sd; then
  echo "expected modified mission to fail integrity baseline compare" >&2
  exit 1
fi

echo "== runtime intrusion: trace fails runtime tamper-check =="
if "$BIN" tamper-check examples/showcase/runtime_intrusion/intrusion.trace; then
  echo "expected intrusion trace to fail runtime tamper-check" >&2
  exit 1
fi

echo "trust showcase smoke ok"
