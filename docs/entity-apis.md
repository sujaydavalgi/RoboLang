# Entity APIs

REST and gRPC reference for the **Unified Entity Model**. All routes are **additive** — existing `/v1/devices`, `/v1/robots`, `/v1/fleets`, and `/v1/programs/*` endpoints are unchanged.

**Implementation:** `crates/spanda-api/src/sdk_ops.rs`, `entity_mutations.rs`, `handlers.rs`  
**OpenAPI:** `GET /v1/openapi.json` (generated from handlers)  
**gRPC proto:** `crates/spanda-api/proto/spanda/v1/control_center.proto` — semver **1.0.3**, **83 RPCs**

## Authentication

| Class | Auth |
|-------|------|
| Read (`GET`, `POST /v1/entities/query`, `POST …/verify`) | None by default |
| Mutations (`register`, `tags`, `relationships`, `sync`) | `Authorization: Bearer <SPANDA_API_KEY>` |

Mutations are audited when RBAC is enabled. gRPC mutations use the same Bearer token in request metadata.

## Versioning

Every JSON response includes `"version": "v1"`. Pin gRPC clients with `GET /v1/version` → `grpc.proto_semver` and `grpc.rpc_count`.

## Endpoint summary

| Method | Path | gRPC RPC | Auth |
|--------|------|----------|------|
| `GET` | `/v1/entities` | `ListEntities` | — |
| `GET` | `/v1/entities/graph` | `GetEntityGraph` | — |
| `GET` | `/v1/entities/traceability` | `GetEntityTraceability` | — |
| `POST` | `/v1/entities/query` | `QueryEntities` | — |
| `GET` | `/v1/entities/{id}` | `GetEntity` | — |
| `GET` | `/v1/entities/{id}/relationships` | `GetEntityRelationships` | — |
| `GET` | `/v1/entities/{id}/health` | `GetEntityHealth` | — |
| `GET` | `/v1/entities/{id}/readiness` | `GetEntityReadiness` | — |
| `GET` | `/v1/entities/{id}/trust` | `GetEntityTrust` | — |
| `POST` | `/v1/entities/{id}/verify` | `VerifyEntity` | — |
| `POST` | `/v1/entities/register` | `RegisterEntity` | Bearer |
| `POST` | `/v1/entities/{id}/tags` | `TagEntity` | Bearer |
| `POST` | `/v1/entities/relationships` | `RelateEntities` | Bearer |
| `POST` | `/v1/entities/sync` | `SyncEntities` | Bearer |

## List and query

### `GET /v1/entities`

Optional query filters (aliases in parentheses):

| Parameter | Alias | Filters on |
|-----------|-------|------------|
| `kind` | `entity_type` | `EntityKind` string |
| `health_status` | `health` | Health enum |
| `readiness_status` | `readiness` | Readiness enum |
| `trust_status` | `trust` | Trust enum |
| `lifecycle_state` | `lifecycle` | Lifecycle enum |
| `tag`, `label`, `provider`, `package` | — | Metadata fields |
| `firmware_version` | `firmware` | Firmware string |
| `assigned_to`, `depends_on`, `participates_in`, `parent_id` | — | Graph fields |
| `search` | `q` | Free-text search |

```http
GET /v1/entities?kind=robot&health_status=degraded
```

Response:

```json
{
  "version": "v1",
  "count": 2,
  "entities": [
    {
      "id": "rover-001",
      "kind": "robot",
      "entity_type": "robot",
      "display_name": "Warehouse Rover 1",
      "health_status": "healthy",
      "readiness_status": "ready",
      "capabilities": ["navigate", "inspect"]
    }
  ]
}
```

Legacy field **`kind`** mirrors `entity_type` for SDK compatibility.

### `POST /v1/entities/query`

Same filter fields as JSON body. See [entity-query-language.md](./entity-query-language.md).

```json
{
  "kind": "robot",
  "firmware_version": "2.1.0",
  "health_status": "degraded"
}
```

Response wraps results in `result`:

```json
{
  "version": "v1",
  "result": {
    "entities": [],
    "count": 0,
    "query": { "kind": "robot" }
  }
}
```

## Graph and traceability

### `GET /v1/entities/graph`

Returns nodes, edges, and neighborhood index for Control Center visualization.

```json
{
  "version": "v1",
  "graph": {
    "nodes": [],
    "edges": []
  }
}
```

### `GET /v1/entities/traceability`

Query parameters: `entity_id`, `capability`, `device_id` (combinable).

```http
GET /v1/entities/traceability?entity_id=rover-001
```

Unifies entity registry edges with program graph and digital-thread context.

### `GET /v1/entities/{id}/relationships`

Returns directed edges, impact set, and `dependency_chain` for the entity.

## Entity detail

### `GET /v1/entities/{id}`

```json
{
  "version": "v1",
  "entity": {
    "id": "rover-001",
    "kind": "robot",
    "display_name": "Warehouse Rover 1",
    "parent_id": "warehouse-a",
    "capabilities": ["navigate"],
    "health_status": "healthy",
    "readiness_status": "ready",
    "trust_status": "trusted",
    "lifecycle_state": "active",
    "security": {},
    "metadata": {}
  }
}
```

Returns `404` when the id is not in the registry projection.

## Evaluation endpoints

Evaluation routes return **legacy snapshot fields** plus an enriched **`report`** when Control Center has loaded configuration (`--config`).

