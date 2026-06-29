# Entity Verification

All Spanda verification engines route through the **Unified Entity Model**. Instead of separate `verify(robot)`, `verify(device)`, and `verify(fleet)` code paths at the API layer, callers use a single entry point:

```text
verify_entity(entity_id) → EntityVerifyReport
```

Domain-specific engines (hardware, mission, fleet, device pool, quarantine, config validation) are invoked based on the entity kind and optional program context.

## Architecture

```text
EntityRegistry
      │
      ▼
verify_entity(id, registry, config, options)
      │
      ├── Robot / Drone / Vehicle → device pool + quarantine + hardware + mission
      ├── Fleet / Swarm         → member graph + fleet verify + per-robot checks
      ├── Mission               → mission verify + participant graph
      ├── Human / Team          → human registry (availability, certifications)
      ├── Device / Sensor / …   → device pool + quarantine
      ├── Package / Provider    → manifest and provider registry
      ├── Facility / Zone       → structural child graph
      └── All kinds             → health, readiness, trust snapshot + relationship integrity
```

**Canonical implementation:** `crates/spanda-readiness/src/entity_verify.rs`

## API

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/entities/{id}/verify` | Unified verification report |

### Request body (optional)

```json
{
  "include_dependencies": true,
  "file": "path/to/program.sd"
}
```

When `file` is omitted, the Control Center uses its loaded `--program` if present.

### Response

```json
{
  "version": "v1",
  "verify": {
    "entity_id": "rover-001",
    "entity_type": "robot",
    "compatible": true,
    "findings": [],
    "capabilities": ["navigate", "inspect"],
    "relationships_checked": 4,
    "dependencies_checked": 2,
    "health_status": "healthy",
    "readiness_status": "ready",
    "trust_status": "trusted"
  }
}
```

Existing program verification routes remain unchanged:

| Route | CLI parity |
|-------|------------|
| `POST /v1/programs/verify/hardware` | `spanda verify` |
| `POST /v1/programs/verify/capabilities` | `spanda verify --capabilities` |
| `POST /v1/programs/verify/mission` | `spanda verify mission` |

## CLI

```bash
spanda entity verify rover-001 --config spanda.toml
spanda entity verify rover-001 --program mission.sd --dependencies --json
spanda entity verify gps-001 --config spanda.toml
spanda entity verify mission-alpha --program patrol.sd
```

Exit code is non-zero when `compatible` is false.

## SDK

Full method reference: [entity-sdk.md](./entity-sdk.md).

| SDK | Method |
|-----|--------|
| Rust | `client.entity_verify(id, body)` |
| TypeScript | `client.verifyEntity(id, { includeDependencies, file })` |
| Python | `client.entity_verify(id, include_dependencies=..., file=...)` |

## What entity verification checks

| Dimension | Source engines |
|-----------|----------------|
| Capabilities | `EntityRecord.capabilities` |
| Relationships | `EntityRegistry` edge integrity |
| Health / readiness / trust | Entity snapshot + device pool |
| Dependencies | `dependency_chain` traversal |
| Hardware | `verify_with_system_config` (when program provided) |
| Mission | `verify_mission` (robot/mission kinds) |
| Fleet | `verify_fleet` (fleet kind + program) |
| Device pool | `evaluate_device_readiness`, `evaluate_quarantine_policy` |
| Config | `ResolvedSystemConfig.validation` findings referencing entity id |

## Backward compatibility

- `spanda verify`, `spanda verify-fleet`, `spanda device *`, and `/v1/programs/verify/*` are unchanged.
- Entity verification is **additive** — it composes existing engines rather than replacing them.
- Legacy REST routes (`/v1/devices`, `/v1/robots`, `/v1/fleets`) continue to work; entity verify is the unified cross-kind path.

## Related docs

- [entity-model.md](./entity-model.md) — unified model overview
- [entity-registry.md](./entity-registry.md) — registry projection
- [entity-graph.md](./entity-graph.md) — relationships and dependencies
- [entity-integration-report.md](./entity-integration-report.md) — Phase 2 integration report
