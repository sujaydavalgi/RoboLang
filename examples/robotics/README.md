# Robotics examples

Runnable `.sd` programs demonstrating the Spanda robotics platform. See [robotics platform guide](../docs/robotics-platform.md).

| Example | Focus |
|---------|--------|
| `mission_management.sd` | Named mission steps and lifecycle |
| `fleet_management.sd` | Program-level `fleet` declarations |
| `fleet_peer_missions.sd` | Peer mesh fleet orchestration |
| `swarm_coordination.sd` | Swarm policies (`round_robin`, `broadcast`, `leader_follow`) |
| `safety_zones.sd` | Program-level `safety_zone` speed caps |
| `sensor_fusion.sd` | `observe` + `fusion.read()` |
| `navigation.sd` | Navigation stdlib helpers |
| `nav2_bridge.sd` | Nav2 golden path (`/cmd_vel` publish) |
| `slam_integration.sd` | SLAM package orchestration pattern |
| `ota_deployment.sd` | Local OTA deploy targets |
| `certified_deployment.sd` | `certify` metadata for verify/deploy |
| `remote_ota_deployment.sd` | Remote OTA via deploy agents |
| `edge_cloud.sd` | Edge/cloud split pattern |
| `predictive_maintenance.sd` | Maintenance-oriented robot program |

**Scripts**

- `golden_path_deploy.sh` — check → verify → certify prove → deploy plan/rollout dry-run → verify-adapter → fleet orchestrate → swarm coordinate
