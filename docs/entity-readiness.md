# Entity Readiness

Operational readiness for any managed object flows through the Unified Entity Model via `evaluate_entity_readiness`.

**Implementation:** `crates/spanda-readiness/src/entity_readiness.rs`

## API

`GET /v1/entities/{id}/readiness` returns legacy snapshot fields plus an enriched `report` when Control Center has resolved configuration:

```json
{
  "readiness_status": "ready",
  "mission_ready": true,
  "report": {
    "entity_id": "rover-001",
    "mission_ready": false,
    "score": 65,
    "issues": [],
    "sources": ["entity_snapshot", "device_pool"]
  }
}
```

## CLI

```bash
spanda entity readiness rover-001 --config spanda.toml
spanda entity readiness rover-001 --program patrol.sd --dependencies --json
```

## Kind routing

| Entity kind | Engines |
|-------------|---------|
| Robot | Device pool, linked missions, optional program readiness |
| Fleet | Member rollup, per-robot checks |
| Mission | Participant graph |
| Human | Availability, collaboration readiness |
| Device | `evaluate_device_readiness` |
| Facility | Child entity rollup |

See also: [entity-sdk.md](./entity-sdk.md), [entity-verification.md](./entity-verification.md), [entity-model.md](./entity-model.md).
