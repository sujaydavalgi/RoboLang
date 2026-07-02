# Decision trail — differentiation NOW

Demonstrates **Decision Audit Trail** (differentiation pillar #6): sim emits v3 decision frames, then audit and explain commands reconstruct why the system acted.

```bash
export SPANDA_DECISION_TRACE=1
spanda sim main.sd --record --inject-health-faults
spanda audit decisions main.trace
spanda explain decision main.trace
spanda decision trace main.trace
```

Part of `spanda demo differentiation` and `scripts/differentiation_smoke.sh`.
