#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${CARGO_TARGET_DIR:-target}/debug/spanda"
if [[ ! -x "$BIN" ]]; then
  cargo build -p spanda --quiet
fi

echo "== diagnose tamper: runtime intrusion =="
INTRUSION_OUT="$("$BIN" diagnose tamper examples/showcase/runtime_intrusion/intrusion.trace 2>&1 || true)"
echo "$INTRUSION_OUT" | grep -q "Intruder"
echo "$INTRUSION_OUT" | grep -q "Result: FAIL"

echo "== diagnose tamper: gps spoofing =="
SPOOF_OUT="$("$BIN" diagnose tamper examples/showcase/gps_spoofing/spoof.trace 2>&1 || true)"
echo "$SPOOF_OUT" | grep -qi "spoof"
echo "$SPOOF_OUT" | grep -q "Result: FAIL"

echo "tamper diagnose smoke ok"
