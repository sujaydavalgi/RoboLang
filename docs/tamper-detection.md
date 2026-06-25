# Tamper Detection

**Status:** Experimental (verify-time + trace runtime) · **Phase:** Verify, Operate, Recover · **Priority:** P3.1

Detect unauthorized modification, compromise, spoofing, tampering, or suspicious behavior in autonomous systems.

**Core question:** *Can this robot, device, fleet, mission, or provider still be trusted?*

## Threat types

Hardware tampering · Sensor spoofing · GPS spoofing · Firmware modification · Configuration tampering · Package tampering · Provider tampering · Unauthorized OTA · Network intrusion · Identity spoofing · Agent manipulation · Mission modification · Safety rule modification · Capability registry modification · Runtime injection · Replay attacks · Privilege escalation

## Framework types

| Type | Role |
|------|------|
| `TamperEvent` | Raw detection signal |
| `TamperAlert` | Operator-facing notification |
| `TamperEvidence` | Supporting data (hash, trace, telemetry) |
| `TamperSeverity` | Info · Low · Medium · High · Critical |
| `TamperPolicy` | Declarative response rules |
| `TamperDetectionResult` | Full analysis outcome |
| `TamperStatus` | Trusted · Suspicious · Tampered · Compromised · Unknown |

## CLI

```bash
spanda tamper-check rover.sd
spanda tamper-check rover.sd --json
spanda tamper-check mission.trace
spanda diagnose tamper mission.trace [--json]
```

Verify-time `spanda tamper-check` composes threat modeling, safety audit, security analysis, and structural integrity signals. Runtime analysis accepts `.trace` files (or `--runtime`) for capability denials and tamper events. `spanda diagnose tamper <trace>` adds tamper source, affected components, impact, timeline, and recovery recommendations (composes runtime tamper + spoofing alerts).

## Integration

Readiness · Assurance · Diagnosis · Health · Security · Capability verification · Hardware verification · Trust score · Audit · Replay

## Crate

`spanda-tamper` — evidence collection, detection engine, trust scorer, response dispatcher.

See [integrity-verification.md](./integrity-verification.md) · [trust-framework.md](./trust-framework.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
