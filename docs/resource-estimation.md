# Mission Resource Estimation

**Status:** Planned · **Phase:** Simulate, Deploy · **Priority:** P2.3

Estimate mission cost before execution.

## CLI

```bash
spanda estimate mission.sd
spanda estimate mission.sd --target RoverV1 --json
```

## Estimates

| Resource | Source |
|----------|--------|
| Battery | Mission duration × power model from hardware profile |
| CPU | Task budgets, agent inference declarations |
| Memory | Hardware profile + package requirements |
| Storage | Trace/replay buffers, log policies |
| Network | Topic rates, connectivity policy |
| Mission duration | `mission { }` timing and task schedules |

## Output

`MissionEstimate` — per-resource estimates with confidence and assumptions.

## Integration

Composes `spanda-hardware`, `spanda-capability`, readiness mission verification, and sim telemetry.

See [hardware-compatibility.md](./hardware-compatibility.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
