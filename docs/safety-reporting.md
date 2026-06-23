# Safety Reporting

Generate deployable **safety case evidence** for audits, certification, and release gates.

## CLI

```bash
spanda safety-report examples/showcase/safety_report/rover.sd
spanda safety-report examples/showcase/safety_report/rover.sd --json
spanda safety-report examples/showcase/safety_report/rover.sd --markdown
spanda safety-report examples/showcase/safety_report/rover.sd --html
```

## Report contents

| Section | Description |
|---------|-------------|
| Hardware Verification | Compatibility report |
| Capability Verification | Minimum capability checklist |
| Health Checks | Static + traceability matrix |
| Safety Rules | `safety { }` block rules |
| Kill Switch Validation | Declared kill switches |
| Connectivity Validation | Policy and hardware checks |
| Mission Verification | Mission achievability |
| Traceability Matrix | Requirement → capability → hardware → provider |
| Test Results | Summary pass/fail gates |
| Known Risks | Warnings and open gaps |

## Deployable gate

The report includes `deployable: true/false` when all critical gates pass.

## API

```rust
use spanda_readiness::{generate_safety_report, SafetyCaseReport};
```

See also: [Readiness](readiness.md), [Safety Auditor](safety-auditor.md), [CI Verify](ci-verify.md).
