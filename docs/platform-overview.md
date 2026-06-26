# Spanda Platform Overview

<p align="center">
  <img src="../assets/image/low_res_logo.png" alt="Spanda logo" width="280">
</p>

**Spanda is an Autonomous Systems Platform with a safety-first programming language at its core.**

Short form: *The Autonomous Systems Platform.*

*Pronounced **SPUN-duh** (/ˈspʌndə/)* — Sanskrit for *the divine pulse*; see [overview/philosophy.md](./overview/philosophy.md).

This document explains how the **Spanda Platform** relates to the **Spanda Language**, what each major component does, and where to go next.

---

## Platform vs language

| | **Spanda Platform** | **Spanda Language** |
|---|---------------------|---------------------|
| **What it is** | End-to-end toolchain for designing, verifying, simulating, deploying, and operating autonomous systems | The `.sd` programming language — one core component of the platform |
| **Scope** | Runtime, verification, safety engine, simulation, replay, health, fleet, packages, providers, tooling | Syntax, types, robot primitives, safety types, units, and compile-time checks |
| **How you use it** | `spanda check`, `verify`, `sim`, `replay`, `fleet`, `install`, `demo`, … | Write `.sd` source; import packages; declare `robot`, `sensor`, `actuator`, `safety`, `deploy` |
| **Identity** | *Build. Verify. Simulate. Deploy. Operate.* | Robot-native, safety-typed, unit-aware source code |

The language is **not** being replaced or renamed. `.sd` files, language tutorials, and language reference docs remain first-class. The platform framing adds context: Spanda is more than a compiler — it is the layer where autonomous systems are designed, validated, and operated.

**Previous positioning:** *The Autonomous Systems Language* — still accurate for the language itself; the platform name reflects the full product scope. See [platform-positioning-migration.md](./platform-positioning-migration.md) for messaging guidance.

---

## Spanda Platform architecture

```
Spanda Platform
│
├── Spanda Language (.sd)
├── Spanda Runtime
├── Spanda Verify
├── Spanda Safety
├── Spanda Sim
├── Spanda Replay
├── Spanda Health
├── Spanda Readiness
├── Spanda Mission Assurance
├── Spanda Fleet
├── Spanda Registry
├── Spanda Providers
└── Enterprise Operations (Control Center, Device Pool, APIs, …)
```

### Spanda Language (.sd)

The safety-first programming language at the center of the platform.

- Source files use the **`.sd`** extension (unchanged).
- First-class concepts: `robot`, `sensor`, `actuator`, `ai_model`, `agent`, `safety`, `task`, `deploy`, `hardware`, `verify { }`, physical units.
- Safety types: `ActionProposal` (untrusted AI output) vs `SafeAction` (post-`safety.validate()`).
- Reference: [spanda-language.md](./spanda-language.md), [spanda-reference.md](./spanda-reference.md), [tutorials/README.md](./tutorials/README.md).

### Spanda Runtime

Execution layer after compile-time and certification gates.

- Tree-walking interpreter (`spanda-interpreter`) as the primary execution path.
- Scheduler, cooperative concurrency (`spawn`, `join`, `select`), HAL bindings, trigger-driven execution.
- Provider dispatch connects official packages to live or mock backends.
- Crates: `spanda-runtime`, `spanda-interpreter`, `spanda-concurrency`, `spanda-hal`, `spanda-certify`.
- Reference: [architecture.md](./architecture.md), [concurrency.md](./concurrency.md), [triggers.md](./triggers.md).

### Spanda Verify

Pre-deploy and continuous verification.

- **Hardware verification:** `spanda verify` — memory, sensors, actuators, timing, battery, network, AI model requirements against hardware profiles.
- **Capability verification:** exposure, grants, traceability matrices, minimum-hardware analysis.
- **Behavioral verification:** `verify { }` blocks and `simulate_compatibility` fault injection.
- Reference: [hardware-compatibility.md](./hardware-compatibility.md), [capability-traceability.md](./capability-traceability.md), [ci-verify.md](./ci-verify.md).

