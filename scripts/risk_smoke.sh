#!/usr/bin/env bash
# Smoke mission deployment risk scoring (NEXT differentiation pillar).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FILE="${ROOT}/examples/showcase/risk/deployment_risk.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-risk crate tests =="
cargo test -p spanda-risk --quiet

echo "== mission risk text =="
run_spanda risk "$FILE" >/dev/null

echo "== mission risk json =="
run_spanda risk "$FILE" --json >/dev/null

echo "== demo risk =="
export SPANDA_ROOT="${ROOT}"
run_spanda demo risk >/dev/null

echo "Mission risk smoke OK"
