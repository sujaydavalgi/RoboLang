# Swarm Health

Swarm health checks verify quorum and mesh connectivity. When fleet health is critical, swarm coordination logs events and may defer peer deliveries (Phase 34–35).

**Example:** [`examples/robotics/swarm_coordination.sd`](../examples/robotics/swarm_coordination.sd)

---

## Syntax

```spanda
health_check SwarmHealth for swarm DroneSwarm {
    require quorum >= 70%;
    require communication.mesh_connected == true;
}
```

---

## CLI

```bash
spanda swarm coordinate examples/robotics/swarm_coordination.sd
spanda swarm coordinate examples/robotics/swarm_coordination.sd --mesh-url http://127.0.0.1:8787
spanda health robot examples/robotics/swarm_coordination.sd --json
```

Fleet health refinement runs via `apply_fleet_health_checks` when programs declare both `fleet` and `swarm` blocks.

---

## Related

- [Health Checks](./health-checks.md)
- [Fleet Health](./fleet-health.md)
- [Concurrency](./concurrency.md) — swarm policies and mesh relay
