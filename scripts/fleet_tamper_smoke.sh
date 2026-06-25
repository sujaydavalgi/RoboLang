#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${CARGO_TARGET_DIR:-target}/debug/spanda"
if [[ ! -x "$BIN" ]]; then
  cargo build -p spanda --quiet
fi

MANIFEST="examples/showcase/fleet_tamper/manifest.json"

echo "== fleet tamper-check correlation =="
if "$BIN" tamper-check --fleet "$MANIFEST"; then
  echo "expected fleet tamper correlation to fail" >&2
  exit 1
fi

OUTPUT="$("$BIN" tamper-check --fleet "$MANIFEST" 2>&1 || true)"
echo "$OUTPUT" | grep -q "shared_agent_intrusion"
echo "$OUTPUT" | grep -q "RoverAlpha"
echo "$OUTPUT" | grep -q "RoverBeta"

echo "== fleet diagnose tamper correlation =="
DIAG="$("$BIN" diagnose tamper --fleet "$MANIFEST" 2>&1 || true)"
echo "$DIAG" | grep -q "Fleet tamper correlation"
echo "$DIAG" | grep -q "Intruder"

echo "fleet tamper smoke ok"
