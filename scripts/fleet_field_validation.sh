#!/usr/bin/env bash
# Multi-process fleet field validation: agents, mesh, orchestrate, recovery, continuity.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

resolve_spanda_bin() {
  if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
    echo "${SPANDA_BIN}"
    return
  fi
  local bases=("${ROOT}/target")
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    bases+=("${CARGO_TARGET_DIR}")
  fi
  for base in "${bases[@]}"; do
    for candidate in "${base}/debug/spanda" "${base}/release/spanda"; do
      if [[ -x "${candidate}" ]]; then
        echo "${candidate}"
        return
      fi
    done
  done
}

if resolved="$(resolve_spanda_bin)"; [[ -n "${resolved}" ]]; then
  export SPANDA_BIN="${resolved}"
  run_spanda() { "${SPANDA_BIN}" "$@"; }
else
  unset SPANDA_BIN
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

FIELD_DIR="${ROOT}/.spanda/field-validation"
FLEET_PEER="${ROOT}/examples/robotics/fleet_peer_missions.sd"
FLEET_RECOVERY="${ROOT}/examples/showcase/fleet_recovery/fleet.sd"
CONTINUITY="${ROOT}/examples/showcase/continuity/warehouse.sd"
BIND_A="127.0.0.1:19701"
BIND_B="127.0.0.1:19702"
MESH_BIND="127.0.0.1:19703"
AGENT_PID_A=""
AGENT_PID_B=""
MESH_PID=""

cleanup() {
  [[ -n "${MESH_PID}" ]] && kill "${MESH_PID}" 2>/dev/null || true
  [[ -n "${AGENT_PID_B}" ]] && kill "${AGENT_PID_B}" 2>/dev/null || true
  [[ -n "${AGENT_PID_A}" ]] && kill "${AGENT_PID_A}" 2>/dev/null || true
  for port in 19701 19702 19703; do
    lsof -ti ":${port}" 2>/dev/null | xargs kill -9 2>/dev/null || true
  done
}
trap cleanup EXIT

mkdir -p "${FIELD_DIR}"
export SPANDA_FLEET_AGENTS="${FIELD_DIR}/fleet-agents.json"
: > "${SPANDA_FLEET_AGENTS}"

for port in 19701 19702 19703; do
  lsof -ti ":${port}" 2>/dev/null | xargs kill -9 2>/dev/null || true
done

echo "== start fleet agents =="
run_spanda fleet agent start --robot ScoutA --bind "${BIND_A}" &
AGENT_PID_A=$!
run_spanda fleet agent start --robot ScoutB --bind "${BIND_B}" &
AGENT_PID_B=$!
sleep 2
run_spanda fleet agent register ScoutA "http://${BIND_A}"
run_spanda fleet agent register ScoutB "http://${BIND_B}"

echo "== start mesh coordinator =="
run_spanda fleet mesh start --bind "${MESH_BIND}" &
MESH_PID=$!
sleep 1

echo "== fleet orchestrate local/remote/mesh =="
run_spanda fleet orchestrate "${FLEET_PEER}" >/dev/null
run_spanda fleet orchestrate "${FLEET_PEER}" --remote >/dev/null
mesh_json=$(run_spanda fleet orchestrate "${FLEET_PEER}" --mesh-url "http://${MESH_BIND}" --json)
echo "${mesh_json}" | grep -q 'peer_deliveries' || { echo "${mesh_json}"; exit 1; }

echo "== mesh integration tests (recovery + continuity) =="
cargo test -p spanda-fleet --test mesh_integration mesh_coordinator_relays_fleet_recovery --quiet
cargo test -p spanda-fleet --test mesh_integration swarm_continuity_handoff_relays_through_mesh --quiet

echo "== continuity CLI on showcase =="
run_spanda continuity "${CONTINUITY}" --failed ScannerAlpha --progress 72 --trigger robot_failed >/dev/null

echo "== recovery CLI on fleet showcase =="
run_spanda heal "${FLEET_RECOVERY}" >/dev/null

echo "== recovery runtime auto-trigger tests =="
cargo test -p spanda-interpreter recovery_auto_triggers --quiet

echo "== fleet agent interpreter recovery smoke =="
"${ROOT}/scripts/fleet_agent_recovery_smoke.sh"

echo "Fleet field validation OK"
