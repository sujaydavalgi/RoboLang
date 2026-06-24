# Spanda Roadmap

Version plan organized by **platform area**. Tiers: **Stable** (CI-backed, documented), **Experimental** (usable with caveats), **Future** (planned, not shipped).

Current release line: **v0.4.0**.

Platform overview: [platform-overview.md](./platform-overview.md) · Feature truth table: [feature-status.md](./feature-status.md)

---

## Platform areas at a glance

| Area | Current focus (v0.4) | Next |
|------|----------------------|------|
| [Language](#language) | Stable core; typed handler I/O | Generics polish, self-hosting subset (future) |
| [Runtime](#runtime) | Interpreter LTS; certify gate | Native codegen golden paths |
| [Verification](#verification) | `spanda verify`, capability matrices | 5+ production hardware profiles (v1.0) |
| [Safety](#safety) | ActionProposal → SafeAction stable | Stricter certify / ISO metadata workflows |
| [Simulation](#simulation) | `spanda sim`, twins, fault injection | Deeper package bridges (Gazebo/Webots scaffolds) |
| [Health](#health) | health_check, fleet require | Swarm quorum hardening |
| [Fleet](#fleet) | In-process + HTTP agents | Distributed orchestration polish |
| [Packages](#packages) | 29 registry scaffolds, publish mirror | Curated remote registry growth |
| [Tooling](#tooling) | CLI, demos, CI golden paths | VS Code Marketplace, LSP polish |
| [Mission assurance](#mission-assurance) | Lean-core analysis + CLI | Package-backed ML anomaly backends |

---

## Mission assurance

**NASA-style autonomous operations** — knowledge models, state estimation, anomaly detection, prognostics, mitigation, resilience, assurance cases.

| Item | Status |
|------|--------|
| `spanda-assurance` crate (static analysis) | **Stable** |
| Language declarations (`knowledge_model`, `state_estimator`, `anomaly_detector`, …) | **Stable** |
| CLI (`assure`, `anomaly scan`, `state estimate`, `prognostics`, `mission verify`, `resilience check`, `mitigation plan`) | **Stable** |
| Runtime `state_estimator` → weighted fusion bindings | **Experimental** |
| Learned anomaly backends (`learned backend`, `spanda-anomaly`) | **Experimental** — runtime `scan_learned` + EMA volatility + optional ONNX (`SPANDA_ANOMALY_ONNX_MODEL_PATH`) |
| Weighted fusion package (`spanda-fusion`, `assurance.fusion`) | **Experimental** — provider dispatch for fusion weights |
| Full ML inference (custom ONNX architectures) | **Future** — beyond 2-feature anomaly scaffold |

See [mission-assurance.md](./mission-assurance.md), [state-estimation.md](./state-estimation.md).

---

## Language

**Spanda Language (`.sd`)** — syntax, types, robot primitives, units, safety types.

| Item | Status |
|------|--------|
| Lexer, parser, AST, type checker | **Stable** |
| Physical units, `module`/`import`, structs/enums/traits | **Stable** |
| Robot primitives (`robot`, `sensor`, `actuator`, `task`, `agent`) | **Stable** |
| Trigger model (`on`, `every`, `when`, `while`) | **Stable** |
| Typed handler return types | **Stable** |
| Regex literals and validation rules | **Stable** |
| Self-hosting compiler subset | **Future** |
| LLVM as language execution path | **Experimental** — see [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) |

Foundation: Phases 1–35 complete — [lean-core-roadmap.md](./lean-core-roadmap.md)

---

## Runtime

**Spanda Runtime** — interpreter, scheduler, HAL, concurrency, provider dispatch.

| Item | Status |
|------|--------|
| Tree-walking interpreter (primary path) | **Stable** |
| Cooperative concurrency (`spawn`, `join`, `select`) | **Stable** |
| Trigger-driven scheduler + telemetry flags | **Stable** |
| `spanda-certify` runtime gate | **Stable** |
| Real-time contracts (`deadline`, `jitter`, `priority`) | **Stable** |
| Reliability (watchdogs, modes, `recover from`) | **Stable** |
| World model / fusion belief hook | **Experimental** |
| Native binary via LLVM | **Experimental** — `spanda deploy --target native`, [native-deploy.md](./native-deploy.md) |

---

## Verification

**Spanda Verify** — hardware, capability, and behavioral verification.

| Item | Status |
|------|--------|
| `spanda verify` (profiles, `--simulate`, `--json`) | **Stable** |
| `deploy`, `requires_hardware`, hardware profiles | **Stable** |
| Behavioral `verify { }` assertions | **Stable** |
| Capability traceability matrices | **Stable** |
| `spanda check --verification-json` + LSP quick-fixes | **Stable** |
| CI integration guide | **Stable** — [ci-verify.md](./ci-verify.md) |
| Production verify on 5+ profiles | **Future** (v1.0) |
| Hardware adapter trait codegen | **Future** |

---

## Safety

**Spanda Safety** — type-level and runtime safety engine.

| Item | Status |
|------|--------|
| `ActionProposal` → `SafeAction` compile-time gate | **Stable** |
| `safety { }` zones, `max_speed`, `stop_if` | **Stable** |
| Kill switch + `remote_signed` handlers | **Stable** |
| Agent `can[]` capability clarity | **Stable** |
| Certification metadata (`certify`, `spanda certify prove`) | **Experimental** |
| Minimum-hardware safety analysis | **Stable** |

---

## Simulation

**Spanda Sim** and **Spanda Replay** — test and regress without hardware.

| Item | Status |
|------|--------|
| `spanda run` / `spanda sim` (physics-lite) | **Stable** |
| Digital twins (`twin`, mirror, replay buffer) | **Stable** |
| `simulate_compatibility` fault injection | **Stable** |
| Mission trace `--record` | **Stable** |
| `spanda replay` (`--deterministic`, `--playback`) | **Stable** |
| Wall-clock sim mode | **Stable** — [realtime.md](./realtime.md), [replay.md](./replay.md) |
| Twin cloud SaaS | **Future** |
| Full physics (Gazebo/Isaac class) | **Out of scope** — package bridges only |

---

## Health

**Spanda Health** — operational readiness and fleet policies.

| Item | Status |
|------|--------|
| `health_check`, `health_policy` | **Stable** |
| Fleet `require` clauses at runtime | **Stable** |
| `spanda demo health` showcase | **Stable** |
| Operational readiness engine (`spanda readiness`) | **Stable** — [readiness.md](./readiness.md) |
| Mission verification, failure analysis, safety reports | **Stable** — see readiness docs |
| Swarm quorum / mesh health | **Experimental** — [swarm-health.md](./swarm-health.md) |

---

## Fleet

**Spanda Fleet** — multi-robot simulation and distributed coordination.

| Item | Status |
|------|--------|
| `spanda fleet run` (in-process) | **Stable** |
| Fleet orchestrate (round-robin report) | **Stable** |
| HTTP fleet agents + `--remote` | **Experimental** — [fleet-distributed.md](./fleet-distributed.md) |
| Fleet mesh coordinator | **Experimental** |
| OTA deploy plan / rollout / rollback | **Stable** (local state) / remote **Experimental** |
| ROS2 rclpy golden path | **Experimental** — [ros2-golden-path.md](./ros2-golden-path.md) |
| `spanda ros2 check` | **Stable** |

---

## Packages

**Spanda Registry** and **Spanda Providers** — extensibility without bloating the core.

| Item | Status |
|------|--------|
| `spanda install` / `update` / `publish` | **Stable** |
| Hosted registry index (29 scaffolds) | **Stable** — [registry.md](./registry.md) |
| Provider dispatch + `--trace-providers` | **Stable** |
| Official packages (ROS2, MQTT, GPS, vision, …) | **Stable** scaffolds / live **Experimental** |
| Live AI providers (OpenAI, Anthropic, ONNX) | **Experimental** — [live-ai-provider.md](./live-ai-provider.md) |
| Live IoT / MQTT bridges | **Experimental** |
| Blockchain / ledger adapters | **Future** (community packages only) |

---

## Tooling

CLI, LSP, debugger, docs site, and contributor ergonomics.

| Item | Status |
|------|--------|
| Native CLI (`check`, `verify`, `run`, `sim`, `fleet`, `fmt`, `lint`) | **Stable** |
| `cargo install spanda` | **Stable** |
| Bundled `spanda demo {rover,safety,verify,fleet,health,readiness,assurance}` | **Stable** |
| Operations dashboard (`packages/web` Operations view) | **Experimental** |
| mdBook GitHub Pages | **Stable** |
| LSP hover + SafeAction quick-fix | **Stable** |
| VS Code snippets + VSIX CI | **Stable** |
| VS Code Marketplace listing | **Partial** — pending `VSCE_PAT` |
| DAP debugger | **Experimental** — [debugging.md](./debugging.md) |
| WASM web playground | **Experimental** |

---

## Release milestones

### v0.4 — Deploy path (current)

**Theme:** Native binaries, ROS2 polish, distributed fleet docs.

| Item | Status |
|------|--------|
| `spanda deploy --target native` | **Experimental** |
| `spanda compile-native` / LLVM golden paths | **Experimental** |
| `spanda ros2 check` | **Stable** |
| Distributed fleet guide | **Stable** |

### v0.3 — Tooling polish (complete)

**Theme:** IDE, diagnostics, registry, install ergonomics.

| Item | Status |
|------|--------|
| Crate rename → `spanda`, bundled demos | **Stable** |
| Hosted registry (29 scaffolds) | **Stable** |
| LSP + showcase CI smoke | **Stable** |

### v0.2 — Credibility & onboarding (complete)

**Theme:** Trust table, showcase demos, docs site, one-command demos.

| Item | Status |
|------|--------|
| Feature status audit + `spanda demo` | **Stable** |
| mdBook GitHub Pages | **Stable** |

### v1.0 — Production positioning

**Theme:** Trust for field deployment.

| Item | Tier |
|------|------|
| Interpreter + sim as supported LTS runtime | Stable |
| Safety + verify + replay as certified workflows | Stable |
| Native codegen for selected HAL profiles | Experimental → Stable |
| Self-hosting compiler subset | Future (not primary) |
| Blockchain / cryptocurrency adapters | **Out of scope** |
| Advanced swarm intelligence research | **Out of scope** |

---

## Related

- [platform-overview.md](./platform-overview.md)
- [feature-status.md](./feature-status.md)
- [product-strategy.md](./product-strategy.md)
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md)
- [lean-core-roadmap.md](./lean-core-roadmap.md)
- [roadmap-codebase-audit-2026-06.md](./roadmap-codebase-audit-2026-06.md)
