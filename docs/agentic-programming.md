# Agentic Programming

Spanda supports safety-gated agentic programming with tool permissions, memory scopes, and approval gates.

## Example

```spanda
agent Planner {
    goal "Navigate safely";
    tools [camera, lidar, map];
    memory short_term;
    policy safe_only;

    can [ read(lidar), propose_motion ];

    plan {
        let proposal = planner.reason(goal);
        let action = safety.validate(proposal);
        return action;
    }
}
```

## Rules

- Agents cannot directly execute actuators unless permitted and safety-gated
- High-risk actions require approval or `SafeAction`
- Empty `can []` **default-denies** `propose_motion` and `execute` at runtime (Phase 32)
- Capability grant/deny events are written to the audit trail when configured (Phase 31)
- Reasoning traces captured for audit when `audit` is configured
- `ActionProposal` must pass through `safety.validate` before actuator execution
- Agent `plan` blocks with motion grants must return `SafeAction` — see [typed-handler-io.md](./typed-handler-io.md)

**Examples:**

- [`examples/features/agent_capabilities.sd`](../examples/features/agent_capabilities.sd) — populated `can[]`
- [`examples/features/agent_can_deny.sd`](../examples/features/agent_can_deny.sd) — empty `can[]` denial

## Runtime

`spanda-ai` provides `AgentRuntime` with mock and live AI paths. Capability enforcement runs in the interpreter.

See [Architecture — AI Safety](./architecture.md) and [Feature Status](./feature-status.md).
