# Integrity Verification

**Status:** Planned (Future) · **Phase:** Verify, Deploy · **Priority:** P3.1

Verify that declared system artifacts match approved baselines.

## CLI

```bash
spanda integrity rover.sd
spanda integrity rover.sd --agent Rover@JetsonOrin --json
```

## Verified artifacts

| Artifact | Method |
|----------|--------|
| Hardware profiles | Profile hash vs agent report |
| Mission definitions | AST hash vs audit baseline |
| Capability definitions | Registry hash |
| Policies | Policy block hash |
| Safety rules | Safety AST hash |
| Health policies | Health decl hash |
| Package metadata | Registry signature + hash |
| Provider registrations | Provider manifest hash |

## Output

`IntegrityReport` — per-artifact status: **Trusted**, **Modified**, or **Unknown**.

## Foundation

Builds on `spanda-audit`, deploy bundle signatures (`--require-signature`, `--require-hash`), and `spanda certify prove`.

See [tamper-detection.md](./tamper-detection.md) · [audit-provenance.md](./audit-provenance.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
