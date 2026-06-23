# Root Cause Analysis

Leverage the **mission replay** system to diagnose failures from recorded traces.

## CLI

```bash
spanda diagnose examples/showcase/root_cause_analysis/mission.trace
spanda diagnose examples/showcase/root_cause_analysis/mission.trace --json
```

## What is analyzed

- Triggers and scheduler events
- Messages and topic activity
- Provider calls (including failures)
- Safety decisions and kill switch activations
- Health status changes
- Failure events

## Output structure

```
Root Cause
provider_call: {"failed": true, "module": "positioning.gps", ...}

Contributing Factors
* Provider call failures detected
* Safety rules triggered during mission

Timeline
  T+0ms provider_call — ...
  T+1500ms health_change — GPS fix lost
  T+2000ms provider_call — GPS read failed
  T+2100ms safety_stop — navigation uncertainty

Recommended Actions
* Verify provider connectivity and credentials
* Review safety zone and stop_if thresholds
* Replay mission with --deterministic for confirmation
```

## Workflow

1. Record a mission: `spanda run --record --trace-out mission.trace`
2. Diagnose: `spanda diagnose mission.trace`
3. Confirm with deterministic replay: `spanda replay mission.trace --deterministic`

## API

```rust
use spanda_readiness::{diagnose_trace, RootCauseReport};
```

See also: [Replay](../examples/showcase/replay/), [Failure Analysis](failure-analysis.md).
