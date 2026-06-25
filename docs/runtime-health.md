# Runtime Health

Spanda aggregates runtime health from heartbeats, process health, active faults, and configured monitors.

## Status values

| Status | Meaning |
|--------|---------|
| `Healthy` | No active faults |
| `Warning` | Minor fault or configuration concern |
| `Degraded` | Fault affecting operation but not critical |
| `Critical` | Fault requiring immediate attention |
| `Crashed` | Process or runtime crash detected |
| `Rebooted` | Unexpected reboot detected |
| `Unknown` | Insufficient data |

## Types

- `RuntimeHealth` — overall health with heartbeats, processes, and active faults
- `ProcessHealth` — per-process status, exit code, restart count, uptime
- `HeartbeatStatus` — per-target heartbeat monitoring state
- `ResourcePressure` — CPU, memory, disk, GPU, battery, network pressure

## CLI

```bash
spanda runtime health rover.sd
spanda runtime health rover.sd --json
spanda fault scan rover.sd
```

## Integration

Runtime health feeds into:

- **Readiness** — fault impact on deployment readiness
- **Assurance** — uptime and crash-free duration evidence
- **Diagnosis** — causal explanation of faults
- **Recovery** — recommended recovery actions
- **Replay** — fault events in mission traces

## See also

- [Runtime Fault Detection](runtime-fault-detection.md)
- [Health Checks](health-checks.md)
- [Readiness](readiness.md)
