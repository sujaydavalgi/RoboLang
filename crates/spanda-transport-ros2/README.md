# spanda-transport-ros2

ROS 2 transport backend for Spanda. Extracted from `spanda-core` as part of the lean-core architecture.

## Backends

1. **Native rclrs** — dynamically loads `libspanda_ros2_rclrs_native` when `SPANDA_ROS2_RCLRS=1`
2. **rclpy daemon** — persistent subprocess via `scripts/spanda_ros2_daemon.py`

Spanda core retains the per-call Python bridge fallback and `RuntimeValue` conversion shims.

## Related crates

- `spanda-ros2-rclrs-native` — build the native cdylib (requires ROS 2 Humble)
- `spanda-mqtt` — official package under `packages/registry/spanda-ros2`
