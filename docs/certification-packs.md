# Certification Packs

**Status:** Planned · **Horizon:** LATER (6–12 months) · **Priority:** P2

Generate deployment-ready evidence bundles for field approval workflows.

## CLI

```bash
spanda certify rover.sd
spanda certify rover.sd --bundle deployment-evidence.zip
spanda certify rover.sd --format json|markdown
```

## Bundle contents

| Evidence type | Source |
|---------------|--------|
| Verification | `spanda verify --json` |
| Safety | Safety auditor + safety coverage |
| Readiness | `spanda readiness --json` |
| Assurance | `spanda assure` evidence cases |
| Trust | Trust framework composite score |
| Traceability | Capability + hardware matrices |
| Recovery | Recovery coverage report |
| Audit | Decision trail summary |

## Core types

`CertificationPack`, `CertificationEvidence`, `ComplianceEvidence`.

**Important:** Bundles are **evidence templates**, not accredited certifications. Compliance profile templates ship as packages.

Extends existing `certify` metadata and `spanda certify prove`.

See [differentiation-roadmap.md](./differentiation-roadmap.md) · [compliance-profiles.md](./compliance-profiles.md).
