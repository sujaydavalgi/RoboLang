# ADAS application variants

Device-tree configurations for common intelligent-vehicle deployments. Copy or layer with `spanda config` / `--config`.

| Application | Device tree | Profile | Notes |
|-------------|-------------|---------|-------|
| Passenger (default) | `passenger/spanda.toml` → `../spanda.devices.toml` | `iso26262` | Full sensor suite |
| Commercial truck | `truck/spanda.devices.toml` | `iso26262` | Long-range radar, rear radar |
| Autonomous shuttle | `shuttle/spanda.devices.toml` | `iso26262` | Geo-fenced, pedestrian focus |
| Campus shuttle | `campus/spanda.devices.toml` | `iso26262` | Multi-camera pedestrian suite |
| Mining vehicle | `mining/spanda.devices.toml` | `iec61508` | Ruggedized LiDAR, redundant GPS |
| Delivery vehicle | `delivery/spanda.devices.toml` | `warehouse` | Urban mix, parking assist |
| Agricultural | `agricultural/spanda.devices.toml` | `agriculture` | RTK GPS, outdoor connectivity |
| Airport ground | `airport/spanda.devices.toml` | `iso26262` | Geofence gateway, no driver monitor |
| Construction | `construction/spanda.devices.toml` | `iso13849` | 360° LiDAR, machinery safety |

Scenario traces for playback and diagnosis: [`fixtures/`](../fixtures/)

Sim-recorded golden trace (scheduler frames): [`sim_record/lane_keep_task.trace`](../sim_record/lane_keep_task.trace)

```bash
spanda device-tree inspect vehicle-truck-001 \
  --config examples/solutions/adas/applications/truck/spanda.toml
spanda readiness src/highway_drive.sd --profile iso26262 \
  --config examples/solutions/adas/spanda.toml
spanda replay sim_record/lane_keep_task.trace --deterministic
```

See [docs/solutions/adas.md](../../../docs/solutions/adas.md#applications).
