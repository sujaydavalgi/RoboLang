#!/usr/bin/env bash
# ADAS Solution Blueprint smoke — verify, readiness, replay, compliance, diagnosis.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ADAS="$ROOT/examples/solutions/adas"
MAIN="$ADAS/src/highway_drive.sd"
CONFIG="$ADAS/spanda.toml"

cd "$ROOT"
export SPANDA_ROOT="${SPANDA_ROOT:-$ROOT}"

# shellcheck source=lib/registry_env.sh
source "${ROOT}/scripts/lib/registry_env.sh"
ensure_spanda_registry_url "$ROOT"
cargo build -p spanda -q

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== ADAS Solution Blueprint smoke =="

check() {
  echo "--- $* ---"
  run_spanda "$@"
}

check check "$MAIN"
check verify "$MAIN" --profile iso26262 --capabilities --traceability
check readiness "$MAIN" --profile iso26262 --config "$CONFIG"
echo "--- replay src/highway_drive.trace --deterministic ---"
( cd "$ADAS" && run_spanda replay src/highway_drive.trace --deterministic )
check compliance report "$MAIN" --profile iso26262

echo "--- diagnose / explain (scenario fixtures) ---"
check diagnose "$MAIN" "$ADAS/fixtures/camera_failure_recovery.trace"
check explain "$ADAS/driver_takeover/driver_takeover.sd" "$ADAS/fixtures/driver_takeover.trace"
( cd "$ADAS" && run_spanda replay fixtures/aeb_activation.trace --playback )

for example in \
  "$ADAS/lane_keeping/lane_keeping.sd" \
  "$ADAS/adaptive_cruise/adaptive_cruise.sd" \
  "$ADAS/automatic_emergency_braking/aeb.sd" \
  "$ADAS/sensor_failure_recovery/camera_failure.sd" \
  "$ADAS/driver_takeover/driver_takeover.sd" \
  "$ADAS/parking_assist/parking_assist.sd" \
  "$ADAS/blind_spot_monitoring/blind_spot.sd" \
  "$ADAS/traffic_sign_recognition/traffic_sign.sd" \
  "$ADAS/pedestrian_detection/pedestrian.sd" \
  "$ADAS/ros2_automotive/automotive_nav.sd" \
  "$ADAS/canbus_gateway/canbus_gateway.sd"
do
  check check "$example"
  check verify "$example" --capabilities
done

echo "--- application device trees ---"
inspect_app() {
  local app="$1"
  local robot="$2"
  check device-tree inspect "$robot" --config "$ADAS/applications/${app}/spanda.toml"
}
inspect_app passenger vehicle-001
inspect_app truck vehicle-truck-001
inspect_app shuttle vehicle-shuttle-001
inspect_app campus vehicle-campus-001
inspect_app mining vehicle-mining-001
inspect_app delivery vehicle-delivery-001
inspect_app agricultural vehicle-agricultural-001
inspect_app airport vehicle-airport-001
inspect_app construction vehicle-construction-001

echo "--- sim-recorded trace replay ---"
( cd "$ADAS" && run_spanda replay sim_record/lane_keep_task.trace --deterministic )
( cd "$ADAS" && run_spanda replay lane_keeping/lane_keeping.trace --deterministic )

echo "--- continuity (camera_failure.sd) ---"
run_spanda continuity "$ADAS/sensor_failure_recovery/camera_failure.sd" \
  --failed front_camera --trigger sensor_failed || true

echo ""
echo "ADAS smoke complete."
