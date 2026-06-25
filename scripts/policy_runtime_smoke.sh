#!/usr/bin/env bash
# Smoke runtime operational policy enforcement during simulation.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/policy/warehouse.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== sim warehouse with runtime policy =="
run_spanda sim "$FILE" --enforce-policy WarehousePolicy >/dev/null

echo "Policy runtime smoke OK"
