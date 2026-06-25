#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${CARGO_TARGET_DIR:-$ROOT/target}/debug/spanda"
ROVER="$ROOT/examples/showcase/tamper_policy/rover.sd"

cd "$ROOT"
cargo build -p spanda -q

echo "== composite program trust =="
OUTPUT="$("$BIN" trust "$ROVER" 2>&1 || true)"
echo "$OUTPUT"
echo "$OUTPUT" | grep -q "Composite trust:"
echo "$OUTPUT" | grep -q "package_trust"
echo "$OUTPUT" | grep -q "device_integrity"

echo "== composite program trust json =="
JSON="$("$BIN" trust "$ROVER" --json 2>&1 || true)"
echo "$JSON" | grep -q '"score"'
echo "$JSON" | grep -q '"integrity_status"'

echo "trust program smoke ok"