| Endpoint | Engine | Guide |
|----------|--------|-------|
| `GET …/health` | `evaluate_entity_health` | [entity-health.md](./entity-health.md) |
| `GET …/readiness` | `evaluate_entity_readiness` | [entity-readiness.md](./entity-readiness.md) |
| `GET …/trust` | `evaluate_entity_trust` | [entity-trust.md](./entity-trust.md) |
| `POST …/verify` | `verify_entity` | [entity-verification.md](./entity-verification.md) |

### `GET /v1/entities/{id}/health`

```json
{
  "version": "v1",
  "entity_id": "rover-001",
  "health_status": "healthy",
  "report": {
    "entity_id": "rover-001",
    "status": "healthy",
    "diagnostics": [],
    "sources": ["entity_snapshot", "device_pool"]
  }
}
```

### `GET /v1/entities/{id}/readiness`

```json
{
  "version": "v1",
  "entity_id": "rover-001",
  "readiness_status": "ready",
  "mission_ready": true,
  "report": {
    "entity_id": "rover-001",
    "mission_ready": true,
    "score": 85,
    "issues": []
  }
}
```

### `GET /v1/entities/{id}/trust`

```json
{
  "version": "v1",
  "entity_id": "rover-001",
  "trust_status": "trusted",
  "report": {
    "entity_id": "rover-001",
    "score": 90,
    "findings": []
  }
}
```

### `POST /v1/entities/{id}/verify`

Request body (optional):

```json
{
  "include_dependencies": true,
  "file": "path/to/program.sd"
}
```

When `file` is omitted, Control Center uses its `--program` if loaded.

```json
{
  "version": "v1",
  "verify": {
    "entity_id": "rover-001",
    "entity_type": "robot",
    "compatible": true,
    "findings": [],
    "capabilities": ["navigate"],
    "relationships_checked": 4,
    "dependencies_checked": 2,
    "health_status": "healthy",
    "readiness_status": "ready",
    "trust_status": "trusted"
  }
}
```

## Mutations (overlay)

Mutations update an in-memory **overlay** merged into `build_entity_registry()`. Call `POST /v1/entities/sync` to persist fragments to TOML.

### `POST /v1/entities/register`

```json
{
  "id": "calibration-bay-2",
  "entity_type": "calibration_station",
  "display_name": "Bay 2",
  "parent_id": "warehouse-a",
  "capabilities": ["calibrate"],
  "tags": ["production"],
  "metadata": { "zone": "north" },
  "persist": false
}
```

### `POST /v1/entities/{id}/tags`

```json
{
  "add": ["smoke-test"],
  "remove": ["deprecated"]
}
```

### `POST /v1/entities/relationships`

```json
{
  "from_id": "rover-001",
  "to_id": "gps-001",
  "kind": "depends_on",
  "label": "primary_gps"
}
```

Relationship `kind` values: `depends_on`, `participates_in`, `assigned_to`, `hosts`, `controls`, `monitors`, and overlay extensions. See [entity-relationships.md](./entity-relationships.md).

### `POST /v1/entities/sync`

```json
{}
```

Response includes `path`, `entities_written`, `relationships_written`.

Default overlay file: `.spanda/entity-overlays.json` (override with `SPANDA_ENTITY_OVERLAY_PATH`).

## gRPC

Start Control Center with `--grpc-bind`. All entity RPCs return `JsonResponse { json: "<same as REST body>" }`.

```bash
spanda control-center serve \
  --bind 127.0.0.1:8080 \
  --grpc-bind 127.0.0.1:50051 \
  --config spanda.toml
```

Bearer metadata for mutations:

```text
authorization: Bearer <SPANDA_API_KEY>
```

Pin clients:

```http
GET /v1/version
```

```json
{
  "grpc": {
    "proto_semver": "1.0.3",
    "rpc_count": 83
  }
}
```

## JSON-RPC gateway

`POST /v1/rpc` exposes **read-only** entity methods (list, get, graph, traceability, query, relationships, readiness). Mutations are **gRPC-only** (not on the JSON-RPC gateway). See [control-center-api.md](./control-center-api.md).

## CLI parity

| API | CLI |
|-----|-----|
| `GET /v1/entities` | `spanda entity list` |
| `GET /v1/entities/{id}` | `spanda entity inspect <id>` |
| `GET /v1/entities/graph` | `spanda entity graph` |
| `GET /v1/entities/{id}/relationships` | `spanda entity relationships <id>` |
| `GET /v1/entities/traceability` | `spanda entity traceability` |
| `POST /v1/entities/query` | `spanda entity query` / `search` |
| `GET …/readiness` | `spanda entity readiness <id>` |
| `GET …/health` | `spanda entity health <id>` |
| `GET …/trust` | `spanda entity trust <id>` |
| `POST …/verify` | `spanda entity verify <id>` |

CLI evaluation runs engines **locally** against `--config`; REST/gRPC evaluation uses the Control Center loaded config and program.

## Related docs

- [entity-overview.md](./entity-overview.md) — documentation map
- [entity-sdk.md](./entity-sdk.md) — Rust, TypeScript, Python clients
- [entity-model.md](./entity-model.md) — core concepts and taxonomy
- [control-center-api.md](./control-center-api.md) — full Control Center API
- [entity-integration-report.md](./entity-integration-report.md) — integration phase status
