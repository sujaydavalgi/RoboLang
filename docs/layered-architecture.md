# Layered Architecture

How Spanda Platform v2.0 layers stack, what each layer owns, and how components map to workspace crates.

**Parent:** [platform-architecture.md](./platform-architecture.md)

---

## Layer stack

```
                    Spanda Platform
──────────────────────────────────────────────
Solution Blueprints          (layer 6)
  Warehouse, SAR, ADAS, Healthcare, …
──────────────────────────────────────────────
Interfaces                   (layer 5)
  Control Center, SDKs, REST, gRPC, CLI, VS Code
──────────────────────────────────────────────
Platform Services            (layer 4)
  Readiness, Assurance, Diagnosis, Recovery, Trust,
  Health, Security, Telemetry, Simulation, Replay, Policy
──────────────────────────────────────────────
Core Platform                (layer 3)
  Entity Model, Device Registry, Capabilities, Hardware,
  Mission Model, Config, Provider/Package Registry, Transport, Fleet, OTA
──────────────────────────────────────────────
Language Runtime             (layer 2)
  Scheduler, Triggers, Interpreter, Execution Engine,
  Concurrency, Comm, Safety, HAL, AI, Driver
──────────────────────────────────────────────
Compiler                     (layer 1)
  Lexer, Parser, AST, Type Checker, SIR, LLVM, Tooling
──────────────────────────────────────────────
Foundation                   (layer 0)
  Audit, Connectivity types, Deploy HTTP, Diagnostics
```

---

## Dependency direction

```mermaid
flowchart LR
  L6[Blueprints] --> L5[Interfaces]
  L5 --> L4[Platform Services]
  L4 --> L3[Core Platform]
  L3 --> L2[Language Runtime]
  L2 --> L1[Compiler]
  L1 --> L0[Foundation]
```

**Rules:**

1. A module may depend on **strictly lower** layers (e.g. platform services → core platform → runtime).
2. **Same-layer** dependencies are allowed for horizontal composition (e.g. assurance → readiness).
3. **Upward** dependencies (lower layer → higher layer) are forbidden unless waived and tracked.
4. **No new circular strongly connected components** without architecture review.

See [dependency-rules.md](./dependency-rules.md).

---

## Layer 0 — Foundation

Shared primitives with minimal Spanda semantics.

| Module | Path | Role |
|--------|------|------|
| `spanda-audit` | `crates/spanda-audit` | Audit records, provenance, ledger |
| `spanda-connectivity` | `crates/spanda-connectivity` | GPS/Wi-Fi/BLE/cellular type catalogs |
| `spanda-deploy-http` | `crates/spanda-deploy-http` | Minimal HTTP/1.1 for deploy agents |

---

## Layer 1 — Compiler

Transforms `.sd` source into typed IR and diagnostics.

| Module | Path | Role |
|--------|------|------|
| `spanda-ast` | `crates/spanda-ast` | AST nodes, foundations |
| `spanda-lexer` | `crates/spanda-lexer` | Tokenization |
| `spanda-parser` | `crates/spanda-parser` | Parsing |
| `spanda-typecheck` | `crates/spanda-typecheck` | Units and type system |
| `spanda-error` | `crates/spanda-error` | Error types |
| `spanda-sir` | `crates/spanda-sir` | Spanda IR |
| `spanda-modules` | `crates/spanda-modules` | Multi-file loader |
| `spanda-regex-lang` | `crates/spanda-regex-lang` | Regex compilation |
| `spanda-format` | `crates/spanda-format` | Formatter |
| `spanda-lint` | `crates/spanda-lint` | Linter |
| `spanda-codegen` | `crates/spanda-codegen` | Codegen metadata |
| `spanda-docs` | `crates/spanda-docs` | Reference generators |
| `spanda-llvm` | `crates/spanda-llvm` | LLVM backend (experimental) |
| `spanda-rt` | `crates/spanda-rt` | Native runtime ABI (experimental) |

TypeScript mirror: `src/lexer`, `src/parser`, `src/ast`, `src/types`, …

---

## Layer 2 — Language Runtime

Executes programs — scheduler, triggers, interpreter, comm, safety.

