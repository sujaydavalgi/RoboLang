# Entity Relationships

Entities connect through typed, directed **relationship edges**. Relationships power graph traversal, impact analysis, Control Center navigation, and query filters.

## Relationship kinds

| Kind | Semantics | Example |
|------|-----------|---------|
| `owns` | Organizational ownership | Organization → Fleet |
| `contains` | Structural containment | Fleet → Robot → Compute → Device |
| `connected_to` | Logical/physical binding | Robot → Sensor (logical map) |
| `controls` | Actuator control | Robot → Gripper |
| `monitors` | Observation / mirroring | Digital twin → Robot; Hazard → Robot |
| `depends_on` | Runtime or supply dependency | Device → Provider |
| `assigned_to` | Assignment | Wearable → Human; Device → Robot |
| `communicates_with` | Protocol peer | Gateway ↔ Device |
| `backs_up` | Failover target | Device → Backup device |
| `replaces` | Replacement lineage | Retired device → Successor |
| `reports_to` | Command hierarchy | Operator → Supervisor |
| `belongs_to` | Group membership | Human → Team |
| `located_in` | Spatial containment | Robot → Zone |
| `secured_by` | Security policy binding | Device → Certificate authority |
| `managed_by` | Operational management | Fleet → Control center |
| `provides` | Service provision | Provider → Capability consumer |
| `consumes` | Service consumption | Device → Package |
| `participates_in` | Mission/session membership | Robot → Mission (Phase 2) |

Parse programmatically:

```rust
EntityRelationshipKind::parse("depends_on"); // Some(DependsOn)
```

## Edge structure

```json
{
  "from_id": "wearable-001",
  "to_id": "operator-001",
  "kind": "assigned_to",
  "label": null
}
```

Optional `label` disambiguates edge provenance (`"logical_map"`, `"assigned"`, `"mirrors"`, …).

## API

```http
GET /v1/entities/rover-001/relationships
```

Response:

```json
{
  "version": "v1",
  "entity_id": "rover-001",
  "relationships": [ ... ],
  "impact": ["gps-001", "lidar-front"],
  "dependency_chain": ["spanda-gps"]
}
```

## Automatic edge emission

| Configuration | Edges created |
|---------------|---------------|
| Device tree hierarchy | `contains` (fleet→robot→compute→device) |
| `assigned_robot` on device | `contains` + `assigned_to` |
| Wearable `human_id` | `assigned_to` |
| Hazard `linked_robots` | `monitors` |
| Twin `entity_id` | `monitors` (mirrors) |
| Logical map | `connected_to`, `controls` |
| Device `provider` | `depends_on` |
| Organization + fleet | `owns` |

## Impact and dependency analysis

**Impact analysis** answers: “If entity X fails or goes offline, what else is affected?”

Traverses:

- Outgoing `contains`, `depends_on`, `consumes`
- Incoming `depends_on`, `consumes`, `assigned_to`, `belongs_to` from dependents

**Dependency chain** walks outgoing `depends_on` / `consumes` edges to root providers or packages.

## Query by relationship

```http
GET /v1/entities?assigned_to=operator-001
GET /v1/entities?depends_on=spanda-gps
```

Or POST body:

```json
{
  "assigned_to": "operator-001",
  "kind": "wearable"
}
```

## Extending relationships

1. Add variant to `EntityRelationshipKind` if a new semantic is platform-wide
2. Emit edges in `build_entity_registry` projection
3. Update impact/dependency traversals if the kind participates in failure propagation
4. Document in this file and [entity-model.md](./entity-model.md)
