# Showcase — three flagship examples

New evaluators should start with these three pillars — **safety**, **verify**, and **sim** — before browsing the full examples library.

Full 5-minute walkthrough: [docs/killer-demo.md](../../docs/killer-demo.md)

---

## 1. Safety — compile-time AI gate

**Purpose:** Prove that raw `ActionProposal` values cannot reach actuators without `safety.validate()`.

```bash
# Expect non-zero exit — ActionProposal rejected at compile time
spanda check examples/showcase/ai_safety_violation.sd

# Safe variant passes
spanda check examples/showcase/killer_demo.sd
```

| File | Role |
|------|------|
| [`ai_safety_violation.sd`](./ai_safety_violation.sd) | Minimal 15-line failure case |
| [`killer_demo.sd`](./killer_demo.sd) | Safe hero program with `safety.validate()` |

---

## 2. Verify — hardware fit before deploy

**Purpose:** Answer *"Will this program run on this robot?"* before flash or factory integration.

```bash
spanda verify examples/showcase/hardware_compatibility.sd
spanda verify examples/showcase/hardware_compatibility.sd --json --target RoverV1
```

| File | Role |
|------|------|
| [`hardware_compatibility.sd`](./hardware_compatibility.sd) | Deploy target, requirements, task budgets, fault simulation |
| [`killer_demo.sd`](./killer_demo.sd) | Same verify flow on the flagship patrol program |

CI integration: [docs/ci-verify.md](../../docs/ci-verify.md)

---

## 3. Sim — test without hardware

**Purpose:** Run patrol logic with simulated sensors and `stop_if` emergency rules.

```bash
spanda sim examples/showcase/killer_demo.sd
spanda verify examples/showcase/killer_demo.sd --simulate
```

| File | Role |
|------|------|
| [`killer_demo.sd`](./killer_demo.sd) | Patrol loop, lidar stop, AI planner, deploy + verify + sim |

Record and replay: [`examples/end_to_end/replay_mission.sd`](../end_to_end/replay_mission.sd)

---

## One-liner smoke script

```bash
set -e
spanda check examples/showcase/ai_safety_violation.sd && exit 1 || true
spanda check examples/showcase/killer_demo.sd
spanda verify examples/showcase/hardware_compatibility.sd --json --target RoverV1
spanda sim examples/showcase/killer_demo.sd
```

---

## Supplementary showcase demos

These illustrate additional capabilities but are **not** part of the evaluator quick path:

| File | Topic |
|------|-------|
| [`rover_navigation.sd`](./rover_navigation.sd) | Sensors, AI planning, motion |
| [`warehouse_robot.sd`](./warehouse_robot.sd) | Tasks, comms, safety zones |
| [`communication_demo.sd`](./communication_demo.sd) | Message, topic, service, action |
| [`digital_twin_demo.sd`](./digital_twin_demo.sd) | Twin, telemetry, replay |

Browse by capability: [examples/features/README.md](../features/README.md) · End-to-end workflows: [examples/end_to_end/README.md](../end_to_end/README.md)
