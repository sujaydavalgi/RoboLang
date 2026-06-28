# Entity Query Language

Spanda provides a structured **Entity Query** for finding entities across the unified registry — robots, wearables, missions, packages, providers, and more.

## Query structure

```rust
pub struct EntityQuery {
    pub entity_type: Option<String>,
    pub kind: Option<String>,           // alias for entity_type
    pub health_status: Option<String>,
    pub readiness_status: Option<String>,
    pub trust_status: Option<String>,
    pub lifecycle_state: Option<String>,
    pub tag: Option<String>,
    pub label: Option<String>,
    pub provider: Option<String>,
    pub package: Option<String>,
    pub firmware_version: Option<String>,
    pub assigned_to: Option<String>,
    pub depends_on: Option<String>,
    pub parent_id: Option<String>,
    pub search: Option<String>,
}
```

All fields are optional; unspecified fields do not filter.

## Transport

### GET with query parameters

```http
GET /v1/entities?kind=robot&health_status=degraded
GET /v1/entities?firmware_version=2.1.0&kind=camera
GET /v1/entities?assigned_to=operator-001&kind=wearable
GET /v1/entities?depends_on=spanda-gps
GET /v1/entities?search=warehouse
```

Aliases: `health`, `readiness`, `trust`, `lifecycle`, `firmware`, `q`.

### POST with JSON body

```http
POST /v1/entities/query
Content-Type: application/json

{
  "kind": "device",
  "firmware_version": "2.1.0",
  "health_status": "degraded"
}
```

Response:

```json
{
  "version": "v1",
  "result": {
    "entities": [ ... ],
    "count": 3,
    "query": { ... }
  }
}
```

## Example queries

### Find every robot using camera firmware version X

```json
{
  "kind": "camera",
  "firmware_version": "2.1.0"
}
```

Then filter results whose `parent_id` chain reaches a robot, or query robots and inspect child devices via relationships API.

Combined approach:

```http
GET /v1/entities?kind=camera&firmware_version=2.1.0
GET /v1/entities/{camera_id}/relationships
```

### Find every wearable assigned to operator Y

```http
GET /v1/entities?kind=wearable&assigned_to=operator-001
```

### Find every mission affected by GPS outage

Phase 2 (runtime overlay): query missions with `depends_on=gps-001`. Today, use impact analysis:

```http
GET /v1/entities/gps-001/relationships
```

The `impact` array lists affected entity IDs including robots and assigned devices.

### Find every package used by Fleet A

```http
GET /v1/entities?kind=package
GET /v1/entities/fleet-a/relationships
```

Inspect `consumes` / `depends_on` edges from fleet members to packages.

### Find all entities with degraded health

```http
GET /v1/entities?health_status=degraded
```

### Find all entities that depend on Provider X

```http
GET /v1/entities?depends_on=spanda-mqtt
```

## SDK

```rust
use serde_json::json;

let result = client.query_entities(&json!({
    "health_status": "degraded",
    "kind": "device"
}))?;
```

```typescript
const res = await fetch(`${base}/v1/entities/query`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ depends_on: "spanda-gps" }),
});
```

## Enum values

Filters match lowercase snake_case enum strings:

| Field | Values |
|-------|--------|
| `health_status` | `healthy`, `warning`, `degraded`, `offline`, `critical`, `unknown` |
| `readiness_status` | `ready`, `not_ready`, `partial`, `unknown` |
| `trust_status` | `verified`, `trusted`, `untrusted`, `compromised`, `unknown` |
| `lifecycle_state` | `discovered`, `provisioned`, `verified`, `assigned`, `active`, `suspended`, `degraded`, `offline`, `retired`, `archived`, `unknown` |

Legacy strings (`unverified`, `available`, `failed`) map during entity projection — query using normalized enum values.

## Future extensions

- Graph pattern queries (Cypher-like) — **Research**
- Saved queries and alerts on query results — **Later**
- Full-text search index — **Later**