### Spanda Safety

Safety engine integrated with the type system and runtime.

- Compile-time gate: actuators require `SafeAction`, not `ActionProposal`.
- Runtime rules: `max_speed`, safety zones, `stop_if`, emergency stop.
- Kill switch: `kill_switch`, `remote_signed`, `on kill_switch` handlers.
- Reference: [agentic-programming.md](./agentic-programming.md), [kill-switch.md](./kill-switch.md).

### Spanda Sim

Simulation without physical hardware.

- `spanda run` and `spanda sim` with physics-lite 2D backend.
- Digital twins: `twin { mirror …; replay true; }` for shadow state.
- Fault injection via `simulate_compatibility`.
- Reference: [killer-demo.md](./killer-demo.md), examples in `examples/showcase/`.

### Spanda Replay

Mission trace capture and playback.

- Record: `spanda sim --record` → mission trace files.
- Replay: `spanda replay --deterministic` (parity check) or `--playback` (state snapshots).
- Reference: [replay.md](./replay.md), [realtime.md](./realtime.md).

### Spanda Health

Operational health monitoring and fleet readiness.

- `health_check`, `health_policy`, fleet `require` clauses evaluated at runtime.
- Integration with verification diagnostics and capability analysis.
- Reference: [health-checks.md](./health-checks.md), [fleet-health.md](./fleet-health.md).

### Spanda Readiness

Weighted operational go/no-go scoring before deploy and during operations.

- `spanda readiness` — composite score across verification, health, assurance, and deploy fit.
- Agent APIs: `spanda deploy agent readiness`, fleet readiness aggregates.
- Reference: [readiness.md](./readiness.md), [fleet-readiness.md](./fleet-readiness.md).

### Spanda Mission Assurance

Mission-grade autonomous operations assurance integrated with readiness and verification.

- Language: `knowledge_model`, `state_estimator`, `anomaly_detector`, `on anomaly`, `prognostics`, `mitigation`, `resilience_policy`, `continuity_policy`, `assurance_case`.
- CLI: `spanda assure`, `anomaly scan`, `state estimate`, `diagnose`, `prognostics`, `mission verify`, `resilience check`, `mitigation plan`, `continuity`, `takeover`, `delegate`, `succession`.
- Packages: `spanda-anomaly`, `spanda-fusion`, `spanda-diagnosis`, `spanda-prognostics`, `spanda-mission-planning`, `spanda-mission-continuity`, `spanda-resilience`, `spanda-knowledge-model`, `spanda-assurance`.
- Demo: `spanda demo assurance` on [`examples/showcase/assurance/rover.sd`](../examples/showcase/assurance/rover.sd); `spanda demo continuity` on [`examples/showcase/continuity/warehouse.sd`](../examples/showcase/continuity/warehouse.sd).
- Reference: [mission-assurance.md](./mission-assurance.md), [mission-continuity.md](./mission-continuity.md), [state-estimation.md](./state-estimation.md), [anomaly-detection.md](./anomaly-detection.md).

### Spanda Fleet

Multi-robot coordination and distributed operation.

- In-process: `spanda fleet run` for multi-robot simulation.
- Orchestration: `spanda fleet orchestrate`, mesh coordinator, HTTP agent relay (`--remote`, `--mesh-url`).
- OTA: deploy plan, rollout, rollback, remote agents.
- Reference: [concurrency.md](./concurrency.md), [fleet-distributed.md](./fleet-distributed.md).

### Spanda Registry

Package distribution and discovery.

- Hosted index (`registry/index.json`) with Ed25519-signed tarballs.
- CLI: `spanda install`, `spanda update`, `spanda publish`.
- Override registry URL via `SPANDA_REGISTRY_URL`.
- Reference: [registry.md](./registry.md), [packages.md](./packages.md).

### Spanda Providers

Extensibility through official and community packages.

