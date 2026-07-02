# Mission Risk Analysis — Stable Hardening Checklist

**Promoted 2026-07-02** — second NEXT differentiation pillar after What-If.

**Related:** [mission-risk-analysis.md](./mission-risk-analysis.md) · [differentiation-roadmap.md](./differentiation-roadmap.md)

| Gate | Status |
|------|--------|
| `scripts/risk_smoke.sh` | **Shipped** |
| `GET /v1/analytics/mission-risk` | **Shipped** |
| gRPC `GetAnalyticsMissionRisk` | **Shipped** |
| Stable gate | `scripts/risk_stable_promotion_gate.sh` |
| Field soak | **Pending** — `./scripts/risk_field_soak_init.sh` |

```bash
SPANDA_RISK_SKIP_SOAK=1 SPANDA_RISK_SKIP_SMOKE=1 ./scripts/risk_stable_promotion_gate.sh
```
