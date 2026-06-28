# Package Trust Framework

**Status:** Experimental · **Phase:** Verify, Build · **Priority:** P0.4

Improve ecosystem trust with transparent scoring for registry packages.

## CLI

```bash
spanda trust spanda-mqtt
spanda trust spanda-mqtt --version 0.1.0 --json
spanda trust spanda-mqtt --project examples/showcase/rover
```

## Evaluation factors

| Factor | Weight | Signal |
|--------|--------|--------|
| Registry listed | 20 | Package appears in local/remote registry index |
| Official framework | 15 | Official catalog name **with registry provenance** (registry version or canonical `packages/registry/` path); path/git name squatting scores 0 when `--project` is set |
| License | 10 | Permissive license (Apache-2.0, MIT, BSD-3-Clause) |
| Maintained | 10 | At least one published version |
| Checksum | 15 | SHA-256 checksum in registry index |
| Signed | 20 | Ed25519 registry signature verified with trust key |
| Safety metadata | 10 | Vendored `spanda.toml` `[safety]` level |

## Output

`TrustScoreReport` (0–100) with factor breakdown, tier (`trusted` / `acceptable` / `low`), and recommendations. Pass threshold: **60**.

## Integration

- `spanda explain --config` includes a `package_trust` section for configured packages
- `spanda deploy gate --config` runs a `package_trust` gate when packages are declared in `spanda.toml`
- **`spanda deploy gate --policy production`** additionally enforces `official_provenance` (no path/git official-name overrides) and `registry_signatures` (`SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` + verified lockfile signatures) — see [deployment-gates.md](./deployment-gates.md)
- Feeds composite `TrustScore` in `spanda-trust` (`spanda trust <file.sd>`)
- When `SPANDA_REGISTRY_URL` is unset, the CLI defaults to the bundled registry slice (`spanda-trust-jetson`, `spanda-trust-pi`, `spanda-gps`, `spanda-fusion`) so `spanda trust` and `spoof-check` work offline after `cargo install` (sync via `scripts/sync_bundled_registry.sh`)

## Crate

`spanda-package::trust` — composes registry metadata, `registry_sign`, and vendored safety manifests.

See [trust-framework.md](./trust-framework.md) · [registry.md](./registry.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
