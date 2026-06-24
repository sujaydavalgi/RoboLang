# State Estimation

State estimation declares **sensor fusion inputs** and **estimate types** for mission assurance.

## Syntax

```spanda
state_estimator RoverState {
    inputs [gps.fix, imu.data, wheel_odometry];
    output StateEstimate;
}
```

## Core types

| Type | Role |
|------|------|
| `StateEstimate` | Named estimate with confidence and source attribution |
| `BeliefState` | Aggregated estimates from all estimators |
| `SensorFusionState` | Per-estimator fusion snapshot |

## Analysis

`spanda state estimate <file.sd>` runs static analysis:

- Extracts estimator inputs and **weighted fusion previews** (by sensor type)
- Builds an aggregate belief state
- Validates non-empty inputs

Results are also included in `spanda assure` JSON under `state`.

## Runtime

At robot setup, each `state_estimator` registers a `SensorFusion` binding. A single estimator aliases `fusion` (same as `observe { }`). `fusion.read()` / `{Name}.read()` perform **weighted fusion** by sensor type (GPS, Lidar, IMU, …), include `sources` and `estimator` fields, and populate `state_estimate.confidence`.

Optional **`spanda-fusion`** package (`import assurance.fusion;`) exposes the same weight table via `weight_for_sensor` and `confidence_for_types` for custom tooling.

## Example

See `examples/assurance/rover_assurance.sd`.

## Related

- [Mission assurance](mission-assurance.md)
- [Knowledge models](knowledge-models.md)
