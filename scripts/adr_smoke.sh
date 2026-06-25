#!/usr/bin/env bash
# Smoke architecture decision record generation.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/policy/warehouse.sd"
OUT="${ROOT}/.spanda/adr-smoke"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

rm -rf "$OUT"

echo "== adr markdown =="
run_spanda adr "$FILE" >/dev/null

echo "== adr json =="
run_spanda adr "$FILE" --json >/dev/null

echo "== adr out dir =="
run_spanda adr "$FILE" --out "$OUT" >/dev/null
test -f "${OUT}/architecture-decisions.md"

echo "ADR smoke OK"
