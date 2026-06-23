# Safety Auditor

The **Autonomous Safety Auditor** performs static analysis to find deployment gaps before they reach the field.

## CLI

```bash
spanda audit examples/showcase/safety_report/rover.sd
spanda audit examples/showcase/readiness/rover.sd --json
```

## Checks performed

| Category | Example finding |
|----------|----------------|
| Kill switch | Missing or unsigned kill switch |
| Fallback | Robot missing degraded/hold mode |
| Safety block | Missing `safety { }` on robot |
| Capability | Minimum hardware not satisfied |
| Health | Missing health check declarations |
| Connectivity | Weak or missing secure comm |
| Authentication | Trust boundary gaps |
| Verification | Capability/traceability diagnostics |

## Severity levels

- **Critical** — blocks deployment
- **High** — must resolve before mission
- **Medium** — degraded readiness
- **Low** — informational

## Output

```
Critical: 1  High: 2  Medium: 1  Low: 0
[Critical] kill-switch — Missing kill switch declaration
[High] safety — Robot 'Rover' missing safety block
```

Exit code is non-zero when critical findings exist.

## API

```rust
use spanda_readiness::{audit_program, SafetyAuditReport};
```

See also: [Safety Reporting](safety-reporting.md), [Security](security.md).
