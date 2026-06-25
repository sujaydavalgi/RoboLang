#!/usr/bin/env bash
# Smoke chaos engineering experiments.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/self_healing/rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== chaos default injections =="
run_spanda chaos "$FILE" >/dev/null

echo "== chaos gps injection json =="
run_spanda chaos "$FILE" --inject gps-failure --json >/dev/null

echo "Chaos smoke OK"
