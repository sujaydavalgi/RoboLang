# ROS 2 Automotive Stack Bridge

Reference workflow for bridging the ADAS blueprint to **ROS 2 Humble** perception and **Nav2** motion stacks via `spanda-ros2`.

## Prerequisites

```bash
cd examples/solutions/adas
spanda install   # resolves spanda-ros2 from registry
source /opt/ros/humble/setup.bash   # when using live ROS 2
```

## Verify and simulate

```bash
spanda check ros2_automotive/automotive_nav.sd
spanda verify ros2_automotive/automotive_nav.sd --capabilities --profile iso26262
spanda sim ros2_automotive/automotive_nav.sd
```

## Live ROS 2 bridge

```bash
export SPANDA_ROS2_LIVE=1
spanda run ros2_automotive/automotive_nav.sd
```

`automotive_nav.sd` publishes `/cmd_vel` for Nav2 or manual teleop verification. Optional Nav2 goal bridge: `SPANDA_NAV2_CMD` (see [docs/ffi-and-ecosystem.md](../../../docs/ffi-and-ecosystem.md)).

## Topic map (reference)

| Spanda surface | ROS 2 topic | Role |
|----------------|-------------|------|
| `cmd_vel` publish | `/cmd_vel` | Motion commands to Nav2 controller |
| `front_lidar.read()` | `/scan` (via bridge) | Obstacle avoidance |
| `front_camera.analyze()` | `/camera/image` (via bridge) | Vision processing |

## Related

- [solutions/adas.md](../../../docs/solutions/adas.md)
- [spanda-ros2](../../../packages/registry/spanda-ros2/README.md)
- [ros2_cmd_vel_ping.sd](../../../examples/communication/ros2_cmd_vel_ping.sd)
