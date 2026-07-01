# Control Center desktop release runbook

Signed, notarized macOS builds and active auto-update for `@spanda/control-center-desktop`.

## Prerequisites

| Platform | Requirements |
|----------|----------------|
| macOS | Apple Developer ID, `APPLE_SIGNING_IDENTITY`, notarytool profile |
| Windows | Authenticode cert in `WINDOWS_SIGNING_CERT` (optional CI secret) |
| Updates | Tauri updater signing keypair (`TAURI_UPDATER_PUBKEY`, `TAURI_SIGNING_PRIVATE_KEY`) |

## Version sync

Keep these three files on the **same semver** before tagging:

| File | Field |
|------|-------|
| `packages/control-center-desktop/package.json` | `"version"` |
| `packages/control-center-desktop/src-tauri/Cargo.toml` | `version` |
| `packages/control-center-desktop/src-tauri/tauri.conf.json` | `"version"` |

```bash
./scripts/verify_desktop_release_ready.sh
```

## Release tag

```bash
# After version bump + verify script passes:
git tag desktop-v0.4.2
git push origin desktop-v0.4.2
```

Tag pattern **`desktop-v*`** triggers `.github/workflows/desktop-release.yml`, uploads macOS bundle artifacts, and creates a GitHub Release with `.dmg` / `.app.tar.gz` when present.

## Build

```bash
export TAURI_BUILD=1
npm run build --workspace=@spanda/control-center-desktop
./scripts/sign_tauri_macos.sh
```

## Auto-update

1. Generate updater keypair: `npm run tauri signer generate -- -w ~/.tauri/spanda-updater.key`
2. Set `TAURI_UPDATER_PUBKEY` in CI and embed in `tauri.conf.json` `plugins.updater.pubkey`
3. Enable with `SPANDA_DESKTOP_UPDATER_ACTIVE=1` for release builds
4. Publish artifacts to `https://releases.spanda.dev/control-center/...` (see workflow)

## Key rotation

1. Generate new updater keypair
2. Ship dual-signed release accepting previous pubkey window (30 days)
3. Update `plugins.updater.pubkey` and CI secret
4. Document rotation in fleet change log

## CI

`.github/workflows/desktop-release.yml` builds macOS artifacts on tag `desktop-v*` and runs `sign_tauri_macos.sh` when secrets are present.

## Related

- [packages/control-center-desktop/README.md](../packages/control-center-desktop/README.md)