- Provider traits and dispatch: ROS2, MQTT, GPS, SLAM, vision, fleet, OTA, cloud, and more.
- Lean core: transport and domain logic live in packages, not the language spec.
- Reference: [how-providers-work.md](./how-providers-work.md), [official-packages.md](./official-packages.md), [provider-interfaces.md](./provider-interfaces.md).

### Enterprise Operations (Control Center)

Production operations layer for fleet visibility, provisioning, governance, and integration — composes existing engines without duplicating them.

- **Control Center:** `spanda control-center serve` — React/TypeScript UI (`ControlCenterPanel`), Rust `spanda-api` backend, Tauri desktop scaffold.
- **Device Pool & Provisioning:** Central inventory, lifecycle states, discover → verify → assign → ready workflow.
- **Governance:** RBAC, secret management, alerting, compliance exports, digital thread query.
- **Integration:** REST v1, Python SDK, WebSocket telemetry, OTLP observability.
- Reference: [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md), [control-center.md](./control-center.md).

---

## Typical workflow

```
Write (.sd)  →  check  →  verify  →  sim  →  deploy  →  operate
     │            │          │         │        │          │
 Language    type/units   hardware   safety   target    health
             safety       capability  replay   fleet    readiness
                                       assurance replay
                                       continuity takeover
```

1. **Build** — Author robot programs in `.sd` with safety rules and deployment targets.
2. **Verify** — `spanda verify` against hardware profiles; capability and behavioral checks.
3. **Simulate** — `spanda sim` before hardware is available; inject faults.
4. **Assure** — `spanda assure` / `anomaly scan` / `state estimate` for mission assurance reports (optional `spanda demo assurance`).
5. **Deploy** — `deploy Robot to Profile`; optional native codegen (experimental) or interpreter on edge.
6. **Operate** — Health policies, readiness scoring, fleet coordination, mission continuity (takeover/delegation/succession), self-healing recovery, and mission replay for incidents.
7. **Govern** — Control Center for fleet visibility; RBAC, secrets, alerting, compliance exports (experimental).

Flagship walkthrough: [killer-demo.md](./killer-demo.md) · Platform demo: [examples/showcase/autonomous_rover/README.md](../examples/showcase/autonomous_rover/README.md) · Mission assurance: [examples/showcase/assurance/README.md](../examples/showcase/assurance/README.md) · Mission continuity: [examples/showcase/continuity/README.md](../examples/showcase/continuity/README.md).

---

## What Spanda is not

Spanda does not replace Python for ML training, C++ for low-level drivers, ROS2 for production transport at scale, or Gazebo/Isaac for full physics simulation. It **coordinates** and **verifies** autonomous logic across those layers — with safety and deploy fit enforced in one platform.

See [product-strategy.md](./product-strategy.md) for competitive positioning and [feature-status.md](./feature-status.md) for honest capability tiers.

---

## Related documents

| Document | Purpose |
|----------|---------|
| [README.md](../README.md) | Project home, quick start, examples |
| [vision.md](./vision.md) | Long-term vision and philosophy |
| [product-strategy.md](./product-strategy.md) | Priorities, pillars, release scope |
| [roadmap.md](./roadmap.md) | Roadmap by platform area |
| [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) | Control Center, Device Pool, provisioning, APIs (20 pillars) |
| [control-center.md](./control-center.md) | `spanda control-center serve`, REST v1, desktop scaffold |
| [mission-assurance.md](./mission-assurance.md) | Mission assurance CLI, packages, and examples |
| [mission-continuity.md](./mission-continuity.md) | Mission continuity, takeover, delegation, succession |
| [continuity-policies.md](./continuity-policies.md) | `continuity_policy` syntax and validation |
| [platform-positioning-migration.md](./platform-positioning-migration.md) | Messaging migration notes |
| [architecture.md](./architecture.md) | Compiler pipeline and crate map |
| [lean-core.md](./lean-core.md) | Lean-core workspace design |
