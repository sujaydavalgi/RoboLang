# Spoofing Detection

**Status:** Planned (Future) · **Phase:** Operate, Recover · **Priority:** P3.5

Detect GPS and sensor spoofing through plausibility checks and cross-sensor fusion.

## Detection examples

| Signal | Detection |
|--------|-----------|
| GPS impossible movement | Velocity/acceleration bounds |
| GPS vs IMU conflict | Position delta mismatch |
| Camera vs localization conflict | Visual odometry disagreement |
| Sensor out of bounds | Declared `health_check` ranges |

## Output

- `SpoofingAlert` — sensor, confidence, evidence
- **Confidence score** (0–1) — never binary-only for Critical actions

## Response

Integrates with `tamper_policy` and `recovery_policy` — default: alert + audit; Critical may require human approval before kill switch.

## Implementation

**Package-backed** — core provides hooks in `spanda-connectivity-runtime` and fusion; vendor-specific models in `spanda-gps`, `spanda-fusion` packages.

## Demo

`examples/showcase/gps_spoofing/` — GPS reports impossible jump; IMU disagrees; alert generated.

See [tamper-detection.md](./tamper-detection.md) · [state-estimation.md](./state-estimation.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
