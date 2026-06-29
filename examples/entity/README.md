# Entity Model Examples

Programs and workflows demonstrating the Unified Entity Model across verification, graph traversal, readiness, trust, and traceability.

## Traceability chain

```text
Operator
    ↓
Mission (PatrolMission)
    ↓
Robot (rover-001)
    ↓
Camera / Lidar / GPS (devices)
    ↓
Firmware / Provider (spanda-gps, spanda-lidar, spanda-canbus)
    ↓
Package
```

## Programs

| File | Demonstrates |
|------|----------------|
| [entity_verify.sd](./entity_verify.sd) | Unified `spanda entity verify` |
| [entity_graph.sd](./entity_graph.sd) | Fleet graph and neighborhood traversal |
| [entity_query.sd](./entity_query.sd) | Multi-kind inventory query filters |
| [entity_relationships.sd](./entity_relationships.sd) | Mission ↔ robot ↔ provider edges |
| [entity_health.sd](./entity_health.sd) | Health checks and `evaluate_entity_health` |
| [entity_readiness.sd](./entity_readiness.sd) | Mission readiness and capability gates |
| [entity_trust.sd](./entity_trust.sd) | Package and platform trust evaluation |
| [entity_traceability.sd](./entity_traceability.sd) | End-to-end digital thread chain |

## Fixture config

Use the warehouse fixture for CLI demos:

```bash
CONFIG=crates/spanda-config/tests/fixtures/warehouse/spanda.toml
```

## CLI workflows

```bash
CONFIG=crates/spanda-config/tests/fixtures/warehouse/spanda.toml

# Inventory and graph
spanda entity list --config "$CONFIG"
spanda entity graph --config "$CONFIG" --json
spanda entity relationships rover-001 --config "$CONFIG"

# Evaluation engines
spanda entity verify rover-001 --program examples/entity/entity_verify.sd --config "$CONFIG"
spanda entity readiness rover-001 --program examples/entity/entity_readiness.sd --config "$CONFIG"
spanda entity health rover-001 --program examples/entity/entity_health.sd --config "$CONFIG"
spanda entity trust rover-001 --program examples/entity/entity_trust.sd --config "$CONFIG"

# Query and traceability
spanda entity query --kind robot --config "$CONFIG"
spanda entity traceability --entity-id rover-001 --config "$CONFIG"
```

## REST API

```bash
curl -s http://127.0.0.1:8080/v1/entities/rover-001/verify \
  -H 'Content-Type: application/json' \
  -d '{"include_dependencies":true,"file":"examples/entity/entity_verify.sd"}'
```

## SDK

```typescript
const report = await client.verifyEntity("rover-001", {
  includeDependencies: true,
  file: "examples/entity/entity_verify.sd",
});
```

## Related docs

- [entity-overview.md](../../docs/entity-overview.md)
- [entity-apis.md](../../docs/entity-apis.md)
- [entity-sdk.md](../../docs/entity-sdk.md)
- [entity-model.md](../../docs/entity-model.md)
