# Package Trust Framework

**Status:** Planned · **Phase:** Verify, Build · **Priority:** P0.4

Improve ecosystem trust with transparent scoring for registry packages.

## CLI

```bash
spanda package trust spanda-mqtt
spanda package trust spanda-mqtt --json
```

## Evaluation factors

| Factor | Signal |
|--------|--------|
| Signed | Ed25519 signature on bundle |
| Maintained | Recent publish date, changelog |
| Tests passing | Package `test` block / CI metadata |
| Coverage | Declared or inferred test coverage |
| Known vulnerabilities | CVE scan hook (package adapter) |
| Provider reputation | Maintainer and download metrics |

## Output

`TrustScore` (0–100) with factor breakdown and recommendations.

## Integration

Feeds composite `TrustScore` in tamper framework; readiness package factor; deployment gates.

## Crate

`spanda-trust` — composes `spanda-package`, `spanda-security`, `spanda-audit`.

See [trust-framework.md](./trust-framework.md) · [registry.md](./registry.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
