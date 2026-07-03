#!/usr/bin/env bash
# Smoke distributed decision runtime: enforcement tests, attack sims, GPS demo, trace audit.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SHOWCASE="${ROOT}/examples/showcase/distributed_decisions/main.sd"
SIM="${ROOT}/examples/showcase/distributed_decisions/obstacle_reflex_stop/main.sd"
OFFLINE="${ROOT}/examples/showcase/distributed_decisions/offline_mission_continue/main.sd"
GPS_DEMO="${ROOT}/examples/showcase/distributed_decisions/gps_loss_recovery/mission.sd"
TRACE="${ROOT}/examples/showcase/distributed_decisions/obstacle_reflex_stop/main.trace"
GPS_TRACE="${ROOT}/examples/showcase/distributed_decisions/gps_loss_recovery/mission.trace"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-decision crate tests =="
cargo test -p spanda-decision --quiet

echo "== rule enforcement tests =="
cargo test -p spanda-decision --test rule_enforcement --quiet

echo "== attack simulation tests =="
cargo test -p spanda-decision --test attack_simulations --quiet

echo "== interpreter decision runtime tests =="
cargo test -p spanda-interpreter --test decision_runtime --quiet

echo "== decision CLI inspect =="
run_spanda decision list "$SHOWCASE" >/dev/null
run_spanda decision inspect "$SHOWCASE" --entity Rover001 --action emergency_stop >/dev/null
run_spanda decision simulate "$SHOWCASE" --offline >/dev/null

echo "== attack simulations (live enforcement) =="
run_spanda decision simulate-attack policy-tamper >/dev/null
run_spanda decision simulate-attack replayed-decision >/dev/null
run_spanda decision simulate-attack fake-coordinator >/dev/null
run_spanda decision simulate-attack offline-abuse >/dev/null

echo "== stable gap-fix tests =="
cargo test -p spanda-decision --test stable_gaps --quiet

echo "== fleet mesh decision aggregation + shared nonce =="
cargo test -p spanda-fleet mesh_coordinator_resolves_decision_conflicts_and_shared_nonce -q

echo "== decision sign-policy, sign-tree, and cache sync =="
export SPANDA_DECISION_POLICY_SIGNING_KEY="${SPANDA_DECISION_POLICY_SIGNING_KEY:-offline-smoke-signing-key}"
run_spanda decision sign-policy "$OFFLINE" --policy RoverOffline --json >/dev/null
run_spanda decision sign-tree "$GPS_DEMO" --tree GPSLossRecovery --write-cache --json >/dev/null
run_spanda decision cache sync "$OFFLINE" --sign --json >/dev/null
run_spanda decision cache show --json >/dev/null

echo "== API policy cache and sim trace tests =="
cargo test -p spanda-api --test decision_traces_api_tests --quiet

echo "== sim with decision trace + audit =="
rm -f "$TRACE"
export SPANDA_DECISION_TRACE=1
run_spanda sim "$SIM" --record --inject-health-faults >/dev/null
if [[ -f "$TRACE" ]]; then
  run_spanda decision trace "$TRACE" >/dev/null
  run_spanda audit decisions "$TRACE" >/dev/null
fi

echo "== flagship GPS loss recovery demo =="
run_spanda check "$GPS_DEMO" >/dev/null
run_spanda decision simulate "$GPS_DEMO" --offline --entity Rover001 >/dev/null
run_spanda decision inspect "$GPS_DEMO" --entity Rover001 --action degraded_mode --signal "gps.status == Failed=true,visual_odometry.available=true" >/dev/null
rm -f "$GPS_TRACE"
run_spanda sim "$GPS_DEMO" --record --inject-health-faults >/dev/null || true
if [[ -f "$GPS_TRACE" ]]; then
  run_spanda replay "$GPS_TRACE" >/dev/null
  run_spanda decision trace "$GPS_TRACE" >/dev/null
  run_spanda audit decisions "$GPS_TRACE" >/dev/null
fi
run_spanda assure "$GPS_DEMO" >/dev/null || true

echo "== showcase typecheck =="
run_spanda check "$SIM" >/dev/null
run_spanda check "$SHOWCASE" >/dev/null

echo "Distributed decisions smoke OK"
