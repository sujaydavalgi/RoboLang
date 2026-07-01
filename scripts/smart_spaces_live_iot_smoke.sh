#!/usr/bin/env bash
# Smoke Smart Spaces live building I/O bridges (external cmd + Python mock fallback).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BRIDGE="${ROOT}/scripts/spanda_python_bridge.py"

echo "== BACnet external cmd =="
unset SPANDA_LIVE_BACNET SPANDA_BACNET_CMD
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_CMD='echo mock-bacnet:{device}:{object_id}'
cargo test -p spanda-providers live_bacnet_external_cmd_parses_stdout -- --nocapture

echo "== BACnet Python bridge mock (no bacpypes3 required) =="
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_FORCE_MOCK=1
RESULT="$(printf '%s\n' '{"fn":"bacnet_read_point","args":["ahu-12","analog-value,1"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'
echo "${RESULT}" | grep -q 'mock-bacnet:ahu-12:analog-value,1'

echo "== BACnet package read_point.py mock =="
RESULT="$(python3 "${ROOT}/packages/registry/spanda-bacnet/scripts/read_point.py" ahu-12 analog-value,1)"
echo "${RESULT}" | grep -q 'mock-bacnet:ahu-12:analog-value,1'

echo "== KNX Python bridge mock =="
export SPANDA_LIVE_KNX=1
export SPANDA_KNX_FORCE_MOCK=1
RESULT="$(printf '%s\n' '{"fn":"knx_read_group","args":["1/2/3"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'
echo "${RESULT}" | grep -q 'mock-knx:1/2/3'

echo "== KNX package read_group.py mock =="
RESULT="$(python3 "${ROOT}/packages/registry/spanda-knx/scripts/read_group.py" 1/2/3)"
echo "${RESULT}" | grep -q 'mock-knx:1/2/3'

if [[ "${SPANDA_LIVE_IOT_HARDWARE:-0}" == "1" ]]; then
  echo "== BACnet hardware (bacpypes3) =="
  unset SPANDA_BACNET_FORCE_MOCK
  python3 "${ROOT}/packages/registry/spanda-bacnet/scripts/read_point.py" "${SPANDA_BACNET_HW_DEVICE:-ahu-12}" "${SPANDA_BACNET_HW_OBJECT:-analog-value,1}"

  echo "== KNX hardware (xknx) =="
  unset SPANDA_KNX_FORCE_MOCK
  python3 "${ROOT}/packages/registry/spanda-knx/scripts/read_group.py" "${SPANDA_KNX_HW_ADDRESS:-1/2/3}"
fi

echo "== Thread Python bridge mock =="
export SPANDA_LIVE_THREAD=1
RESULT="$(printf '%s\n' '{"fn":"thread_read_endpoint","args":["thread-sensor-1"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== Z-Wave Python bridge mock =="
export SPANDA_LIVE_ZWAVE=1
RESULT="$(printf '%s\n' '{"fn":"zwave_read_value","args":["zwave-lock-1","DoorLock"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "== Home Assistant Python bridge mock =="
export SPANDA_LIVE_HOME_ASSISTANT=1
RESULT="$(printf '%s\n' '{"fn":"home_assistant_get_state","args":["binary_sensor.leak_basement"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q '"ok": true'

echo "Smart Spaces live IoT smoke complete."
