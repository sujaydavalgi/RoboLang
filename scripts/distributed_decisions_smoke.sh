#!/usr/bin/env bash
# Smoke distributed decision runtime: sim trace emission, CLI inspect, audit, cache, API.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SHOWCASE="${ROOT}/examples/showcase/distributed_decisions/main.sd"
SIM="${ROOT}/examples/showcase/distributed_decisions/obstacle_reflex_stop/main.sd"
OFFLINE="${ROOT}/examples/showcase/distributed_decisions/offline_mission_continue/main.sd"
TRACE="${ROOT}/examples/showcase/distributed_decisions/obstacle_reflex_stop/main.trace"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-decision crate tests =="
cargo test -p spanda-decision --quiet

echo "== interpreter decision runtime tests =="
cargo test -p spanda-interpreter --test decision_runtime --quiet

echo "== decision CLI inspect =="
run_spanda decision list "$SHOWCASE" >/dev/null
run_spanda decision inspect "$SHOWCASE" --entity Rover001 --action emergency_stop >/dev/null
run_spanda decision simulate "$SHOWCASE" --offline >/dev/null

echo "== decision sign-policy and cache sync =="
export SPANDA_DECISION_POLICY_SIGNING_KEY="${SPANDA_DECISION_POLICY_SIGNING_KEY:-offline-smoke-signing-key}"
run_spanda decision sign-policy "$OFFLINE" --policy RoverOffline --json >/dev/null
run_spanda decision cache sync "$OFFLINE" --sign --json >/dev/null
run_spanda decision cache show --json >/dev/null

echo "== API policy cache and sim trace tests =="
cargo test -p spanda-api --test decision_traces_api_tests --quiet
cargo test -p spanda-api --test openapi_parity_tests --quiet

echo "== sim with decision trace + audit =="
rm -f "$TRACE"
export SPANDA_DECISION_TRACE=1
run_spanda sim "$SIM" --record --inject-health-faults >/dev/null
if [[ -f "$TRACE" ]]; then
  run_spanda decision trace "$TRACE" >/dev/null
  run_spanda audit decisions "$TRACE" >/dev/null
fi

echo "== showcase typecheck =="
run_spanda check "$SIM" >/dev/null
run_spanda check "$SHOWCASE" >/dev/null

echo "Distributed decisions smoke OK"
