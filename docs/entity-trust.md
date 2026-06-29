# Entity Trust

Trust evaluation for packages, devices, robots, and program-linked entities flows through `evaluate_entity_trust`.

**Implementation:** `crates/spanda-trust/src/entity_trust.rs`

## API

`GET /v1/entities/{id}/trust` returns snapshot security metadata plus a scored `report`:

```json
{
  "trust_status": "trusted",
  "security": { "certificates": [] },
  "report": {
    "score": 80,
    "passed": false,
    "categories": [
      { "name": "quarantine", "score": 0, "passed": false, "detail": "trust_not_verified" }
    ],
    "tamper_status": null
  }
}
```

## CLI

```bash
spanda entity trust rover-001 --config spanda.toml
spanda entity trust package-spanda-gps --program patrol.sd --json
```

## Kind routing

| Entity kind | Engines |
|-------------|---------|
| Package / provider | `evaluate_package_trust` |
| Device / sensor | Quarantine policy, device trust level |
| Robot | Contained device trust rollup |
| Program context | `evaluate_composite_trust` (tamper/integrity) |

See also: [entity-sdk.md](./entity-sdk.md), [entity-verification.md](./entity-verification.md), [entity-model.md](./entity-model.md).
