# Fleet Health

Fleet-level health checks aggregate robot health, evaluate `require` clauses at runtime, and coordinate mission pauses when thresholds fail.

**Example:** [`examples/features/fleet_health_require.sd`](../examples/features/fleet_health_require.sd)

---

## Syntax

```spanda
fleet WarehouseFleet {
    RoverA;
    RoverB;
}

health_check FleetHealth for fleet WarehouseFleet {
    require at_least 80% robots Healthy;
    require no robot Unsafe;
    require coordinator.status == Healthy;
    check rover.status == Healthy;
}

health_policy FleetPolicy {
    on Critical { enter degraded_mode; }
}

on health fleet becomes Critical {
    pause_new_missions();
}
```

---

## Runtime evaluation

Phase 35 wired fleet `require` clauses into `apply_fleet_health_checks`:

| Clause | Meaning |
|--------|---------|
| `require at_least N% robots Healthy` | Fraction of fleet members reporting Healthy |
| `require no robot Unsafe` | Fails if any member is Unsafe |
| `require coordinator.status == Healthy` | Fleet orchestrator health |

Simulate faults:

```bash
spanda sim examples/features/fleet_health_require.sd --inject-health-faults
spanda health robot examples/features/fleet_health_require.sd --json
spanda verify examples/features/fleet_health_require.sd --health
```

---

## Related

- [Health Checks](./health-checks.md) — robot/sensor/actuator checks
- [Swarm Health](./swarm-health.md) — quorum and mesh checks
- [Concurrency](./concurrency.md) — `spanda fleet orchestrate`, mesh relay
