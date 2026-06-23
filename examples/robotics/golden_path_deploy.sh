#!/usr/bin/env bash
# Golden-path robotics workflow: certify, deploy, fleet, swarm, mesh, and adapter verify.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [[ -n "${SPANDA:-}" ]]; then
  :
elif [[ -x "${ROOT}/target/release/spanda" ]]; then
  SPANDA="${ROOT}/target/release/spanda"
else
  SPANDA="spanda"
fi
export SPANDA

CERTIFIED="${ROOT}/examples/robotics/certified_deployment.sd"
REMOTE="${ROOT}/examples/robotics/remote_ota_deployment.sd"
FLEET="${ROOT}/examples/robotics/fleet_peer_missions.sd"
SWARM="${ROOT}/examples/robotics/swarm_coordination.sd"
NAV2_PKG="${ROOT}/examples/packages/nav2_adapter_package"
GOLDEN_DIR="${ROOT}/.spanda/golden-path"
FLEET_AGENT_BIND_B="127.0.0.1:18766"
FLEET_AGENT_BIND_C="127.0.0.1:18768"
MESH_BIND="127.0.0.1:18767"
DEPLOY_AGENT_BIND="127.0.0.1:18765"
DEPLOY_TARGET="RoverProgram@JetsonOrin"

FLEET_AGENT_PID_B=""
FLEET_AGENT_PID_C=""
MESH_PID=""
DEPLOY_AGENT_PID=""

cleanup() {
  [[ -n "${DEPLOY_AGENT_PID}" ]] && kill "${DEPLOY_AGENT_PID}" 2>/dev/null || true
  [[ -n "${MESH_PID}" ]] && kill "${MESH_PID}" 2>/dev/null || true
  [[ -n "${FLEET_AGENT_PID_C}" ]] && kill "${FLEET_AGENT_PID_C}" 2>/dev/null || true
  [[ -n "${FLEET_AGENT_PID_B}" ]] && kill "${FLEET_AGENT_PID_B}" 2>/dev/null || true
  for port in 18765 18766 18767 18768; do
    lsof -ti ":${port}" 2>/dev/null | xargs kill -9 2>/dev/null || true
  done
}
trap cleanup EXIT

start_background() {
  "$@" &
  local pid=$!
  disown "${pid}" 2>/dev/null || true
  echo "${pid}"
}

for port in 18765 18766 18767 18768; do
  lsof -ti ":${port}" 2>/dev/null | xargs kill -9 2>/dev/null || true
done

mkdir -p "${GOLDEN_DIR}"
export SPANDA_FLEET_AGENTS="${GOLDEN_DIR}/fleet-agents.json"
export SPANDA_DEPLOY_AGENTS="${GOLDEN_DIR}/deploy-agents.json"
export SPANDA_DEPLOY_STATE="${GOLDEN_DIR}/deploy-state.json"
export SPANDA_SWARM_STATE="${GOLDEN_DIR}/swarm-state.json"
export SPANDA_NAV2_CMD="bash ${ROOT}/examples/adapters/nav2_bridge.sh {goal}"
export SPANDA_SLAM_CMD="bash ${ROOT}/examples/adapters/slam_bridge.sh {op}"

echo "== adapter bridge fixtures =="
bash "${ROOT}/examples/adapters/slam_bridge.sh" localize | grep -q "slam-bridge"
bash "${ROOT}/examples/adapters/nav2_bridge.sh" Dock | grep -q "nav2-bridge"

echo "== check certified deployment =="
"${SPANDA}" check "${CERTIFIED}"

echo "== verify with strict certify =="
"${SPANDA}" verify "${CERTIFIED}" --all-targets --strict-certify

echo "== certification proof artifact =="
"${SPANDA}" certify prove "${CERTIFIED}" --strict --out /tmp/spanda-certified-proof.json

echo "== deploy plan with certification summary =="
"${SPANDA}" deploy plan "${CERTIFIED}" --version 1.0.0

echo "== dry-run rollout with --require-certify =="
"${SPANDA}" deploy rollout "${CERTIFIED}" --require-certify --dry-run --version 1.0.0

echo "== remote OTA example (plan) =="
"${SPANDA}" deploy plan "${REMOTE}" --version 1.3.0

echo "== remote OTA dry-run rollout =="
"${SPANDA}" deploy rollout "${REMOTE}" --remote --require-certify --dry-run --version 1.3.0

echo "== verify Nav2 adapter package =="
"${SPANDA}" verify-adapter --project "${NAV2_PKG}" --import navigation.nav2

echo "== start fleet mesh services =="
: > "${SPANDA_FLEET_AGENTS}"
FLEET_AGENT_PID_B="$(start_background "${SPANDA}" fleet agent start --robot ScoutB --bind "${FLEET_AGENT_BIND_B}")"
FLEET_AGENT_PID_C="$(start_background "${SPANDA}" fleet agent start --robot ScoutC --bind "${FLEET_AGENT_BIND_C}")"
sleep 1
"${SPANDA}" fleet agent register ScoutB "http://${FLEET_AGENT_BIND_B}"
"${SPANDA}" fleet agent register ScoutC "http://${FLEET_AGENT_BIND_C}"
MESH_PID="$(start_background "${SPANDA}" fleet mesh start --bind "${MESH_BIND}")"
sleep 1

echo "== fleet orchestration (local) =="
"${SPANDA}" fleet orchestrate "${FLEET}"

echo "== fleet orchestration (remote HTTP relay) =="
"${SPANDA}" fleet orchestrate "${FLEET}" --remote

echo "== fleet orchestration via mesh =="
"${SPANDA}" fleet orchestrate "${FLEET}" --mesh-url "http://${MESH_BIND}"

echo "== swarm coordination (round_robin tick 1) =="
"${SPANDA}" swarm coordinate "${SWARM}"

echo "== swarm coordination (round_robin tick 2) =="
"${SPANDA}" swarm coordinate "${SWARM}"

echo "== swarm coordination via mesh =="
rm -f "${SPANDA_SWARM_STATE}"
"${SPANDA}" swarm coordinate "${SWARM}" --mesh-url "http://${MESH_BIND}"

echo "== fleet field trial (3 agents) =="
FLEET_TRIAL="${ROOT}/examples/robotics/fleet_field_trial.sd"
"${SPANDA}" check "${FLEET_TRIAL}"
"${SPANDA}" fleet orchestrate "${FLEET_TRIAL}" --remote

echo "== start remote deploy agent =="
: > "${SPANDA_DEPLOY_AGENTS}"
DEPLOY_AGENT_PID="$(start_background "${SPANDA}" deploy agent start --target "${DEPLOY_TARGET}" --require-certify --bind "${DEPLOY_AGENT_BIND}")"
sleep 1
"${SPANDA}" deploy agent register "${DEPLOY_TARGET}" "http://${DEPLOY_AGENT_BIND}"

echo "== remote OTA live dry-run against agent registry =="
"${SPANDA}" deploy rollout "${REMOTE}" --remote --require-certify --dry-run --version 1.3.0

cleanup

echo "Robotics golden path complete."
