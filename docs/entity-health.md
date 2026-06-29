# Entity Health

Health diagnostics for any entity are evaluated through `evaluate_entity_health`, composing device pool checks, fleet rollups, and optional program health checks.

**Implementation:** `crates/spanda-readiness/src/entity_health.rs`

## API

`GET /v1/entities/{id}/health` includes a `report` with diagnostics and metrics:

```json
{
  "health_status": "healthy",
  "report": {
    "diagnostics": [],
    "metrics": {
      "blocked_devices": 1,
      "total_devices": 3,
      "health_checks_passed": 0,
      "health_checks_failed": 0
    },
    "sources": ["entity_snapshot", "device_pool"]
  }
}
```

## CLI

```bash
spanda entity health rover-001 --config spanda.toml
spanda entity health rover-001 --program patrol.sd --json
```

## Dimensions

| Signal | Source |
|--------|--------|
| Snapshot health | `EntityRecord.health_status` |
| Device blockers | `evaluate_device_readiness` |
| Fleet members | Child robot/device rollup |
| Program checks | `evaluate_health_checks`, `evaluate_runtime_health` |

See also: [entity-sdk.md](./entity-sdk.md), [entity-readiness.md](./entity-readiness.md), [entity-model.md](./entity-model.md).
