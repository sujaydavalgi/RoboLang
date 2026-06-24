#!/usr/bin/env bash
# Smoke mission assurance commands in CI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ROVER="${ROOT}/examples/showcase/assurance/rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-assurance crate tests =="
cargo test -p spanda-assurance --quiet

echo "== assurance CLI =="
run_spanda assure "$ROVER" --json >/dev/null
run_spanda anomaly scan "$ROVER" >/dev/null
run_spanda state estimate "$ROVER" >/dev/null
run_spanda prognostics "$ROVER" >/dev/null
run_spanda mission verify "$ROVER" >/dev/null
run_spanda resilience check "$ROVER" >/dev/null
run_spanda mitigation plan "$ROVER" >/dev/null

echo "== demo assurance =="
export SPANDA_ROOT="${ROOT}"
run_spanda demo assurance

echo "Assurance smoke OK"
