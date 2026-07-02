#!/usr/bin/env bash
# Smoke autonomous systems scorecard rollup.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/scorecard/executive.sd"
READINESS="${ROOT}/examples/showcase/readiness/rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== score text =="
run_spanda score "$FILE" >/dev/null

echo "== score json =="
run_spanda score "$FILE" --json >/dev/null

echo "== score markdown =="
run_spanda score "$FILE" --format markdown >/dev/null

echo "== score readiness baseline =="
run_spanda score "$READINESS" >/dev/null

echo "Scorecard smoke OK"
