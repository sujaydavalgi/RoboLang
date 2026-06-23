# Operational Readiness

Spanda's **Operational Readiness Engine** answers one question:

> Can this robot safely perform this mission right now?

It composes existing platform gates — hardware verification, capability verification, health checks, connectivity validation, safety rules, and mission requirements — into a single weighted **readiness score** and go/no-go report.

## Quick start

```bash
spanda readiness examples/showcase/readiness/rover.sd
spanda readiness examples/showcase/readiness/rover.sd --json
spanda readiness examples/showcase/readiness/rover.sd --markdown
spanda readiness examples/showcase/readiness/rover.sd --html
```

Example output:

```
Mission Ready: YES
Score: 92/100

Issues:
* LTE signal weak
* Camera calibration due in 5 days
* Battery below recommended threshold
```

## Readiness factors

| Factor | Source |
|--------|--------|
| Hardware | `spanda verify` / hardware profiles |
| Capabilities | Capability registry + minimum hardware |
| Health | `health_check` declarations |
| Connectivity | Connectivity policy + hardware |
| Safety | Minimum capabilities + kill switches |
| Battery | Mission duration vs battery budget |
| Storage / Compute | Resource budgets (when declared) |
| Packages / Providers | Traceability matrix |
| Mission Requirements | `mission { requires capabilities [...] }` |

## Types

- `ReadinessStatus` — Ready, Degraded, NotReady, Unknown
- `ReadinessReport` — full evaluation with score and issues
- `ReadinessScore` — weighted total and per-factor breakdown
- `ReadinessIssue` — severity, factor, message, suggested action
- `ReadinessPolicy` — minimum score threshold and factor weights

## Related commands

| Command | Purpose |
|---------|---------|
| `spanda readiness <file.sd>` | Unified readiness evaluation |
| `spanda verify mission <file.sd>` | Mission achievability check |
| `spanda fleet readiness <file.sd>` | Fleet-level readiness |
| `spanda twin readiness <file.sd>` | Physical vs digital twin drift |
| `spanda audit <file.sd>` | Autonomous safety auditor |

## Crate

Rust API: `spanda-readiness` (`evaluate_readiness`, `ReadinessReport`, …)

See also: [Mission Verification](mission-verification.md), [Fleet Readiness](fleet-readiness.md), [Safety Reporting](safety-reporting.md).
