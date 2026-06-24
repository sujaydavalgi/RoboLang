# Spanda examples

Runnable `.sd` programs for learning, demos, regression, and CI. **Start here** if you learn by reading code.

**Guided tutorials:** [Tutorials index](../docs/tutorials/README.md) · [Spanda 101](../docs/spanda-101/README.md) · [Getting started](../docs/getting-started.md)

---

## Quick start (three pillars)

Evaluators and new users should run these first:

| Pillar | Command | File |
|--------|---------|------|
| **Safety** | `spanda check …` (expect fail) | [`showcase/ai_safety_violation.sd`](showcase/ai_safety_violation.sd) |
| **Verify** | `spanda verify … --target RoverV1` | [`showcase/hardware_compatibility.sd`](showcase/hardware_compatibility.sd) |
| **Sim** | `spanda sim …` | [`showcase/killer_demo.sd`](showcase/killer_demo.sd) |

Walkthrough: [docs/killer-demo.md](../docs/killer-demo.md) · Index: [showcase/README.md](showcase/README.md)

```bash
spanda check examples/showcase/ai_safety_violation.sd && exit 1 || true
spanda check examples/showcase/killer_demo.sd
spanda verify examples/showcase/hardware_compatibility.sd --json --target RoverV1
spanda sim examples/showcase/killer_demo.sd
```

---

## Learning ladder

Work through tiers in order, or jump via [features/README.md](features/README.md) (one file per capability).

| Tier | Directory | Index | Spanda 101 |
|------|-----------|-------|------------|
| **1 — Basics** | [`basics/`](basics/) | [README](basics/README.md) | Lessons 1–5, 9 |
| **2 — Integration** | [`integration/`](integration/) | triggers, concurrency, verify | Lessons 7–8 |
| **3 — Features** | [`features/`](features/) | [README](features/README.md) | Lookup by topic |
| **4 — End-to-end** | [`end_to_end/`](end_to_end/) | [README](end_to_end/README.md) | Lesson 10 |
| **5 — Packages** | [`packages/`](packages/) | [README](packages/README.md) | Lesson 9 |

---

## By topic

| Topic | Directory | Doc |
|-------|-----------|-----|
| Showcase demos | [`showcase/`](showcase/) | [killer-demo.md](../docs/killer-demo.md) |
| Real-time & reliability | [`realtime/`](realtime/) | [realtime.md](../docs/realtime.md) |
| Mission replay | [`end_to_end/replay_mission.sd`](end_to_end/replay_mission.sd) | [replay.md](../docs/replay.md) |
| Triggers & concurrency | [`triggers_demo.sd`](triggers_demo.sd), [`concurrency.sd`](concurrency.sd) | [triggers.md](../docs/triggers.md) |
| Communication & fleet | [`communication/`](communication/) | [concurrency.md](../docs/concurrency.md) |
| Hardware & deploy | [`hardware/`](hardware/) | [hardware-compatibility.md](../docs/hardware-compatibility.md) |
| Health & capabilities | [`hardware/capability_verification.sd`](hardware/capability_verification.sd) | [health-checks.md](../docs/health-checks.md) |
| IoT & live bridges | [`iot/`](iot/) | [iot.md](../docs/iot.md) |
| Live AI providers | [`features/live_openai.sd`](features/live_openai.sd) | [live-ai-provider.md](../docs/live-ai-provider.md) |
| Testing & compile-fail | [`basics/12_compile_fail_tests.sd`](basics/12_compile_fail_tests.sd) | [testing.md](../docs/testing.md) |
| Verification diagnostics | [`hardware/capability_verification.sd`](hardware/capability_verification.sd) | [verification-diagnostics.md](../docs/verification-diagnostics.md) |
| Robotics platform | [`robotics/`](robotics/) | [robotics-platform.md](../docs/robotics-platform.md) |
| **Mission assurance** | [`assurance/`](assurance/), [`anomaly/`](anomaly/), [`diagnostics/`](diagnostics/), [`prognostics/`](prognostics/), [`resilience/`](resilience/), [`mission/`](mission/), [`showcase/assurance/`](showcase/assurance/) | [mission-assurance.md](../docs/mission-assurance.md) |
| Readiness & go/no-go | [`showcase/readiness/`](showcase/readiness/) | [readiness.md](../docs/readiness.md) |
| Regex | [`regex/`](regex/) | [regex.md](../docs/regex.md) |
| Security | [`security/`](security/) | [secure-communication.md](../docs/secure-communication.md) |
| FFI & ROS2 | [`ffi_*.sd`](ffi_python_extern.sd), [`ros2_bridge.sd`](ros2_bridge.sd) | [ffi-and-ecosystem.md](../docs/ffi-and-ecosystem.md) |
| Adapter bridges | [`adapters/`](adapters/) | [ros2-golden-path.md](../docs/ros2-golden-path.md) |
| Package layouts | [`packages/`](packages/) | [packages.md](../docs/packages.md) |
| Publish mirror workflow | [`packages/publish_mirror_project/`](packages/publish_mirror_project/) | [registry.md](../docs/registry.md) |
| Standard library | [`std/`](std/) | [standard-library.md](../docs/standard-library.md) |
| Type snippets | [`types/`](types/) | [spanda-type-system.md](../docs/spanda-type-system.md) |
| Modules | [`modules/`](modules/) | [spanda-language.md](../docs/spanda-language.md) |

---

## Official packages (lean-core)

Domain integrations (ROS2, MQTT, GPS, fleet, OTA, …) ship as **optional official packages** under [`packages/registry/`](../packages/registry/). Example projects show how to depend on them:

```bash
spanda check examples/packages/ros2_adapter_package/src/main.sd
spanda verify-adapter examples/packages/nav2_adapter_package
```

Registry catalog: [docs/official-packages.md](../docs/official-packages.md) (**37** hosted packages, including 8 mission assurance packages) · Architecture: [docs/lean-core.md](../docs/lean-core.md)

---

## Notable single-file demos (repo root)

| File | Topic |
|------|--------|
| [`lidar_avoidance.sd`](lidar_avoidance.sd) | Classic lidar stop |
| [`ai_navigation.sd`](ai_navigation.sd) | AI + navigation |
| [`humanoid_assistant.sd`](humanoid_assistant.sd) | Humanoid agent pattern |
| [`warehouse_logistics.sd`](warehouse_logistics.sd) | Logistics workflow |
| [`jetson_inspection.sd`](jetson_inspection.sd) | Edge vision / Jetson |

---

## Regression & CI

All examples are type-checked in CI:

```bash
./scripts/check_all_examples.sh
```

Golden fixtures: [`tests/golden/manifest.json`](../tests/golden/manifest.json) · Vitest: `npm test -- tests/golden/rust.test.ts`

**Tier 3 experimental golden paths** (MQTT, twin cloud, LLVM, cpp-native, ledger, world model, self-host lexer): [docs/tier-3-golden-paths.md](../docs/tier-3-golden-paths.md)

---

## Adding an example

1. Place under the right tier (`basics/`, `features/`, `end_to_end/`, …)
2. Header comment with `spanda check` / `run` commands
3. Update the directory **README** and [features/README.md](features/README.md) if applicable
4. Add to [docs/tutorials/README.md](../docs/tutorials/README.md) when it is a learning resource
5. Optional: `tests/golden/manifest.json` for CI

See [CONTRIBUTING.md](../CONTRIBUTING.md#how-to-add-an-example).
