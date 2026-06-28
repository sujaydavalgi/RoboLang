# Spanda SDK Overview

Official SDKs integrate with Spanda through the **Control Center API** (`spanda-api`) — a stable REST v1, gRPC, and WebSocket gateway. SDKs are **thin clients**: all business logic remains in Rust runtime crates; the CLI and SDKs call the same APIs.

## SDKs

| SDK | Package | Priority | Status |
|-----|---------|----------|--------|
| Rust | `spanda-sdk` crate | P0 | Experimental |
| Python | `pip install spanda-sdk` (`sdk/python/`) | P1 | Experimental |
| TypeScript | `@davalgi-spanda/sdk` (`sdk/typescript/`) | P2 | Experimental |

Legacy Python client: `packages/sdk-python` (Control Center helpers; use `sdk/python` for full SDK surface).

## Architecture

```
Application / Robot / Dashboard
        │
        ▼
   SDK (Rust / Python / TypeScript)
        │
        ▼
   spanda-api  ── REST /v1/*  ──► domain crates
        │         gRPC ControlCenter
        │         WS /v1/stream/telemetry
        ▼
   spanda-readiness, spanda-assurance, spanda-config, …
```

## Quick start

Start Control Center (serves API + optional UI):

```bash
spanda control-center serve --config examples/robotics --program examples/robotics/rover.sd
```

### Rust

```rust
use spanda_sdk::SpandaClient;

let client = SpandaClient::local();
let report = client.readiness("rover.sd")?;
println!("{}", report.score.unwrap_or(0));
```

### Python

```python
from spanda import SpandaClient

client = SpandaClient.local()
report = client.readiness("rover.sd")
print(report["report"]["score"])
```

### TypeScript

```typescript
import { SpandaClient } from "@davalgi-spanda/sdk";

const client = SpandaClient.local();
const report = await client.readiness("rover.sd");
console.log(report.score);
```

## Authentication

| Mode | Configuration |
|------|----------------|
| Local | Default `http://127.0.0.1:8080` — no auth for read-only program ops |
| API key | `SPANDA_API_KEY` or client `api_key` / Bearer token |
| Remote | `SPANDA_CONTROL_CENTER_URL` |
| mTLS / API keys (future) | Planned; do not hardcode secrets |

## Event stream

Real-time events (`health_changed`, `readiness_changed`, `mission_started`, `recovery_triggered`, …) are available via:

- **WebSocket:** `WS /v1/stream/telemetry`
- **gRPC:** streaming RPCs on `ControlCenter` service

See language-specific docs for stream helpers.

## Error model

All SDKs expose structured errors:

- `SpandaError` — base type
- `ValidationError`, `ReadinessError`, `VerificationError`
- `SecurityError`, `ConnectionError`, `PermissionError`

## Documentation

- [Rust SDK](sdk-rust.md)
- [Python SDK](sdk-python.md)
- [TypeScript SDK](sdk-typescript.md)
- [Publishing SDKs (PyPI / npm)](sdk-publishing.md)
- [Control Center API](control-center-api.md)

## Examples

| Language | Path |
|----------|------|
| Rust | `crates/spanda-sdk/examples/` |
| Python | `examples/sdk/python/` |
| TypeScript | `examples/sdk/typescript/` |

## Known limitations

- **Simulation / replay:** Pass `"execute": true` on `POST /v1/programs/simulation` to run the driver; replay supports `"deterministic": true` and `"playback": true`. Default remains inspect-only metadata.
- **Local file paths:** Program endpoints resolve paths relative to Control Center `--config` project root.
- **Pool vs program readiness:** `POST /v1/readiness/run` remains device-pool impact; use `POST /v1/programs/readiness` for CLI-equivalent program scoring.
