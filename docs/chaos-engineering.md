# Chaos Engineering

**Status:** Planned · **Phase:** Simulate, Recover · **Priority:** P2.1

Validate resilience by injecting controlled failures and verifying recovery, health, readiness, and safety.

## CLI

```bash
spanda chaos rover.sd
spanda chaos rover.sd --inject gps-failure
spanda chaos rover.sd --inject camera-failure,connectivity-failure
spanda chaos rover.sd --json
```

## Injection kinds

| Kind | Foundation |
|------|------------|
| GPS failure | `simulate_compatibility`, connectivity runtime |
| Camera failure | Health check faults, `sim --inject-health-faults` |
| Connectivity failure | Connectivity policy + sim inject |
| Provider failure | Provider dispatch mock failure |
| Package failure | Missing provider backend |
| Battery failure | Mission battery budget |

## Verification after chaos

Each run reports pass/fail for:

- Recovery (`spanda-assurance` recovery planner)
- Health (`health_check` status)
- Readiness (`evaluate_readiness`)
- Safety (no unsafe actuator commands)

## Crate

`spanda-chaos` — orchestrates sim inject + assurance + readiness evaluation.

See [self-healing.md](./self-healing.md) · [resilience.md](./resilience.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
