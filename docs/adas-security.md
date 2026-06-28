# ADAS Security

Security integration for intelligent vehicles using the existing Spanda security framework.

**Config:** `examples/solutions/adas/spanda.security.toml`

---

## Threat scenarios

| Scenario | Detection | Response |
|----------|-----------|----------|
| CAN intrusion | Anomaly on unexpected CAN frames | `tamper_policy` → degraded mode |
| ECU firmware tampering | Secure boot attestation mismatch | Block deploy, quarantine device |
| OTA validation failure | `require_certify` on OTA plan | Rollback, readiness re-check |
| Sensor spoofing | Trace plausibility + trust score | Lower trust, request takeover |
| GPS spoofing | GPS anomaly detector | Switch to visual odometry, audit |
| Unauthorized provider | Provider registry allowlist | Block registration, alert |
| Certificate validation | TLS + signed message verification | Reject connection, audit |

---

## Configuration

```toml
# spanda.security.toml
[security]
encryption = "required"
authentication = "signed"
secure_boot = true

[security.trust]
min_package_trust = 0.8
require_signed_providers = true

[security.ota]
require_certify = true
rollback_on_readiness_fail = true
```

Program-level:

```spanda
import trust.jetson;

secure_comm {
  encryption: required;
  authentication: signed;
}

tamper_policy AutomotiveTamperResponse {
  on tamper severity Critical {
    enter degraded_mode;
    audit.record("adas_tamper_detected");
  }
}
```

---

## Secure communication

ISO 26262 profile requires `secure_comm` with encryption and signed authentication. Vehicle ECU communication via optional `spanda-canbus` and `spanda-automotive-ethernet` packages uses the same security contracts.

---

## OTA security

```bash
# Production policy enforces certification proof
SPANDA_OTA_REQUIRE_CERTIFY=1 spanda deploy rollout --remote

# Auto-rollback when post-deploy readiness fails
SPANDA_OTA_ROLLBACK_ON_READINESS_FAIL=1
```

Control Center: `GET /v1/ota/status`, `POST /v1/ota/plan` with canary/staged strategies.

---

## Trust validation

```bash
spanda verify src/highway_drive.sd --profile iso26262  # includes trust checks
spanda demo trust                                       # tamper showcases
```

Trust factors: package signatures, secure boot attestation, tamper policy presence, provider allowlist.

---

## Automotive protocol security

Optional packages implement security at the transport layer:

| Package | Protocols | Security features |
|---------|-----------|-------------------|
| `spanda-canbus` | CAN, CAN FD | Frame authentication hooks |
| `spanda-automotive-ethernet` | SOME/IP, DoIP | TLS, certificate pinning |
| `spanda-uds` | UDS, ISO-TP | Secure diagnostic sessions |
| `spanda-v2x` | DSRC, C-V2X | Message signing, cert validation |

These remain optional — configure in `spanda.providers.toml`.

---

## Showcase cross-references

| Showcase | Demonstrates |
|----------|--------------|
| `examples/showcase/gps_spoofing/` | GPS spoofing detection |
| `examples/showcase/secure_boot/` | Jetson attestation |
| `examples/showcase/tamper_policy/` | Runtime tamper response |
| `examples/showcase/package_tampering/` | Package trust scoring |
| `examples/showcase/compliance/automotive_rover.sd` | ISO 26262 security requirements |

---

## Related

- [security-architecture.md](./security-architecture.md) — Security contracts
- [tamper-detection.md](./tamper-detection.md) — Tamper detection
- [solutions/adas.md](./solutions/adas.md) — Blueprint architecture
- [provider-interfaces.md](./provider-interfaces.md) — Optional protocol packages
