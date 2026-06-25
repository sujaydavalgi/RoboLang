# Hardware Attestation

**Status:** Experimental · **Phase:** Verify, Deploy, Operate · **Priority:** P3.1

Optional live hardware attestation for secure-boot contract imports (`trust.jetson`, `trust.pi`).

## Verify-time endpoint

Set `SPANDA_ATTESTATION_ENDPOINT` to an HTTP URL that accepts POST JSON:

```json
{
  "contract": "trust.jetson",
  "package": "spanda-trust-jetson",
  "program": "rover.sd"
}
```

Response:

```json
{
  "attested": true,
  "boot_state": "verified",
  "score": 95,
  "detail": "tpm quote ok"
}
```

When configured, `spanda tamper-check` and `spanda integrity` merge live attestation into secure-boot coverage scores.

## Deploy agent status

Deploy agents expose attestation fields on `GET /v1/status` when set via environment:

| Variable | Field |
|----------|-------|
| `SPANDA_ATTESTATION_CONTRACT` | `attestation_contract` |
| `SPANDA_ATTESTATION_VERIFIED=1` | `attestation_verified` |
| `SPANDA_BOOT_STATE` | `boot_state` |

`spanda integrity <file.sd> --agent <Robot@Hardware>` compares attestation when present.

## Packages

- `spanda-trust-jetson` — Jetson secure-boot contract stub
- `spanda-trust-pi` — Raspberry Pi secure-boot contract stub

See [trust-framework.md](./trust-framework.md) · [tamper-detection.md](./tamper-detection.md) · [integrity-verification.md](./integrity-verification.md).