| Module | Path | Role |
|--------|------|------|
| `spanda-runtime` | `crates/spanda-runtime` | Scheduler, runtime state |
| `spanda-interpreter` | `crates/spanda-interpreter` | Tree-walking interpreter, sim |
| `spanda-runtime-host` | `crates/spanda-runtime-host` | RuntimeHost wiring |
| `spanda-driver` | `crates/spanda-driver` | Compile → run orchestration |
| `spanda-comm` | `crates/spanda-comm` | Comm bus |
| `spanda-safety` | `crates/spanda-safety` | Safety monitor |
| `spanda-hal` | `crates/spanda-hal` | HAL simulation |
| `spanda-concurrency` | `crates/spanda-concurrency` | Tasks, channels |
| `spanda-debug` | `crates/spanda-debug` | Debugger |
| `spanda-ai` | `crates/spanda-ai` | AI registry |
| `spanda-connectivity-runtime` | `crates/spanda-connectivity-runtime` | Connectivity runtime hooks |
| `spanda-bridge` / `spanda-ffi` | `crates/spanda-bridge`, `spanda-ffi` | Extern bridges |
| `spanda-lib-registry` | `crates/spanda-lib-registry` | Sensor driver registry |

---

## Layer 3 — Core Platform

Canonical platform infrastructure — entity model, registries, transport.

| Module | Path | Role |
|--------|------|------|
| `spanda-config` | `crates/spanda-config` | **Entity model**, cascading config, device tree |
| `spanda-hardware` | `crates/spanda-hardware` | Hardware profiles |
| `spanda-capability` | `crates/spanda-capability` | Capability registry |
| `spanda-certify` | `crates/spanda-certify` | Deploy certification |
| `spanda-package` | `crates/spanda-package` | Package registry |
| `spanda-providers` | `crates/spanda-providers` | Provider bootstrap |
| `spanda-transport*` | `crates/spanda-transport-*` | Transport adapters and routing |
| `spanda-fleet` | `crates/spanda-fleet` | Fleet coordination |
| `spanda-ota` | `crates/spanda-ota` | OTA rollout |
| `spanda-core` | `crates/spanda-core` | Public facade for embedders |

---

## Layer 4 — Platform Services

Reusable operational services consumed by interfaces and blueprints.

| Service area | Crates |
|--------------|--------|
| Readiness | `spanda-readiness` |
| Assurance | `spanda-assurance`, `spanda-contract`, `spanda-score`, `spanda-compliance`, `spanda-chaos`, `spanda-estimate` |
| Diagnosis | `spanda-explain`, `spanda-runtime-faults`, `spanda-decision`, `spanda-graph`, `spanda-diff` |
| Trust | `spanda-trust` |
| Security | `spanda-security`, `spanda-tamper`, `spanda-spoofing`, `spanda-threat` |
| Telemetry / Replay | `spanda-telemetry-store` |
| Policy | `spanda-policy` |
| Operations | `spanda-ops` |
| DX | `spanda-adr`, `spanda-generate` |

Details: [platform-services.md](./platform-services.md).

---

## Layer 5 — Interfaces

Entry points for operators, developers, and integrations.

| Module | Path | Role |
|--------|------|------|
| `spanda` (CLI) | `crates/spanda-cli` | Native CLI |
| `spanda-api` | `crates/spanda-api` | REST + gRPC Control Center |
| `spanda-sdk` | `crates/spanda-sdk` | Rust SDK |
| `spanda-node` / `spanda-wasm` / `spanda-dap` | `crates/spanda-*` | Bindings |
| `@davalgi-spanda/web` | `packages/web` | Control Center UI |
| `@spanda/lsp` | `packages/lsp` | Language server |
| `@davalgi-spanda/sdk` | `sdk/typescript` | TypeScript SDK |
| `spanda_sdk` | `sdk/python` | Python SDK |
| VS Code extension | `editor/vscode` | IDE integration |

First-party apps depend on workspace crates directly, **not** on `spanda-core`. External embedders use `spanda-core` as facade.

---

## Layer 6 — Solution Blueprints

Industry compositions under `examples/solutions/`:

| Blueprint | Path |
|-----------|------|
| Warehouse | `examples/end_to_end/warehouse_delivery/` |
| SAR | `examples/solutions/spatial-computing/search-and-rescue-ar/` |
| ADAS | `examples/solutions/adas/` |
| Healthcare | `examples/solutions/spatial-computing/wearable-health/` |
| Agriculture | `examples/solutions/agriculture/` |
| Environmental | `examples/solutions/environmental-monitoring/` |
| Maritime | `examples/solutions/maritime/` |
| Spatial computing | `examples/solutions/spatial-computing/` |

Blueprints import platform packages and call CLI/API surfaces — they do not add crates to the workspace.

---

## Rationale

| Decision | Why |
|----------|-----|
| Separate compiler from runtime | Embedders can type-check without executing; WASM/LLVM paths stay optional |
| Core platform owns entity model | Single canonical model prevents drift across REST, CLI, and runtime |
| Platform services as distinct layer | Readiness/trust/telemetry reusable across blueprints without forking |
| Interfaces at top of code layers | CLI/API/SDK aggregate services; blueprints sit above as compositions |
| Package-first extensions | Domain logic (ROS2, MQTT, vision) stays in registry packages, not core |
