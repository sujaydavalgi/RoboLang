# Configuration Drift Detection

**Status:** Planned · **Phase:** Deploy, Operate · **Priority:** P1.1

Detect mismatch between **expected** (declared in `.sd` and audit baseline) and **actual** (agent, twin, or fleet telemetry).

## CLI

```bash
spanda drift rover.sd
spanda drift rover.sd --agent Rover@JetsonOrin
spanda drift rover.sd --json
```

## Comparison dimensions

| Dimension | Expected | Actual |
|-----------|----------|--------|
| Hardware | `deploy`, `requires_hardware` | Agent `/v1/status` hardware report |
| Packages | `spanda.toml` lock + registry | Installed package manifest on device |
| Firmware | Declared version/hash | Agent firmware report |
| Configuration | Approved `.sd` hash | Live program hash on agent |

## Output

`DriftReport` — per-dimension deltas, severity, suggested remediation.

## Foundation

Extends `spanda-readiness::twin` (`configuration_drift`, `capability_drift`, `health_drift`) to fleet agents and audit baselines.

## Crate

`spanda-drift` — composes readiness, hardware verify, and package metadata.

See [readiness.md](./readiness.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
