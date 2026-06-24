<p align="center">
  <img src="assets/image/low_res_logo.png" alt="Spanda — The Autonomous Systems Platform" width="360">
</p>

# Spanda

**The Autonomous Systems Platform** — *with a safety-first programming language at its core.*

*The pulse of autonomous intelligence.*

---

## Build. Verify. Simulate. Deploy. Operate.

**For Autonomous Systems.**

Spanda is an autonomous systems platform centered on the **Spanda Language** (`.sd` files). One toolchain spans design through operations — from typed robot programs to safety gates, hardware checks, simulation, replay, fleet health, and package-backed integrations.

**Spanda provides:**

| | | |
|---|---|---|
| **Autonomous Systems Language** | **Safety Validation** | **Hardware Verification** |
| **Capability Verification** | **Simulation** | **Replay** |
| **Health Monitoring** | **Mission Assurance** | **Package Ecosystem** |
| **Provider System** | **Operational Readiness** | |

Repository: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda) · Platform guide: [docs/platform-overview.md](docs/platform-overview.md)

---

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

**Spanda** is a Sanskrit term meaning *the divine pulse* or *sacred tremor* or *divine vibration*, representing the creative pulsation of absolute consciousness and energy, manifesting as waves of expansion and contraction. It is the universal activity or first stir of awareness that creates and sustains the entire universe.

---

## Spanda Platform

