# Kill Switch

Spanda provides first-class kill switch support for emergency stops that bypass noncritical task queues, preempt AI tasks, stop actuators, and record audit events.

**Examples:**

- [`examples/features/kill_switch.sd`](../examples/features/kill_switch.sd) — declaration + `on kill_switch` handler
- [`examples/security/remote_signed_kill_switch.sd`](../examples/security/remote_signed_kill_switch.sd) — `remote_signed` + identity
- [`examples/hardware/capability_verification.sd`](../examples/hardware/capability_verification.sd) — traceability context

---

## Syntax

```spanda
kill_switch EmergencyStop {
    source: hardware.button.E_STOP;
    priority: critical;
    remote_signed;
    action {
        stop_all_actuators();
        enter emergency_mode;
        audit.record("kill_switch_activated");
    }
}
```

Robot-scoped handler:

```spanda
robot Rover {
    behavior status() -> Bool {
        return true;
    }

    on kill_switch EmergencyStop {
        stop_all_actuators();
    }
}
```

## Rules

- Kill switches bypass noncritical task queues
- Kill switches preempt AI/agent tasks
- Actuators must stop on activation
- Activation is audited when an `audit` block is present
- Remote kill switches require `remote_signed` and verified commands (`RunOptions.kill_switch_signature` JSON at runtime)
- `spanda verify` reports an **error** when `remote_signed` is set without signed communication policy (Phase 35)
- `on kill_switch Name { }` handlers run after activation
- Available in simulation via `--trigger-kill-switch`

## CLI

```bash
spanda sim rover.sd --trigger-kill-switch EmergencyStop
# Remote-signed switches require a JSON signature payload at runtime (see spanda_security::SignedMessage)
spanda trace hardware rover.sd   # shows kill switch in traceability matrix
```

## Related

- [Health Checks](./health-checks.md) — `health_policy` can trigger kill switches on `Unsafe`
- [Verification diagnostics](./verification-diagnostics.md) — `--verification-json`, kill-switch severity
- [Hardware Capabilities](./hardware-capabilities.md) — `emergency_stop` actuator capability
- [Safety model](./architecture.md)
