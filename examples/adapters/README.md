# Reference adapter bridges

Use these stub scripts with Spanda production bridge environment variables:

```bash
export SPANDA_NAV2_CMD="$PWD/examples/adapters/nav2_bridge.sh {goal}"
export SPANDA_SLAM_CMD="$PWD/examples/adapters/slam_bridge.sh {op}"

spanda run examples/robotics/nav2_bridge.sd
spanda run examples/robotics/slam_integration.sd
```

Validate adapter package metadata before wiring production backends:

```bash
spanda verify-adapter --project examples/packages/nav2_adapter_package --import navigation.nav2
spanda verify-adapter --project examples/packages/cartographer_adapter_package --import navigation.cartographer
spanda verify-adapter --project examples/packages/rtabmap_adapter_package --import navigation.rtabmap
```

Replace the scripts with wrappers around your Nav2 action client or Cartographer/RTAB-Map CLI. Spanda does not bundle Nav2 or SLAM binaries — it orchestrates external stacks through `[adapter]` packages and optional subprocess bridges.

Full robotics workflow script: `examples/robotics/golden_path_deploy.sh`.
