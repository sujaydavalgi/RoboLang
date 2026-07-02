#!/usr/bin/env bash
# Smoke what-if failure scenario analysis (NEXT differentiation pillar).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/what_if/gps_failure.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-whatif crate tests =="
cargo test -p spanda-whatif --quiet

echo "== what-if default scenarios =="
run_spanda what-if "$FILE" >/dev/null

echo "== what-if gps_failure scenario =="
run_spanda what-if "$FILE" --scenario gps_failure --json >/dev/null

echo "== demo what-if =="
export SPANDA_ROOT="${ROOT}"
run_spanda demo what-if >/dev/null

echo "What-if smoke OK"
