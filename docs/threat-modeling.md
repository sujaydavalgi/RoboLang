# Threat Modeling

**Status:** Experimental · **Phase:** Verify, Deploy · **Priority:** P1.2

Pre-deployment security analysis for autonomous systems declared in Spanda source.

## CLI

```bash
spanda threat-model rover.sd
spanda threat-model rover.sd --json
```

## Analysis surface

| Category | Sources in `.sd` |
|----------|------------------|
| Connectivity | `requires_network`, connectivity policies |
| MQTT / WiFi / LTE / BLE | Transport and package imports |
| OTA | `deploy`, OTA rollout declarations |
| Remote commands | `remote_signed`, operator topics |
| Agent permissions | `agent can[]` capability grants |
| Provider permissions | Provider registry + dispatch |

## Output types

- `ThreatReport` — full assessment
- `ThreatModel` — structured attack surface
- `ThreatAssessment` — per-threat risk level

Each report includes: **Attack Surface**, **Threats**, **Risk Levels**, **Recommended Mitigations**.

## Integration

Builds on `spanda-security` (`security check`), `trust-boundaries.md`, and capability analysis. Complements `spanda verify` and deployment gates.

## Crate

`spanda-threat` — static analysis composing `spanda-security`, connectivity declarations, and deploy surfaces.

See [security-assurance.md](./security-assurance.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
