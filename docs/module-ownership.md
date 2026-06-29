# Module Ownership Matrix

Every Spanda module belongs to exactly one architectural layer and one ownership domain.

**Parent:** [platform-architecture.md](./platform-architecture.md) · **Rules:** [dependency-rules.md](./dependency-rules.md)

Source of truth: [`scripts/architecture-manifest.yaml`](../scripts/architecture-manifest.yaml)

---

## Ownership domains

| Domain | Scope |
|--------|-------|
| `compiler` | Lex, parse, type-check, SIR, LLVM |
| `runtime` | Interpreter, scheduler, comm, safety, HAL |
| `entity-config` | Entity model, cascading config, device tree |
| `hardware` | Hardware profiles and verification |
| `capability` | Capability registry and traceability |
| `packages` | Package manifest, registry, resolver |
| `providers` | Provider bootstrap and dispatch |
| `transport` | MQTT, ROS2, DDS, WebSocket adapters |
| `fleet` | Fleet mesh and OTA |
| `readiness` | Operational readiness engine |
| `assurance` | Mission assurance and evidence |
| `diagnosis` | Explainability, faults, graphs, diffs |
| `trust` | Trust scoring |
| `security` | Identity, tamper, threat modeling |
| `telemetry` | Telemetry store and replay substrate |
| `policy` | Operational policy engine |
| `control-center` | REST/gRPC API, ops, web UI |
| `cli` | Native CLI |
| `sdk` | Rust/TS/Python SDKs |
| `bindings` | Node, WASM, DAP |
| `tooling` | fmt, lint, LSP, VS Code |
| `codegen` | LLVM, native RT |
| `ffi` | Bridge and extern registry |
| `dx` | ADR generation, templates |
| `platform` | Public facade (`spanda-core`) |
| `connectivity` | Connectivity type catalogs |
| `security-audit` | Audit ledger |
| `verify` | Deploy certification |

---

## Rust crates — full matrix

### Foundation (layer 0)

| Crate | Owner | Responsibility |
|-------|-------|----------------|
| `spanda-audit` | security-audit | Audit records, provenance, ledger |
| `spanda-connectivity` | connectivity | Connectivity/positioning type catalogs |
| `spanda-deploy-http` | fleet | Minimal HTTP/1.1 for deploy agents |

### Compiler (layer 1)

| Crate | Owner | Responsibility |
|-------|-------|----------------|
| `spanda-ast` | compiler | AST nodes, foundations |
| `spanda-lexer` | compiler | Tokenization |
| `spanda-parser` | compiler | Parsing |
| `spanda-typecheck` | compiler | Units and type system |
| `spanda-error` | compiler | Error types |
| `spanda-sir` | compiler | Spanda IR |
| `spanda-modules` | compiler | Multi-file module loader |
| `spanda-regex-lang` | compiler | Regex compilation |
| `spanda-format` | tooling | Source formatter |
| `spanda-lint` | tooling | Static linter |
| `spanda-codegen` | tooling | Codegen metadata |
| `spanda-docs` | tooling | Reference generators |
| `spanda-llvm` | codegen | LLVM backend |
| `spanda-rt` | codegen | Native runtime C ABI |

### Language Runtime (layer 2)

| Crate | Owner | Responsibility |
|-------|-------|----------------|
| `spanda-runtime` | runtime | Scheduler, runtime state |
| `spanda-interpreter` | runtime | Interpreter and simulator |
| `spanda-runtime-host` | runtime | RuntimeHost wiring |
| `spanda-driver` | runtime | Compile → run orchestration |
| `spanda-comm` | runtime | Comm bus |
| `spanda-safety` | runtime | Safety monitor |
| `spanda-hal` | runtime | HAL simulation |
| `spanda-concurrency` | runtime | Tasks, channels |
| `spanda-debug` | runtime | Debugger |
| `spanda-ai` | runtime | AI model registry |
| `spanda-connectivity-runtime` | runtime | Connectivity runtime hooks |
| `spanda-bridge` | ffi | Python/C++ bridges |
| `spanda-ffi` | ffi | Extern registry |
| `spanda-lib-registry` | runtime | Sensor driver registry |

### Core Platform (layer 3)

