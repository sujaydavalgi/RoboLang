#!/usr/bin/env bash
# Smoke compliance profile verification.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/policy/warehouse.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== verify warehouse profile =="
run_spanda verify "$FILE" --profile warehouse >/dev/null

echo "== verify warehouse profile json =="
run_spanda verify "$FILE" --profile warehouse --json >/dev/null

echo "== readiness warehouse profile =="
run_spanda readiness "$FILE" --profile warehouse >/dev/null

echo "Compliance smoke OK"
