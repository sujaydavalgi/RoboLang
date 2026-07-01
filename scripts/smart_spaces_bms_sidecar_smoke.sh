#!/usr/bin/env bash
# Smoke Smart Spaces BMS sidecar bridges (Home Assistant mock + provider wiring).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BRIDGE="${ROOT}/scripts/spanda_python_bridge.py"

echo "== Home Assistant Python bridge mock =="
export SPANDA_LIVE_HOME_ASSISTANT=1
export SPANDA_HOME_ASSISTANT_FORCE_MOCK=1
RESULT="$(printf '%s\n' '{"fn":"home_assistant_get_state","args":["binary_sensor.leak_basement"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'
echo "${RESULT}" | grep -q 'mock-ha:binary_sensor.leak_basement'

echo "== Home Assistant package get_state.py mock =="
RESULT="$(python3 "${ROOT}/packages/registry/spanda-home-assistant/scripts/get_state.py" binary_sensor.leak_basement)"
echo "${RESULT}" | grep -q 'mock-ha:binary_sensor.leak_basement'

if [[ "${SPANDA_BMS_SIDECAR_LIVE:-0}" == "1" ]]; then
  echo "== Home Assistant live REST =="
  unset SPANDA_HOME_ASSISTANT_FORCE_MOCK
  python3 "${ROOT}/packages/registry/spanda-home-assistant/scripts/get_state.py" "${SPANDA_HA_LIVE_ENTITY:-sensor.time}"
fi

echo "Smart Spaces BMS sidecar smoke complete."