| Crate | Owner | Responsibility |
|-------|-------|----------------|
| `spanda-config` | entity-config | Entity model, config, device tree |
| `spanda-hardware` | hardware | Hardware profile catalog |
| `spanda-capability` | capability | Capability registry |
| `spanda-certify` | verify | Deploy certification |
| `spanda-package` | packages | Package registry |
| `spanda-providers` | providers | Provider bootstrap |
| `spanda-transport` | transport | Transport traits |
| `spanda-transport-routing` | transport | RoutingCommBus |
| `spanda-transport-mqtt` | transport | MQTT backend |
| `spanda-transport-ros2` | transport | ROS 2 backend |
| `spanda-transport-dds` | transport | DDS backend |
| `spanda-transport-websocket` | transport | WebSocket backend |
| `spanda-ros2-rclrs-native` | transport | Native rclrs (optional) |
| `spanda-fleet` | fleet | Fleet coordination |
| `spanda-ota` | fleet | OTA rollout |
| `spanda-core` | platform | Public facade |

### Platform Services (layer 4)

| Crate | Owner | Responsibility |
|-------|-------|----------------|
| `spanda-readiness` | readiness | Operational readiness |
| `spanda-assurance` | assurance | Mission assurance |
| `spanda-explain` | diagnosis | Explainability reports |
| `spanda-runtime-faults` | diagnosis | Runtime fault detection |
| `spanda-decision` | diagnosis | Decision audit trail |
| `spanda-graph` | diagnosis | Dependency graph visualization |
| `spanda-diff` | diagnosis | Mission differencing |
| `spanda-contract` | assurance | Mission contracts |
| `spanda-score` | assurance | Scorecards |
| `spanda-compliance` | assurance | Compliance profiles |
| `spanda-chaos` | assurance | Chaos experiments |
| `spanda-estimate` | assurance | Resource estimation |
| `spanda-policy` | policy | Policy evaluation |
| `spanda-tamper` | security | Tamper analysis |
| `spanda-trust` | trust | Trust scoring |
| `spanda-spoofing` | security | Spoofing detection |
| `spanda-threat` | security | Threat modeling |
| `spanda-security` | security | Identity and secrets |
| `spanda-telemetry-store` | telemetry | Telemetry persistence |
| `spanda-ops` | control-center | Alerting and incidents |
| `spanda-adr` | dx | ADR generation |
| `spanda-generate` | dx | Template generation |

### Interfaces (layer 5)

| Crate / Package | Owner | Responsibility |
|-----------------|-------|----------------|
| `spanda` (`spanda-cli`) | cli | Native CLI |
| `spanda-api` | control-center | REST + gRPC API |
| `spanda-sdk` | sdk | Rust SDK |
| `spanda-node` | bindings | Node.js N-API |
| `spanda-wasm` | bindings | WebAssembly |
| `spanda-dap` | bindings | Debug adapter |
| `src/` (TS mirror) | runtime | TypeScript core mirror |
| `packages/lsp` | tooling | Language server |
| `packages/web` | control-center | Control Center UI |
| `packages/native` | bindings | N-API wrapper |
| `packages/control-center-desktop` | control-center | Tauri desktop shell |
| `sdk/typescript` | sdk | TypeScript SDK |
| `sdk/python` | sdk | Python SDK |
| `editor/vscode` | tooling | VS Code extension |

### Solution Blueprints (layer 6)

| Blueprint | Path | Owner |
|-----------|------|-------|
| Warehouse | `examples/end_to_end/warehouse_delivery/` | solutions |
| SAR | `examples/solutions/spatial-computing/search-and-rescue-ar/` | solutions |
| ADAS | `examples/solutions/adas/` | solutions |
| Healthcare | `examples/solutions/spatial-computing/wearable-health/` | solutions |
| Agriculture | `examples/solutions/agriculture/` | solutions |
| Environmental | `examples/solutions/environmental-monitoring/` | solutions |
| Maritime | `examples/solutions/maritime/` | solutions |
| Spatial computing | `examples/solutions/spatial-computing/` | solutions |

---

## Registry packages

Official `.sd` packages under `packages/registry/` are **not** workspace crates. They belong to the package ecosystem layer (Core Platform extension via provider traits). See [official-packages.md](./official-packages.md).

---

## Adding a new module

1. Choose layer and owner domain from this matrix.
2. Add entry to `scripts/architecture-manifest.yaml` → `rust_crates` or `typescript_packages`.
3. Regenerate `architecture-manifest.json`.
4. Ensure `validate_architecture.py` passes.
5. Update [crates/README.md](../crates/README.md) if adding a workspace crate.

---

## Orphan policy

Leaf crates at foundation/compiler layers and interface entry points may have no dependents. Optional crates (`spanda-ros2-rclrs-native`) are excluded from orphan warnings.

Crates with neither dependents nor dependencies outside optional set trigger a warning for review.
