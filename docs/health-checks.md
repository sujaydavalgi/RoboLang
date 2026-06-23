# Health Checks

Spanda provides unified health monitoring for robots, devices, sensors, actuators, agents, fleets, and swarms.

## Health statuses

`Healthy`, `Degraded`, `Warning`, `Critical`, `Failed`, `Unknown`, `Offline`, `Unsafe`

## Robot health check

```spanda
health_check RoverHealth for robot Rover {
    check battery.level > 20%;
    check gps.status == Healthy;
    check wheels.status == Healthy;
    check emergency_stop.available == true;
}
```

## Sensor / actuator health

```spanda
health_check GPSHealth for sensor gps {
    check gps.fix_available == true;
    check gps.accuracy <= 3 m;
}

health_check WheelsHealth for actuator wheels {
    check wheels.temperature < 70 C;
    check wheels.emergency_stop_supported == true;
}
```

## Fleet / swarm health

```spanda
health_check FleetHealth for fleet WarehouseFleet {
    require at_least 80% robots Healthy;
    require no robot Unsafe;
    check rover.status == Healthy;
}
```

Fleet `require` clauses are parsed and evaluated at runtime against fleet membership and monitor faults.

## Health policies

```spanda
health_policy SafetyPolicy {
    on Critical { enter degraded_mode; notify_operator; }
    on Failed { stop_all_actuators; audit.record("health_failed"); }
    on Unsafe { trigger kill_switch EmergencyStop; }
}
```

At runtime, matching reactions execute when health status transitions (wired to `HardwareMonitor` polling). Fleet-target checks are refined against fleet membership and member faults via `apply_fleet_health_checks`. Swarm programs log coordination events when fleet health is critical. Policies latch per `(policy, status)` until health returns to `Healthy`.

**Example:** [`examples/hardware/capability_verification.sd`](../examples/hardware/capability_verification.sd) · Fleet requires: [`examples/features/fleet_health_require.sd`](../examples/features/fleet_health_require.sd)

---

## Typed handler returns

Return type annotations on behaviors, tasks, triggers, and events are documented in [typed-handler-io.md](./typed-handler-io.md).

---

## Health triggers

```spanda
on health Rover becomes Degraded {
    reduce_speed(0.5 m/s);
}
```

## CLI

```bash
spanda health robot rover.sd
spanda health robot rover.sd --json
spanda trace health rover.sd
spanda verify rover.sd --health
spanda sim rover.sd --inject-health-faults
```

See also [Fleet Health](./fleet-health.md) and [Swarm Health](./swarm-health.md).