Spanda consists of several major components. The language is the expressive core; verification, safety, simulation, and operations wrap around it.

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
└── Spanda Providers
```

| Component | What it does |
|-----------|--------------|
| **Spanda Language (.sd)** | Safety-first language for robots, agents, and edge systems — sensors, actuators, AI, safety rules, and deployment targets are first-class syntax |
| **Spanda Runtime** | Interpreter, scheduler, cooperative tasks, HAL bindings, and certified execution after compile-time gates |
| **Spanda Verify** | Hardware compatibility (`spanda verify`), capability exposure, behavioral `verify { }` assertions, and traceability matrices |
| **Spanda Safety** | `ActionProposal` → `SafeAction` gate, safety zones, `stop_if`, emergency stop, and kill-switch handlers |
| **Spanda Sim** | Physics-lite simulation (`spanda run` / `spanda sim`) and digital twins — test without hardware |
| **Spanda Replay** | Mission trace record, deterministic replay, and frame playback for regression and incident review |
| **Spanda Health** | `health_check`, fleet `require` clauses, health policies, and operational readiness gates |
| **Spanda Readiness** | Weighted go/no-go scoring (`spanda readiness`), fleet readiness, agent APIs |
| **Spanda Mission Assurance** | NASA-style knowledge models, state estimation, anomaly detection, prognostics, mitigation, resilience, assurance cases — `spanda assure`, `anomaly scan`, `state estimate`, … |
| **Spanda Fleet** | Multi-robot simulation, orchestration, mesh coordination, and distributed agent relay |
| **Spanda Registry** | Hosted package index, Ed25519-signed tarballs, and `spanda publish` / `spanda install` |
| **Spanda Providers** | Official packages (ROS2, MQTT, GPS, vision, fleet, OTA, cloud) wired through the provider registry |

Deep dive: [docs/platform-overview.md](docs/platform-overview.md) · Diagram: [docs/diagrams/README.md](docs/diagrams/README.md) · Crate map: [docs/architecture.md](docs/architecture.md)

---

## What is Spanda?

Spanda is an **autonomous systems platform** built around the **Spanda Language** — a typed programming language where sensors, AI models, actuators, safety rules, and deployment targets are first-class concepts in source code.

You write a `robot` block with sensors, actuators, safety zones, and agents. The compiler enforces physical units, validates AI proposals before they reach hardware, and checks that your program fits the deployment target before you ship.

```spanda
robot SafePatrol {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM { provider: "mock"; model: "patrol"; }

  safety {
    max_speed = 0.5 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  behavior patrol() {
    loop every 100ms {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "Plan motion", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }
}
```

---

## Why Spanda?

Building autonomous systems today means stitching together Python scripts, C++ drivers, ROS2 nodes, safety monitors, and deployment checklists — with no single platform that treats **AI output as untrusted**, **hardware fit as compile-time**, and **safety as mandatory**.

**Traditional languages focus on:**

- Algorithms
- Data structures
- Applications

**Spanda focuses on:**

- Autonomous systems
- Safety
- Hardware awareness
- Capability verification
- Simulation
- Operational health

Spanda exists to be that coordination layer: one platform where perception, planning, safety validation, simulation, verification, and deployment live together — with the `.sd` language as the expressive core.

---

## What makes Spanda different?

1. **Safety-Typed AI** — `ActionProposal` from LLMs and vision models cannot drive actuators; only `SafeAction` from `safety.validate()` can. Enforced at compile time and runtime.

2. **Hardware Verification** — `deploy Robot to Profile` and `spanda verify` check sensors, memory, timing, power, and network before deployment.

3. **Capability Verification** — Expose, grant, and trace robot capabilities; verify the system can actually perform the mission, not just compile.

4. **Simulation + Replay** — `spanda sim` validates behavior before hardware exists; `spanda replay` records and replays mission traces for regression and incident review.

5. **Health-Aware Runtime** — `health_check`, fleet `require` clauses, and policies monitor robots, fleets, and devices during operation.

6. **Package-Based Extensibility** — Lean core architecture; official packages (ROS2, MQTT, GPS, vision, fleet) extend via the provider registry without bloating the language.

### More differentiators

| Differentiator | What it means |
|----------------|---------------|
| **AI safety gate** | `ActionProposal` from LLMs/vision cannot drive actuators; only `SafeAction` from `safety.validate()` can |
| **Hardware verification** | `deploy Robot to Profile` + `spanda verify` checks sensors, memory, timing, and power before deploy |
| **Physical units** | `1.0 m/s`, `0.5 rad`, `100 ms` — unit algebra enforced at compile time |
| **Robot-native syntax** | Sensors, actuators, topics, services, actions, safety zones, and tasks are language keywords |
| **Deterministic scheduling** | `task every 50ms` with optional resource `budget { }` |
| **Real-time contracts** | `deadline`, `jitter <=`, `priority`, `critical isolated` tasks; latency `pipeline` budgets |
| **Reliability primitives** | Watchdogs, operating `mode` blocks, `recover from`, retry/fallback on faults |
| **Mission trace replay** | `spanda sim --record`, `spanda replay --deterministic` / `--playback` for regression and incident review |
| **First-class regex** | Literals, `Regex` type, string methods, trigger/subscribe filters, `validate` rules |
| **Trigger-driven execution** | Unified `on` / `every` / `when` / `while` handlers for events, topics, safety, state, and AI |
| **Cooperative concurrency** | `spawn`, `join`, `parallel`, channels, and `select` with scheduler telemetry |
| **Simulation built in** | `spanda run` / `spanda sim` — test without hardware |
| **Digital twins** | `twin { mirror pose; replay true; }` for shadow state and replay |
| **Platform packages** | `spanda install` / `update`, provider dispatch, `--trace-providers` | **37** hosted packages (ROS2, MQTT, GPS, vision, fleet, OTA, mission assurance, …) |
| **Mission assurance** | `knowledge_model`, `state_estimator`, `anomaly_detector`, `on anomaly`, `prognostics`, `mitigation`, `resilience_policy`, `assurance_case` | CLI: `assure`, `anomaly scan`, `state estimate`, `diagnose`, `prognostics`, `mission verify`, `resilience check`, `mitigation plan`; `spanda demo assurance` |
| **Weighted sensor fusion** | `observe { }`, `state_estimator`, `fusion.read()` | Type-weighted confidence; optional `spanda-fusion` (`assurance.fusion`) |
| **Learned anomaly detection** | `learned backend assurance.anomaly` | Runtime `scan_learned` + EMA volatility; optional ONNX (`SPANDA_ANOMALY_ONNX_MODEL_PATH`) |
| **World models** | `world_model { enabled; }` + `fusion.read()` belief hook | Observe → fused observation → belief-gated decisions |
| **Verification & DX** | `spanda verify --health`, traceability matrices, kill switch, health policies | Capability exposure, fleet `require` clauses, typed handler I/O |
| **Live providers (optional)** | OpenAI, Anthropic, ONNX via Python bridge; IoT live bridges | Mock fallback when keys or env flags are unset |
| **Package registry** | Hosted index + `spanda publish` mirror to `registry/packages/` | Ed25519-signed tarballs; override with `SPANDA_REGISTRY_URL` |

Lean-core status: Phases 1–35 complete — [docs/lean-core-roadmap.md](docs/lean-core-roadmap.md)

---

## Trust & feature status

Honest snapshot for evaluators ([full matrix](docs/feature-status.md)):

| Feature | Status |
|---------|--------|
| Parser | **Stable** |
| Type checker | **Stable** |
| Safety validation (`ActionProposal` → `SafeAction`) | **Stable** |
| Hardware verification (`spanda verify`) | **Stable** |
| Simulation (`spanda run` / `spanda sim`) | **Stable** |
| Mission replay (`--record`, `spanda replay`) | **Stable** |
| Package loading (`spanda install`, registry) | **Stable** |
| Connectivity (in-memory + optional live bridges) | **Stable** / live **Experimental** |
| Encryption & secure comm | **Stable** (wire frames); live TLS **Experimental** |
| Health framework | **Stable** |
| Mission assurance (static analysis + CLI) | **Stable** |
| Learned anomaly runtime + ONNX backends | **Experimental** |
| Fleet runtime (in-process + HTTP agents) | **Stable** / distributed **Experimental** |
| Debugger (DAP) | **Experimental** |
| LLVM backend | **Experimental** |
| LSP / VS Code extension | **Experimental** |
| Live AI providers (OpenAI, Anthropic, ONNX) | **Experimental** |
| ROS2 adapter | **Experimental** |

Limitations: [docs/known-limitations.md](docs/known-limitations.md)

---

## Quick start (5 minutes)

```bash
# Install (from clone)
./scripts/install.sh
# Or: cargo install --path crates/spanda-cli --locked

# Run flagship demo
spanda demo rover

# Mission assurance suite (knowledge, state, anomaly, resilience)
spanda demo assurance

# Or step by step:
spanda check examples/showcase/killer_demo.sd      # type-check
spanda verify examples/showcase/hardware_compatibility.sd  # hardware fit
spanda sim examples/showcase/killer_demo.sd        # simulate
```

Video script: [docs/demo-script.md](docs/demo-script.md) · Architecture: [docs/diagrams/](docs/diagrams/)

---

## Architecture overview

Spanda uses a **lean-core, package-first** workspace (Phases 1–35 complete). `spanda-driver` orchestrates compile and run; `spanda-interpreter` is the runtime composition root. `spanda-core` is the stable public facade for external embedders; first-party apps import focused workspace crates directly.

```
.sd source + spanda.toml
        ↓
   spanda-driver (orchestration)
        ↓
   lexer → parser → AST → type checker (+ units, safety, capabilities)
        ↓
   hardware verifier · behavioral verify · capability / health gates
        ↓
   spanda-certify runtime gate → interpreter + simulator
        ↓
   provider registry ← official packages (ROS2, MQTT, GPS, …)
        ↓
   SIR → LLVM → native binary (experimental)
```

| Layer | Crates | Responsibility |
|-------|--------|----------------|
| **Public facade** | `spanda-core` | Stable `spanda_core::` re-exports + thin shims |
| **Apps** | `spanda-cli`, `spanda-node`, `spanda-wasm`, `spanda-dap`, `spanda-llvm` | CLI, bindings, debugger, codegen — direct workspace deps |
| **Pipeline** | `spanda-driver`, `spanda-lexer`, `spanda-parser`, `spanda-ast`, `spanda-typecheck`, `spanda-sir`, `spanda-error` | Lex → parse → check → SIR; compile orchestration |
| **Runtime** | `spanda-interpreter`, `spanda-runtime`, `spanda-runtime-host`, `spanda-comm`, `spanda-safety`, `spanda-hal`, `spanda-concurrency` | Execution, scheduling, safety, HAL, cooperative tasks |
| **Transport** | `spanda-transport-routing`, `spanda-transport-*` | Adapters, live bridges, `RoutingCommBus` |
| **Domain** | `spanda-hardware`, `spanda-fleet`, `spanda-ota`, `spanda-certify`, `spanda-connectivity` | Verify, fleet, rollout, certification, connectivity |
| **FFI & bridge** | `spanda-bridge`, `spanda-ffi` | Python/C++ subprocess bridges, live AI/IoT paths |
| **Tooling** | `spanda-format`, `spanda-lint`, `spanda-codegen`, `spanda-docs` | fmt, lint, codegen, docgen |
| **Packages** | `spanda-package`, `spanda-providers` | `spanda.toml`, registry, provider bootstrap |
| **Official packages** | `packages/registry/*` | ROS2, MQTT, GPS, SLAM, vision, fleet, OTA, cloud, **8 mission assurance packages** (37 scaffolds) |
| **Mirror & UX** | `src/`, `packages/lsp`, `packages/web`, `editor/vscode` | TypeScript tests, LSP, web playground, extension |

Crate index: [crates/README.md](crates/README.md) · Diagrams: [docs/diagrams/](docs/diagrams/) · Deep dive: [docs/lean-core.md](docs/lean-core.md) · [docs/architecture.md](docs/architecture.md)

---

## Example code

### AI agent with safety validation

```spanda
robot Rover {
  sensor lidar: Lidar on "/scan";
  sensor camera: Camera on "/camera";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM {
    provider: "mock";
    model: "safe-planner";
    temperature: 0.1;
  }

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  agent Navigator {
    uses planner;
    tools [lidar, camera, wheels];
    memory short_term;
    goal "Reach destination while avoiding obstacles";

    plan {
      let scene = camera.analyze();
      let proposal = planner.reason(
        prompt: "Create a safe navigation action",
        input: scene
      );
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  behavior run() {
    loop every 100ms {
      Navigator.plan();
    }
  }
}
```

### Hardware deploy verification

```spanda
requires_hardware {
  memory >= 2 GB;
  sensors [ Camera, Lidar ];
}

hardware RoverV1 {
  cpu: CortexA78;
  memory: 4 GB;
  sensors [ Camera, Lidar, IMU ];
  actuators [ DifferentialDrive ];
  battery { capacity: 100 Wh; }
  timing { min_period: 10 ms; }
}

robot RoverMission {
  sensor camera: Camera on "/camera";
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  mission { duration: 1 h; }

  task control_loop every 50ms {
    budget {
      cpu <= 25%;
      memory <= 256 MB;
    }
    let scan = lidar.read();
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }

  verify {
    robot.velocity().linear <= 2.0 m/s;
  }
}

simulate_compatibility {
  fault BatteryDegradation;
}

deploy RoverMission to RoverV1;
```

```bash
spanda verify examples/showcase/hardware_compatibility.sd --json
```

### Learn Spanda

**[Tutorials index](docs/tutorials/README.md)** — all learning paths in one place.

| Track | Guide | Time |
|-------|-------|------|
| Plain English | [Spanda for Dummies](docs/spanda-for-dummies/README.md) | ~45 min |
| Hands-on course | [Spanda 101](docs/spanda-101/README.md) | ~3 hours |
| Quickstart | [Getting started](docs/getting-started.md) | ~10 min |

### Examples library

**[examples/README.md](examples/README.md)** — master index: killer demo, learning ladder, topics, packages, CI.

Start with the progressive ladder in [`examples/basics/`](examples/basics/README.md), then integration slices and end-to-end packages:

| Tier | Path | Highlights |
|------|------|------------|
| Basics | `examples/basics/01_minimal_robot.sd` → `11_observe_and_fusion.sd` | Language core from robot blocks to fusion |
| Features | [`examples/features/`](examples/features/README.md) | One file per capability — full coverage index |
| Integration | `examples/integration/` | Triggers, concurrency, verify walkthrough |
| End-to-end | [`examples/end_to_end/`](examples/end_to_end/README.md) | Patrol, warehouse, fleet, replay, real-time workflows |

### Flagship examples (start here)

Three pillars for evaluators — full library has 70+ files; start with these:

| Pillar | Purpose | Command |
|--------|---------|---------|
| **Safety** | Block unsafe AI at compile time | `spanda check examples/showcase/ai_safety_violation.sd` |
| **Verify** | Hardware fit before deploy | `spanda verify examples/showcase/hardware_compatibility.sd --json` |
| **Sim** | Patrol without hardware | `spanda sim examples/showcase/killer_demo.sd` |
| **Platform** | Packages → providers → replay | `cd examples/showcase/autonomous_rover && spanda install && spanda run src/rover.sd --trace-providers` |
| **Assurance** | Mission assurance CLI suite | `spanda demo assurance` |

5-minute walkthrough: [`docs/killer-demo.md`](docs/killer-demo.md) · Platform demo: [`examples/showcase/autonomous_rover/README.md`](examples/showcase/autonomous_rover/README.md) · Mission assurance: [`examples/showcase/assurance/README.md`](examples/showcase/assurance/README.md) · Tier 3 CI golden paths: [`docs/tier-3-golden-paths.md`](docs/tier-3-golden-paths.md)

More showcase demos: [`examples/showcase/README.md`](examples/showcase/README.md). Real-time: [`examples/realtime/`](examples/realtime/); regex: [`examples/regex/`](examples/regex/).

---

## Installation

### Quick install (from source)

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
./scripts/install.sh
spanda demo rover
```

Equivalent: `cargo install --path crates/spanda-cli --locked` (installs the `spanda` binary).

### Prebuilt packages (Linux, macOS, Windows)

Download installable packages from [GitHub Releases](https://github.com/Davalgi/Spanda/releases):

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

Windows: use the `.msi` installer or PowerShell script from the same release page.

Full install guide: [docs/installation.md](docs/installation.md)

### Build from source

#### Prerequisites

- **Node.js** 18+ (for TypeScript tooling and tests)
- **Rust** stable (for native CLI and authoritative runtime)
- **npm** (workspace manager)

#### Clone and build

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
npm install
npm run build:rust    # builds target/release/spanda
npm run build         # TypeScript mirror (tsc) — must pass in CI
npm test
```

The native CLI is at `target/release/spanda`. Add it to your `PATH` or use `npm run spanda:native -- <command>`.

### Web playground (optional)

```bash
npm run build:wasm
npm run web:dev       # http://localhost:5173
```

---

## CLI commands

| Command | Description |
|---------|-------------|
| `spanda init [name]` | Create a new Spanda project |
| `spanda check <file.sd>` | Type-check |
| `spanda verify <file.sd>` | Hardware compatibility verification |
| `spanda run <file.sd>` | Run with simulated backend |
| `spanda sim <file.sd>` | Run simulation with detailed output |
| `spanda fleet run <file.sd>` | Run multi-robot fleet simulation (in-process) |
| `spanda replay <mission.trace>` | Inspect, verify, or play back a recorded mission trace |
| `spanda demo <rover\|safety\|verify\|fleet\|health\|readiness\|assurance>` | One-command showcase demos |
| `spanda assure <file.sd>` | Mission assurance report (evidence, verification, traceability) |
| `spanda anomaly scan <file.sd>` | Anomaly detector analysis (+ learned backends) |
| `spanda state estimate <file.sd>` | State estimators and weighted fusion previews |
| `spanda diagnose <file.sd\|trace>` | Fault diagnosis and root-cause timeline |
| `spanda prognostics <file.sd>` | RUL and degradation warnings |
| `spanda mission verify <file.sd>` | Mission plan achievability |
| `spanda resilience check <file.sd>` | Resilience policies and readiness score |
| `spanda mitigation plan <file.sd>` | Recovery actions and mode transitions |
| `spanda readiness <file.sd>` | Operational go/no-go scoring |
| `spanda test` | Run project tests |
| `spanda fmt <file.sd>` | Format source |
| `spanda lint <file.sd>` | Lint source |
| `spanda doc <file.sd>` | Generate markdown documentation |
| `spanda build` | Build project |
| `spanda install` | Install dependencies |
| `spanda update` | Refresh lockfile and vendored packages |
| `spanda publish` | Publish package tarball to registry mirror |
| `spanda deploy --target native <file.sd>` | Link native binary (experimental; requires LLVM feature) |
| `spanda compile-native <file.sd>` | Emit native binary via SIR → LLVM (experimental) |
| `spanda ros2 check` | Validate ROS 2 distro, rclpy, and bridge before live transport |
| `spanda twin export <file.sd> --out <replay.json>` | Export twin replay buffer as JSON |

Verify flags: `--target <Profile>`, `--all-targets`, `--simulate`, `--json`

Run/sim/fleet trace flags: `--trace-scheduler`, `--trace-tasks`, `--trace-triggers`, `--trace-events`, `--trace-providers`, `--trace-realtime`, `--metrics-json`, `--record`, `--wall-clock`, `--replay` (sim)

Replay flags: `--from T+mm:ss`, `--deterministic` (re-run source and verify frame parity), `--playback` (apply recorded state snapshots)

Quick start guide: [docs/getting-started.md](docs/getting-started.md) · Real-time & replay: [docs/realtime.md](docs/realtime.md), [docs/replay.md](docs/replay.md)

---

## Hardware verification

Spanda checks that autonomous programs **fit the deployment target** — sensors, actuators, memory, GPU, timing, network, power, and AI model requirements.

```bash
spanda verify examples/showcase/hardware_compatibility.sd
spanda verify rover.sd --target RoverV1 --all-targets
spanda verify rover.sd --simulate --json
```

Built-in profiles: `RoverV1`, `RoverV2`, `JetsonOrin`, `RaspberryPi5`, `ESP32`.

Full reference: [docs/hardware-compatibility.md](docs/hardware-compatibility.md)

---

## Safety model

Safety rules in the `safety { }` block are evaluated **before every motion command**:

1. **`max_speed = X m/s`** — clamps drive velocity
2. **`zone`** — circular or rectangular keep-out regions
3. **`stop_if <condition>`** — triggers emergency stop when true
4. **`ActionProposal` → `SafeAction`** — AI outputs cannot reach actuators without `safety.validate()`

Invalid (compile error):

```spanda
wheels.execute(proposal);  // requires SafeAction, not ActionProposal
```

Valid:

```spanda
let action = safety.validate(proposal);
wheels.execute(action);
```

---

## Package ecosystem

Spanda includes a package manager for modular robot programs:

```bash
spanda init my_robot
spanda add local_dependency
spanda build
spanda test
```

Manifest format: [docs/spanda-toml.md](docs/spanda-toml.md)  
Package guide: [docs/packages.md](docs/packages.md)

---

## Roadmap

**v0.4.0 (current):** Native deploy (`spanda deploy --target native`, experimental), `spanda ros2 check`, distributed fleet docs, bundled demos (`spanda demo`), `cargo install spanda`, LSP hover/quick-fixes, live IoT CI, hosted registry index (**37** packages), **mission assurance** CLI suite and showcase.

**v0.3.0 (shipped):** Tooling polish — crate rename to `spanda`, showcase demos without clone, fleet multi-robot fix, verification & DX (Phases 27–35), docs site on GitHub Pages.

**Next:** VS Code Marketplace publish (`VSCE_PAT`), IDE polish, native codegen golden paths. **v1.0:** production verify on 5+ profiles, edge deploy golden path. See [docs/roadmap.md](docs/roadmap.md).

Full roadmap: [docs/roadmap.md](docs/roadmap.md)  
Feature status: [docs/feature-status.md](docs/feature-status.md)  
Vision: [docs/vision.md](docs/vision.md)

---

## Contributing

We welcome contributions — bug reports, examples, documentation, and language proposals.

- [CONTRIBUTING.md](CONTRIBUTING.md) — build, test, coding standards
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- [Issue templates](.github/ISSUE_TEMPLATE/) — bug, feature, language proposal, package proposal

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
npm test
python3 scripts/normalize_inline_docs.py   # after bulk inline doc edits
```

Rust and TypeScript sources use **inline API documentation** (inside function bodies) and plain-English block comments before logic. See [CONTRIBUTING.md](CONTRIBUTING.md#inline-documentation).

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/platform-overview.md](docs/platform-overview.md) | Spanda Platform — components and platform vs language |
| [docs/getting-started.md](docs/getting-started.md) | First robot in 10 minutes |
| [docs/health-checks.md](docs/health-checks.md) | Health checks, fleet `require` clauses, policies |
| [docs/readiness.md](docs/readiness.md) | Operational readiness engine and weighted go/no-go scoring |
| [docs/mission-assurance.md](docs/mission-assurance.md) | Mission assurance domains, CLI, packages, and examples |
| [docs/state-estimation.md](docs/state-estimation.md) | State estimators, weighted fusion, `spanda state estimate` |
| [docs/anomaly-detection.md](docs/anomaly-detection.md) | Anomaly detectors, learned backends, ONNX inference |
| [docs/knowledge-models.md](docs/knowledge-models.md) | System knowledge models and dependencies |
| [docs/diagnostics.md](docs/diagnostics.md) | Fault diagnosis and `spanda diagnose` |
| [docs/prognostics.md](docs/prognostics.md) | Prognostics and remaining useful life |
| [docs/resilience.md](docs/resilience.md) | Resilience policies and degraded-mode recovery |
| [docs/assurance-cases.md](docs/assurance-cases.md) | Assurance cases and evidence linking |
| [docs/official-packages.md](docs/official-packages.md) | Official package catalog (37 hosted packages) |
| [docs/kill-switch.md](docs/kill-switch.md) | Kill switch syntax, `remote_signed`, handlers |
| [docs/iot.md](docs/iot.md) | IoT packages, dispatch, live bridge env flags |
| [docs/live-ai-provider.md](docs/live-ai-provider.md) | OpenAI, Anthropic, ONNX live paths |
| [docs/debugging.md](docs/debugging.md) | VS Code DAP — `behavior`, `task every`, `every` triggers |
| [docs/capability-traceability.md](docs/capability-traceability.md) | Capability exposure and traceability matrices |
| [docs/verification-diagnostics.md](docs/verification-diagnostics.md) | `--verification-json` and LSP quick-fixes |
| [docs/typed-handler-io.md](docs/typed-handler-io.md) | Handler return type annotations |
| [docs/testing.md](docs/testing.md) | `expect_compile_error` and test CLI |
| [docs/packages.md](docs/packages.md) | Package manager, `spanda publish`, capabilities |
| [docs/registry.md](docs/registry.md) | Hosted registry, signatures, golden path |
| [docs/killer-demo.md](docs/killer-demo.md) | 5-minute safety + verify + sim walkthrough |
| [docs/known-limitations.md](docs/known-limitations.md) | Honest platform constraints |
| [docs/benchmarks.md](docs/benchmarks.md) | Reproducible timing commands |
| [docs/demo-script.md](docs/demo-script.md) | 3-minute video walkthrough script |
| [docs/diagrams/](docs/diagrams/) | Architecture Mermaid diagrams |
| [docs/realtime.md](docs/realtime.md) | Deadline-aware tasks, wall-clock scheduling |
| [docs/reliability.md](docs/reliability.md) | Pipelines, watchdogs, recovery, operating modes |
| [docs/replay.md](docs/replay.md) | Mission trace record, deterministic replay, playback |
| [docs/regex.md](docs/regex.md) | Regex literals, triggers, subscription filters |
| [docs/triggers.md](docs/triggers.md) | Trigger-driven execution model |
| [docs/concurrency.md](docs/concurrency.md) | Tasks, spawn, channels, fleet CLI |
| [docs/architecture.md](docs/architecture.md) | Compiler pipeline and workspace crate map |
| [docs/lean-core.md](docs/lean-core.md) | Lean-core architecture (Phases 1–35) |
| [crates/README.md](crates/README.md) | Workspace crate index |
| [docs/feature-status.md](docs/feature-status.md) | Stable vs experimental vs planned |
| [docs/product-strategy.md](docs/product-strategy.md) | v0.5 beta priorities and positioning |
| [docs/spanda-language.md](docs/spanda-language.md) | Language reference |
| [docs/spanda-reference.md](docs/spanda-reference.md) | Full language API (JavaDoc + man-style CLI) |
| [docs/api-documentation.md](docs/api-documentation.md) | API doc hierarchy (language → compiler → JSON) |
| [docs/api-reference.md](docs/api-reference.md) | Rust/TypeScript compiler API (grouped by layer) |
| [docs/tutorials/README.md](docs/tutorials/README.md) | All tutorials, walkthroughs, and learning paths |
| [examples/README.md](examples/README.md) | Runnable examples library index |
| [docs/man/](docs/man/) | CLI manual pages |
| [docs/README.md](docs/README.md) | Full documentation index |

---

## License

Apache-2.0 — see [LICENSE](LICENSE).
