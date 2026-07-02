# Twin Cloud SaaS

**Status:** Experimental · **Package:** `spanda-twin-cloud` · **Horizon:** LATER

Hosted mission twin snapshot registry for field fleets — push digital mission twin state from edge Control Center or CLI, pull latest snapshots for ops dashboards and fleet analytics.

## Architecture

```text
Edge (robot / field laptop)                Twin Cloud SaaS
spanda twin cloud push patrol.sd    -->    Control Center /v1/twins/*
spanda twin mission (local)                In-memory snapshot store (dev)
GET /v1/analytics/mission-twin             GET /v1/twins/{id}
```

Production deployments point `SPANDA_TWIN_CLOUD_URL` at a hosted Control Center or dedicated twin-cloud service. The open-source Control Center embeds the **Twin Cloud backend** for development and field pilots. Snapshots persist to `.spanda/control-center-twins.json` (override with `SPANDA_CONTROL_CENTER_STATE_DIR`).

## Environment

| Variable | Purpose |
|----------|---------|
| `SPANDA_TWIN_CLOUD_URL` | Base URL (falls back to `SPANDA_CONTROL_CENTER_URL`) |
| `SPANDA_TWIN_CLOUD_API_KEY` | Bearer token (falls back to `SPANDA_API_KEY`) |
| `SPANDA_TWIN_CLOUD_TENANT` | Tenant id (defaults to `SPANDA_TENANT_ID` or `default`) |

Legacy replay upload via `SPANDA_CLOUD_UPLOAD_URL` remains for provider `cloud.upload` — Twin Cloud uses structured mission twin snapshots instead.

## CLI

```bash
export SPANDA_TWIN_CLOUD_URL=http://127.0.0.1:8080

spanda twin cloud push examples/showcase/mission_twin/patrol.sd
spanda twin cloud list
spanda twin cloud pull patrol --out patrol-twin.json
```

## REST API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/twins` | List twin summaries for tenant |
| `GET` | `/v1/twins/{id}` | Latest snapshot |
| `POST` | `/v1/twins/{id}/snapshots` | Push snapshot JSON |
| `POST` | `/v1/twins/sync` | Evaluate + store twin for loaded program |

## Crate

`crates/spanda-twin-cloud` — HTTP client, snapshot envelope, in-memory store.

## Registry package

`packages/registry/spanda-twin-cloud` — import surface for Spanda programs (`import twin.cloud`).

## SDK (0.5.3)

Rust (`spanda-sdk`), Python (`spanda_sdk`), and TypeScript (`@davalgi/spanda-sdk`) expose:

- `list_twins` / `listTwins`
- `get_twin` / `getTwin`
- `sync_twin` / `syncTwin`
- `push_twin_snapshot` / `pushTwinSnapshot`

## Tests

- `cargo test -p spanda-twin-cloud`
- `cargo test -p spanda-api twin_cloud`
- `./scripts/twin_cloud_saas_smoke.sh`

See [digital-mission-twin.md](./digital-mission-twin.md) · [replay.md](./replay.md).
