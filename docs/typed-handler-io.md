# Typed handler I/O

Return type annotations on behaviors, tasks, triggers, events, and agent plans — validated at compile time in Rust and the TypeScript mirror (Phases 28–34).

**Example:** [`examples/features/typed_handler_returns.sd`](../examples/features/typed_handler_returns.sd)

---

## Syntax

### Behavior

```spanda
behavior status() -> Bool {
    return true;
}
```

### Task

```spanda
task Monitor every 50ms -> Bool {
    return battery_ok;
}
```

### Triggers

```spanda
every 100ms -> Bool {
    return true;
}

when lidar.nearest_distance > 1.0 m -> Bool {
    return true;
}

while operating -> Bool {
    return true;
}
```

### Event handlers

```spanda
event Alert;

on Alert -> Bool {
    return true;
}
```

Mismatching return types are compile errors:

```spanda
behavior bad() -> Bool {
    return 1;  // error: expected Bool, found Number
}
```

---

## Agent plan returns

When an agent is granted `propose_motion` or `execute`, the `plan` block must return `SafeAction` (not raw `ActionProposal`):

```spanda
agent Navigator {
    uses planner;
    can [ propose_motion ];

    plan {
        let proposal = planner.reason(prompt: "...", input: scan);
        let action = safety.validate(proposal);
        return action;  // SafeAction required
    }
}
```

See [agentic-programming.md](./agentic-programming.md) for capability enforcement.

---

## Agent `can[]` default deny

An empty `can []` denies high-risk actions at runtime (Phase 32):

```spanda
agent Restricted {
    can [];
    plan { /* propose_motion denied at runtime */ }
}
```

**Example:** [`examples/features/agent_can_deny.sd`](../examples/features/agent_can_deny.sd)

---

## Debugger

Typed handlers participate in DAP stepping. Debug sessions enter from `behavior`, `task every`, or top-level `every` trigger bodies.

**Example:** [`examples/integration/debugger_every.sd`](../examples/integration/debugger_every.sd) · [debugging.md](./debugging.md)

---

## Related

- [health-checks.md](./health-checks.md) — health-specific triggers (`on health … becomes`)
- [triggers.md](./triggers.md) — trigger execution model
- [verification-diagnostics.md](./verification-diagnostics.md) — compile-time verification output
