# Entity Registry

The **Entity Registry** is the unified inventory of all platform entities, built from resolved system configuration.

## Core types

```rust
pub struct EntityRegistry {
    pub entities: HashMap<String, EntityRecord>,
    pub relationships: Vec<EntityRelationship>,
}
```

## Building the registry

```rust
use spanda_config::{ConfigResolver, build_entity_registry};

let resolved = ConfigResolver::new().resolve_from_dir(project_root)?;
let registry = build_entity_registry(&resolved);
// or
let registry = resolved.entity_registry();
```

### Projection sources

| Source | Entity kinds emitted |
|--------|---------------------|
| `DeviceTree.fleet` | `fleet`, `robot`, `compute`, device nodes |
| `DeviceRegistry` | Pool devices (merged with tree) |
| `HumanRegistry` | `human`, `wearable`, spatial devices, `digital_twin` |
| Fleet hazard zones | `hazard` |
| Control center nodes | `control_center` |
| `LogicalPhysicalMap` | `connected_to`, `controls` edges |
| `ResolvedSystemConfig.packages` | `package` |
| `ResolvedSystemConfig.providers` | `provider` |
| Project manifest | `organization` |

Human collaboration entities are **not duplicated**: they project from `HumanRegistry` and fleet tree nodes into a single ID namespace.

## Registry operations

| Method | Description |
|--------|-------------|
| `get(id)` | Lookup by ID |
| `list()` | Sorted inventory |
| `query(&EntityQuery)` | Filtered search |
| `graph()` | Nodes + edges |
| `relationships_for(id)` | Incident edges |
| `impact_analysis(id)` | Downstream affected IDs |
| `dependency_chain(id)` | Provider dependency walk |

## REST API

| Endpoint | Behavior |
|----------|----------|
| `GET /v1/entities` | List summaries; supports query string filters |
| `GET /v1/entities/{id}` | Full `EntityRecord` |
| `POST /v1/entities/query` | JSON `EntityQuery` body |

### List filters (query string)

| Parameter | Filters |
|-----------|---------|
| `kind`, `entity_type` | Entity kind |
| `health`, `health_status` | Health enum |
| `readiness`, `readiness_status` | Readiness enum |
| `trust`, `trust_status` | Trust enum |
| `lifecycle`, `lifecycle_state` | Lifecycle enum |
| `tag`, `label` | Tags / labels |
| `provider`, `package` | Supply chain |
| `firmware`, `firmware_version` | Firmware match |
| `assigned_to` | Entities assigned to operator/robot |
| `depends_on` | Entities depending on target |
| `parent_id` | Direct children filter |
| `search`, `q` | Substring search on id/name |

## SDK

```rust
let entities = client.list_entities()?;
let detail = client.get_entity("rover-001")?;
let result = client.query_entities(&serde_json::json!({
    "kind": "robot",
    "health_status": "degraded"
}))?;
```

## Relationship to Device Registry

The **Device Registry** (`DeviceIdentityRecord`) remains the authoritative store for device identity, provisioning, quarantine, and network metadata. The entity registry **projects** devices into the unified model without replacing device pool workflows.

Use:

- **Device pool APIs** (`/v1/devices/*`) for lifecycle mutations
- **Entity APIs** (`/v1/entities/*`) for cross-domain inventory, graph, and queries

## Future: runtime overlay (Phase 2)

Active missions, incidents, and transient sessions will merge into an overlay registry at API time without persisting duplicate TOML records.
