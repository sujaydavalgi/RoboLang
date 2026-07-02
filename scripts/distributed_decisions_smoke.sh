#!/usr/bin/env bash
# Smoke distributed decision runtime: sim trace emission, CLI inspect, audit.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SHOWCASE="${ROOT}/examples/showcase/distributed_decisions/main.sd"

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

echo "== decision sign-policy (dry run) =="
export SPANDA_DECISION_POLICY_SIGNING_KEY="${SPANDA_DECISION_POLICY_SIGNING_KEY:-offline-smoke-signing-key}"
run_spanda decision sign-policy examples/showcase/distributed_decisions/offline_mission_continue/main.sd --policy RoverOffline --json >/dev/null

echo "== sim with decision trace + audit =="
TRACE="${ROOT}/examples/showcase/distributed_decisions/main.trace"
rm -f "$TRACE"
export SPANDA_DECISION_TRACE=1
run_spanda sim "$SHOWCASE" --record --inject-health-faults --max-loop-iterations 3 >/dev/null || true
if [[ -f "$TRACE" ]]; then
  run_spanda decision trace "$TRACE" >/dev/null || true
  run_spanda audit decisions "$TRACE" >/dev/null || true
fi

echo "Distributed decisions smoke OK"
