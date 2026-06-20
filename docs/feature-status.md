# Feature Status

Honest snapshot of Spanda capabilities. **Stubbed** = syntax or API exists without real external integration.

## Language core

| Feature | Status | Notes |
|---------|--------|-------|
| Lexer / parser / AST | Implemented | Rust authoritative; TS mirror includes hardware/deploy |
| Type checker + units | Implemented | Physical unit algebra enforced |
| modules / imports | Implemented | Rust + TS `ModuleRegistry`; project vendor via `spanda install` |
| structs / enums / traits | Implemented | Enum associated data not yet supported |
| generics | Partially implemented | Module function type params only |
| match / Result / Option | Implemented | |
| async / await | Implemented | Cooperative single-threaded |
| spawn / select / channels | Partially implemented | Cooperative concurrency |
| test blocks | Implemented | Rust runtime |
| `extern fn` / FFI | Partially implemented | `extern python` + `extern cpp` subprocess bridges; native stubs |
| Spanda IR (SIR) | Partially implemented | `spanda ir`; imports, behaviors, extern bridge kinds |
| Codegen / LLVM | Partially implemented | SIR-aware templates + `spanda llvm-ir` (signatures only) |

## Autonomous systems

| Feature | Status | Notes |
|---------|--------|-------|
| robot / sensor / actuator | Implemented | |
| agent / goal / task / skill | Implemented | Mock AI |
| ActionProposal → SafeAction | Implemented | Compile + runtime |
| safety zones / emergency stop | Implemented | |
| deterministic scheduler | Implemented | `task every Nms` |
| state machine / events | Implemented | |
| twin / replay | Implemented | Replay buffer; **`twin sync`** telemetry/replay wired |
| observe / fusion | Implemented | |
| verify { } behavioral assertions | Implemented | |
| hardware / deploy | Implemented | Rust verify CLI; TS parse + deploy validation |

## Tooling

| Feature | Status | Notes |
|---------|--------|-------|
| Native CLI (full) | Implemented | check, verify, run, fmt, lint, doc, package |
| TypeScript CLI | Implemented | Delegates to Rust when built; TS fallback for check/run/sim |
| Formatter / linter / docgen | Implemented | Rust |
| LSP | Partially implemented | Symbols include hardware/deploy |
| DAP debugger | Partially implemented | Breakpoints + continue/step commands |
| N-API | Partially implemented | check, run, verify, sir, fmt |
| WASM | Partially implemented | check, run, verify, sir, fmt |

## Ecosystem / FFI

| Feature | Status | Notes |
|---------|--------|-------|
| python.* / cpp.* imports | Partially implemented | Type-check; `extern python`/`extern cpp` subprocess bridges |
| ROS2 adapter | Stubbed | Log-only stub |
| Transport adapters | Partially implemented | In-memory + log stubs; ROS2/MQTT via Python bridge handlers |
| Package manager | Partially implemented | spanda.toml, lockfile, git vendor, local registry packages |
| LLVM / native codegen | Stubbed | See compiler-backend-roadmap.md |

See also [README.md](../README.md), [ffi-and-ecosystem.md](./ffi-and-ecosystem.md), [compiler-backend-roadmap.md](./compiler-backend-roadmap.md).
