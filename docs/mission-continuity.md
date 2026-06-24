# Mission continuity

Mission continuity ensures autonomous operations continue when robots fail, degrade, disconnect, or go offline. Spanda answers:

- **Who can take over?**
- **Can they safely take over?**
- **Should the mission resume or restart?**
- **What evidence supports the decision?**

Related: [self-healing.md](./self-healing.md) · [mission-assurance.md](./mission-assurance.md) · [concurrency.md](./concurrency.md)

---

## Architecture

The continuity framework lives in `spanda-assurance` and composes with readiness, recovery, capability, hardware verification, and trust gates.

| Component | Role |
|-----------|------|
| `MissionContinuityManager` | Top-level orchestrator — evaluates full continuity |
| `MissionCheckpointManager` | Captures and resolves mission checkpoints |
| `MissionStateTransferManager` | Builds snapshots and plans state transfer |
| `SuccessionPlanner` | Ranks successor candidates |
| `TakeoverCoordinator` | Coordinates takeover with safety validation |
| `MissionDelegationManager` | Plans ownership transfer and task redistribution |
| `MissionRecoveryPlanner` | Integrates recovery actions after disruption |
| `ContinuationDecisionEngine` | Decides continue / restart / partial restart / abort |

---

## Takeover modes

| Mode | Behavior |
|------|----------|
| `resume` | Continue from last checkpoint |
| `restart` | Start mission again |
| `partial_restart` | Restart only the failed stage |
| `shadow_takeover` | Backup agent already synchronized |
| `hot_takeover` | Immediate replacement (e.g. battery critical) |
| `cold_takeover` | Replacement initialized after failure |
| `human_takeover` | Transfer control to operator |

---

## CLI commands

```bash
# Full continuity evaluation
spanda continuity <file.sd> [--failed <name>] [--progress <pct>] [--trigger <kind>]

# Coordinate takeover
spanda takeover <file.sd> [--failed <name>] [--successor <name>] [--progress <pct>]

# Mission delegation / ownership transfer
spanda delegate <file.sd> [--failed <name>] [--to <name>] [--progress <pct>]

# Rank successor candidates
spanda succession <file.sd> [--failed <name>] [--scope fleet|swarm|robot]
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--failed` / `--failed-robot` | Entity that failed or went offline |
| `--progress` | Mission progress percent (0–100) |
| `--trigger` | `robot_failed`, `battery_critical`, `fleet_offline`, `swarm_lost`, … |
| `--scope` | `robot`, `fleet`, `swarm`, `group`, `crowd`, `mission_cluster` |
| `--successor` / `--to` | Target entity for takeover/delegation |
| `--json` / `--markdown` / `--html` | Report format |

---

## Example: warehouse inventory scan

Robot **ScannerAlpha** goes offline at **72%** progress during `WarehouseInventoryScan`.

```bash
spanda continuity examples/showcase/continuity/warehouse.sd \
  --failed ScannerAlpha --progress 72 --trigger robot_failed
```

**Evaluation:**

| Factor | Result |
|--------|--------|
| Replacement available | Yes (ScannerBeta, ScannerGamma) |
| Capability match | 100% |
| Health | Healthy |
| Readiness | Mission-ready |
| **Decision** | **Resume from checkpoint at 72%** |

---

## State transfer

`MissionStateSnapshot` captures:

- Current mission and completed steps
- Current goal and progress percent
- Checkpoints with mission, robot, health, safety, and capability state

`MissionContextTransfer` carries environment, safety, and health context to the successor.

---

## Succession selection

`SuccessorSelectionPolicy` weights:

- Capability match
- Health and readiness
- Location distance
- Battery and connectivity
- Trust score

Candidates blocked when untrusted, compromised, tampered, or not mission-ready.

---

## Reports

| Report | CLI |
|--------|-----|
| Mission Continuity Report | `spanda continuity` |
| Takeover Report | `spanda takeover` |
| Delegation Report | `spanda delegate` |
| Succession Report | `spanda succession` |

---

## Showcase examples

| Directory | Demonstrates |
|-----------|--------------|
| `examples/showcase/continuity/` | Checkpoint resume (warehouse scan) |
| `examples/showcase/takeover/` | Hot takeover on battery critical |
| `examples/showcase/delegation/` | Mission ownership transfer |
| `examples/showcase/swarm_takeover/` | Swarm member lost |
| `examples/showcase/fleet_succession/` | Fleet successor ranking |

---

## `continuity_policy` syntax

```spanda
continuity_policy WarehouseContinuity {
    on robot.failed {
        resume from checkpoint;
        reassign mission;
    }
    on battery.critical {
        hot takeover;
    }
}
```

Declared actions influence takeover mode inference (`resume`, `hot takeover`, `restart`, `human takeover`, etc.).

---

## Fleet runtime dispatch

When `SPANDA_FLEET_MESH_URL` is set, the interpreter relays takeover commands through the mesh coordinator:

```bash
# Mesh coordinator
POST /v1/fleet/continuity

# Fleet agent (deployed program required)
POST /v1/continuity/execute
POST /v1/continuity/ack
```

Peer topic: `fleet_takeover` (JSON `FleetContinuityRequest` payload).

Agent status reports `continuity_engine`, `continuity_successor`, `mission_progress_percent`, and `continuity_validation`.

---

## Integrations

| System | Integration |
|--------|-------------|
| **Readiness** | Successor must be mission-ready |
| **Assurance** | Takeover, delegation, continuity evidence |
| **Recovery** | Recovery actions after disruption |
| **Diagnosis** | Why takeover occurred; outcome tracking |
| **Trust** | Block untrusted/compromised successors |
| **Safety** | Safety, capability, hardware, mission verification gates |
