# Control Center desktop release runbook

Production macOS builds and optional signed auto-update for `@spanda/control-center-desktop`.

**Current release:** **0.5.0** — tag [`desktop-v0.5.0`](https://github.com/Davalgi/Spanda/releases/tag/desktop-v0.5.0) (GitHub Release + workflow artifacts).

---

## What operators get

| Artifact | Source |
|----------|--------|
| macOS `.dmg` / `.app.tar.gz` | GitHub Release for `desktop-v*` (when CI bundle succeeds) |
| Unsigned fallback | Workflow artifact `control-center-desktop-macos-<version>` |
| API backend | Separate — `spanda control-center serve` (desktop does not embed `spanda-api`) |

1. Start the API: `spanda control-center serve --bind 127.0.0.1:8080`
2. Install the desktop app from the GitHub Release
3. Optional: `VITE_CONTROL_CENTER_URL` at dev time; production builds default to local API URL from env

---

## Prerequisites (maintainers)

| Platform | Requirements |
|----------|----------------|
| macOS CI | `macos-latest` runner (workflow default) |
| Codesign (optional) | `APPLE_SIGNING_IDENTITY`, `APPLE_NOTARIZE_PROFILE` |
| Auto-update (optional) | `TAURI_UPDATER_PUBKEY`, `TAURI_SIGNING_PRIVATE_KEY` |
| Windows (future) | `WINDOWS_SIGNING_CERT` (optional CI secret) |

Without Apple secrets, CI still produces **unsigned** bundles and publishes a GitHub Release (installers or artifact download link in release notes).

---

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

This runs `control_center_desktop_smoke.sh` (Tauri `cargo check`) and fails on version mismatch.

---

## Release a new version

### 1. Bump versions

Update all three manifest files above (for example `0.4.3`), then:

```bash
./scripts/verify_desktop_release_ready.sh
git add packages/control-center-desktop/
git commit -m "release(control-center-desktop): bump to 0.4.3"
git push origin main
```

### 2. Tag and push

```bash
git tag desktop-v0.4.3
git push origin desktop-v0.4.3
```

Tag pattern **`desktop-v*`** must match the semver in the manifests.

### 3. Watch CI

**GitHub → Actions → Desktop Control Center release**

The workflow (`.github/workflows/desktop-release.yml`):

1. Runs `control_center_desktop_smoke.sh`
2. Builds Tauri bundle (`TAURI_BUILD=1`; `--no-sign` when `TAURI_SIGNING_PRIVATE_KEY` is unset)
3. Optionally runs `sign_tauri_macos.sh` when Apple secrets are set
4. Uploads workflow artifacts (`control-center-desktop-macos-<version>`)
5. Creates a **GitHub Release** with `.dmg` / `.app.tar.gz` when present

Manual trigger: **Actions → Desktop Control Center release → Run workflow** (no Release unless triggered by tag).

---

## GitHub Actions secrets (optional)

| Secret | Purpose |
|--------|---------|
| `TAURI_UPDATER_PUBKEY` | Embed updater public key at build (`build.rs`) |
| `TAURI_SIGNING_PRIVATE_KEY` | Sign updater artifacts; omit to pass `--no-sign` |
| `APPLE_SIGNING_IDENTITY` | macOS codesign |
| `APPLE_NOTARIZE_PROFILE` | notarytool profile for stapled notarization |

Set `SPANDA_DESKTOP_UPDATER_ACTIVE=1` in CI when `TAURI_UPDATER_PUBKEY` is configured.

---

## Local build

```bash
export TAURI_BUILD=1
npm run build --workspace=@spanda/control-center-desktop
# Optional after bundle:
./scripts/sign_tauri_macos.sh
```

Artifacts: `packages/control-center-desktop/src-tauri/target/release/bundle/`

---

## Auto-update (optional upgrade)

1. Generate updater keypair: `npm run tauri signer generate -- -w ~/.tauri/spanda-updater.key`
2. Set `TAURI_UPDATER_PUBKEY` in CI (injected via `src-tauri/build.rs`)
3. Set `TAURI_SIGNING_PRIVATE_KEY` for signed updater artifacts
4. Enable with `SPANDA_DESKTOP_UPDATER_ACTIVE=1` on release builds
5. Host update manifests at `https://releases.spanda.dev/control-center/...` (endpoint template in `tauri.conf.json`)

Until updater secrets are configured, operators install new versions manually from GitHub Releases.

---

## Key rotation

1. Generate new updater keypair
2. Ship dual-signed release accepting previous pubkey window (30 days)
3. Update CI secrets and `plugins.updater.pubkey` injection
4. Document rotation in fleet change log

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `verify_desktop_release_ready.sh` version mismatch | `package.json` / `Cargo.toml` / `tauri.conf.json` out of sync | Align semver in all three files |
| Tag pushed, no GitHub Release | Workflow failed before release step | Check Actions log; fix build; new patch tag |
| Release has no `.dmg` | Bundle step failed or wrong paths | Download workflow artifact instead |
| App won't open on macOS | Unsigned build | Right-click → Open, or configure Apple signing secrets and re-release |
| Desktop can't reach API | API not running or wrong URL | Start `spanda control-center serve`; set `VITE_CONTROL_CENTER_URL` in dev |

---

## Related

- [packages/control-center-desktop/README.md](../packages/control-center-desktop/README.md)
- [sdk-publishing.md](./sdk-publishing.md) — SDK + desktop tag table
- [control-center.md](./control-center.md) — API and UI reference
- [enterprise-ops-stable-promotion.md](./enterprise-ops-stable-promotion.md) — promotion checklist
