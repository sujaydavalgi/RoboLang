# Mission Verification

Mission verification determines whether a declared mission is **achievable** on the configured robot and hardware target.

## Mission syntax

```spanda
robot Rover {
  exposes capabilities [ gps_navigation, obstacle_avoidance ];

  mission WarehousePatrol {
    requires capabilities [
      obstacle_avoidance,
      gps_navigation
    ];
    patrol;
  }
}
```

## What is verified

- Mission achievable on current configuration
- Required hardware exists
- Required capabilities exist
- Required connectivity exists
- Sufficient battery (when mission duration declared)
- Sufficient compute (resource budgets)
- Safety requirements satisfied

## CLI

```bash
spanda verify mission examples/showcase/mission_verification/warehouse_patrol.sd
spanda readiness examples/showcase/mission_verification/warehouse_patrol.sd
```

Use `--json` for machine-readable `MissionVerificationReport` output.

## Human-in-the-loop approvals

Missions may require operator approval for high-risk actions:

```spanda
mission OpenGate {
  requires approval Operator for: open_gate;
  open_sequence;
}
```

Verify approval paths:

```bash
spanda verify-approval examples/showcase/mission_verification/gate_bot.sd
```

## API

```rust
use spanda_readiness::{verify_mission, MissionVerificationReport};
```

See also: [Readiness](readiness.md), [Safety Auditor](safety-auditor.md).
