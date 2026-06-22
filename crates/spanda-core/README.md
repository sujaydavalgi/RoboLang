# spanda-core

**Stable public facade** for the Spanda language implementation. External embedders and legacy integrations should depend on `spanda-core` and import `spanda_core::…`.

## What this crate is

After lean-core Phases 1–20, `spanda-core` is intentionally thin:

- **Re-exports** the compile/run pipeline from [`spanda-driver`](../spanda-driver/README.md)
- **Re-exports** AST, runtime, tooling, fleet, OTA, and security surfaces via `pub use` shims
- **Keeps** a small set of compatibility modules (deploy/fleet shims, `providers` facade)
- **Does not** contain the interpreter body, parser, transport adapters, or transport routing

First-party apps (`spanda-cli`, `spanda-node`, `spanda-wasm`, `spanda-dap`, `spanda-llvm`) import workspace crates directly and do **not** depend on `spanda-core`.

## Typical imports

```rust
use spanda_core::{check, run, verify_compatibility, RunOptions, SpandaError};
use spanda_core::ast::Program;
use spanda_core::providers::ProviderRegistry;
```

Equivalent direct paths (preferred for in-repo code):

```rust
use spanda_driver::{check, run, verify_compatibility, RunOptions};
use spanda_error::SpandaError;
use spanda_ast::nodes::Program;
use spanda_providers::ProviderRegistry;
```

## Removed modules (Phases 17 & 19)

These paths no longer exist on `spanda_core`:

| Removed | Use instead |
|---------|-------------|
| `spanda_core::transport_live` | `spanda_transport_routing::transport_live` |
| `spanda_core::transport_mqtt` | `spanda_transport_mqtt` or `spanda_transport_routing::live_bridges` |
| `spanda_core::transport_dds` | `spanda_transport_dds` or `spanda_transport_routing::live_bridges` |
| `spanda_core::transport_websocket` | `spanda_transport_websocket` or `spanda_transport_routing::live_bridges` |
| `spanda_core::transport` | `spanda_transport_routing::RoutingCommBus` |
| `spanda_core::transport_wire` | `spanda_transport::{encode_wire_value, decode_wire_value}` |
| `spanda_core::transport_security` | `spanda_transport::security` |
| `spanda_core::transport_rclrs` | `spanda_transport_ros2::rclrs` |

## Embedder feature bundles (Phase 20–21)

`spanda-core` defaults to `full` (`ota` + `fleet` + `certify` + `bridge`). For a slimmer dependency graph:

```toml
spanda-core = { path = "...", default-features = false }
# optional: features = ["ota"], ["fleet"], ["certify"], ["bridge"], or ["full"]
```

| Feature | Enables |
|---------|---------|
| `ota` | Deploy agents, bundles, remote rollout (`spanda-ota`, `spanda-deploy-http`) |
| `fleet` | Fleet mesh, orchestrator, swarm coordinator (`spanda-fleet`) |
| `certify` | Certification proof and runtime gate shims (`spanda-certify`) |
| `bridge` | Python/C++ FFI registry shims (`spanda-bridge`, `spanda-ffi`) |
| `full` | All optional surfaces (default) |

## Tests

Integration tests for fleet, OTA, providers, and certify live in the owning workspace crates. `spanda-core/tests/` keeps facade integration tests and `lean_core_shims.rs` guards.

## Related

- [Workspace crate index](../README.md)
- [lean-core-roadmap.md](../../docs/lean-core-roadmap.md)
