# Entity Graph

The **Entity Graph** is the platform-wide directed graph of all Spanda entities and their relationships. It supports traversal, dependency analysis, impact analysis, traceability, and Control Center visualization.

## Structure

```rust
pub struct EntityGraph {
    pub nodes: Vec<EntityRecord>,
    pub edges: Vec<EntityRelationship>,
}
```

Built from [`EntityRegistry::graph()`](entity-registry.md) after `build_entity_registry()` projects configuration into entities.

## Example hierarchy

```text
Organization
    └── Fleet
            ├── Robot
            │     └── Compute
            │           └── Camera
            │                 └── (depends_on) Provider
            ├── Human
            │     └── (assigned_to) Wearable
            ├── Package
            └── Mission (runtime — Phase 2)
```

## Graph operations

| Operation | API / method | Use case |
|-----------|--------------|----------|
| Full graph | `GET /v1/entities/graph` | Dashboard visualization |
| Neighborhood | Control Center entity panel | Inspect one entity and peers |
| Impact analysis | `GET /v1/entities/{id}/relationships` → `impact` | “What breaks if this GPS fails?” |
| Dependency chain | same → `dependency_chain` | Provider/package dependency walk |
| Digital thread | `/v1/digital-thread/query` | Program-centric trace (complementary) |
| **Unified traceability** | `GET /v1/entities/traceability` | Entity + program + digital-thread chain |

Phase 3 (complete) aligns dependency-graph nodes with entity IDs and merges digital-thread links into the entity relationship store. Query params: `entity_id`, `device_id`, `capability`.

### Impact analysis

`EntityRegistry::impact_analysis(entity_id)` traverses outgoing `contains`, `depends_on`, and `consumes` edges (and reverse `depends_on`/`assigned_to`) to collect affected entity IDs.

### Dependency chain

`EntityRegistry::dependency_chain(entity_id)` walks `depends_on` and `consumes` edges from the entity outward — useful for “which provider does this device ultimately depend on?”

## Relationship to other graphs

| Graph | Scope | Crate |
|-------|-------|-------|
| **Entity graph** | All configured platform objects | `spanda-config` |
| **Dependency graph** | Program missions, capabilities, packages | `spanda-graph` |
| **Config graph** | TOML layer merge order | `spanda-config` layer |
| **Collaboration graph** | HRI sessions | `/v1/hri/collaboration` |

Phase 3 of the [migration plan](./entity-model.md#migration-plan) is complete: dependency nodes carry `entity_id` metadata and `/v1/entities/traceability` returns unified chains.

## Visualization

**Control Center:** Entities tab → Graph view renders an SVG neighborhood around the selected entity.

**SDK:**

```rust
let graph = client.entity_graph()?;
```

```typescript
const res = await fetch(`${base}/v1/entities/graph`);
const { graph } = await res.json();
```

## Edge data model

Each edge:

```json
{
  "from_id": "rover-001",
  "to_id": "gps-001",
  "kind": "contains",
  "label": "assigned"
}
```

See [entity-relationships.md](./entity-relationships.md) for the full relationship taxonomy.
