# Fleet Readiness

Evaluate operational readiness across **fleet**, **swarm**, **group**, and **crowd** configurations.

## CLI

```bash
spanda fleet readiness examples/showcase/fleet_readiness/warehouse.sd
spanda verify-fleet examples/showcase/fleet_readiness/warehouse.sd
```

Example output:

```
Fleet Score: 88/100
Healthy Robots: 19
Degraded Robots: 1
Mission Capacity: 95%
```

## Fleet health integration

Fleet programs with `health_check` and `require` clauses are evaluated against fleet policies:

```spanda
fleet Warehouse { PickerA; PickerB; }

health_check FleetHealth for fleet Warehouse {
  require at_least 80% robots Healthy;
  require no robot Unsafe;
}
```

## Multi-robot verification

`spanda verify-fleet` checks for:

- Collision risk (missing shared safety zones)
- Deadlocks and resource contention
- Communication failures
- Mission conflicts
- Safety conflicts

## Dashboard model

`FleetDashboard` aggregates fleet score, healthy/degraded counts, and mission capacity for future UI dashboards.

## API

```rust
use spanda_readiness::{evaluate_fleet_readiness, verify_fleet, FleetReadinessReport};
```

See also: [Readiness](readiness.md), [Health Checks](health-checks.md).
