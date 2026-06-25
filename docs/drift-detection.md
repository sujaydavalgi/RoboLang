# Configuration Drift Detection

**Status:** Experimental · **Phase:** Deploy, Operate · **Priority:** P1.1

Detect mismatch between **expected** (approved baseline configuration) and **actual** (live resolved configuration), plus optional program-to-config mapping alignment.

## CLI

```bash
# Compare approved baseline against live project config
spanda drift --baseline configs/approved/ --config spanda.toml

# Same via config subcommand
spanda config drift --baseline configs/approved/ --config spanda.toml

# Include program mapping checks against live config
spanda drift --baseline configs/approved/ --config spanda.toml rover.sd

# Readiness with baseline drift gates
spanda readiness rover.sd --config spanda.toml --baseline configs/approved/

# JSON output
spanda drift --baseline configs/approved/ --config spanda.toml --json
```

## Comparison dimensions

| Dimension | Baseline | Current |
|-----------|----------|---------|
| Configuration | Merged TOML keys | Live merged TOML |
| Fleet | `fleet.id`, robot ids | Live fleet tree |
| Device | `DeviceRegistry` identity fields | Live device records |
| Provider / Package | Declared providers and packages | Live manifests |
| Mapping | Logical-to-physical sensors/actuators | Live mapping |
| Program | — | `.sd` sensors/actuators vs live map (when file provided) |

## Output

`ConfigDriftReport` — structured findings with `dimension`, `severity`, `message`, and optional `path`. Medium-or-higher severity fails the check (exit code 1).

## Foundation

Implemented in `spanda-config::drift` (semantic comparison on `ResolvedSystemConfig`). Readiness integrates drift via `--baseline`. Agent/firmware/hardware drift (live agent `/v1/status`) remains planned.

## Related

[configuration.md](./configuration.md) · [readiness.md](./readiness.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md)
