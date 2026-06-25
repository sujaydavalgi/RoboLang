# Chaos Engineering

**Status:** Experimental · **Phase:** Simulate, Recover · **Priority:** P2.1

Validate resilience by injecting controlled failures and verifying recovery, health, readiness, and post-injection safety.

## CLI

```bash
spanda chaos examples/showcase/self_healing/rover.sd
spanda chaos rover.sd --inject gps-failure
spanda chaos rover.sd --inject camera-failure,connectivity-failure
spanda chaos rover.sd --json
```

When `--inject` is omitted, injections are inferred from program structure (sensors, connectivity, recovery policies).

## Injection kinds

| Kind | Foundation |
|------|------------|
| `gps-failure` | Recovery planner + sensor faults |
| `camera-failure` | Health runtime faults |
| `lidar-failure` | Recovery planner + sensor faults |
| `connectivity-failure` | Connectivity declarations |
| `provider-failure` | Provider dispatch recovery |
| `package-failure` | Package reload recovery |
| `battery-failure` | Battery health checks |

## Verification after chaos

Each injection reports pass/fail for:

- **Recovery** — `simulate_failure_recovery` plan and execution outcomes
- **Health** — `evaluate_runtime_health` with injected fault signals
- **Readiness** — `evaluate_readiness` with runtime fault injection enabled
- **Safety** — no `Unsafe` recovery results; recovery actions safety-approved

Overall experiment **PASS** when every injection passes all four checks.

## Crate

`spanda-chaos` — composes `spanda-assurance`, `spanda-readiness`, and `spanda-capability`.

Showcase: `examples/showcase/self_healing/rover.sd` · smoke: `scripts/chaos_smoke.sh`

See [self-healing.md](./self-healing.md) · [resilience.md](./resilience.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
