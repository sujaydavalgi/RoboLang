# Versioning policy

Spanda follows [Semantic Versioning 2.0.0](https://semver.org/). Version numbers are bumped **when milestones or important phases complete**, not on every merge.

**Canonical roadmap milestones:** [ROADMAP.md](../ROADMAP.md#release-milestones)

## Unified release streams

All **product release streams** share the **same semver** and bump together on each release. `scripts/bump_version.py` updates every manifest in one step.

| Stream | Manifests | Release tag |
|--------|-----------|-------------|
| **Workspace / CLI** | `Cargo.toml` `[workspace.package].version`, root `package.json`, `editor/vscode/`, `packages/lsp`, `packages/native`, `packages/web` | `vX.Y.Z` |
| **Rust SDK** | `crates/spanda-sdk/Cargo.toml` | `crates-sdk-vX.Y.Z` |
| **Python SDK** | `sdk/python/pyproject.toml`, `packages/sdk-python/pyproject.toml` | `sdk-python-vX.Y.Z` |
| **TypeScript SDK** | `sdk/typescript/package.json` | `npm-sdk-vX.Y.Z` |
| **Control Center desktop** | `packages/control-center-desktop/package.json`, `src-tauri/Cargo.toml`, `tauri.conf.json` | `desktop-vX.Y.Z` |
| **gRPC proto** | `crates/spanda-api/proto/spanda/v1/control_center.proto` | *(no tag — pin via `GET /v1/version`)* |

**Rule:** On every workspace release (`patch`, `minor`, or `major`), bump **all five streams** to the same `X.Y.Z`, then push all five tags. SDK-only hotfixes without a workspace release are rare; prefer a workspace **patch** so versions stay aligned.

## Semver component guide

| Component | Increment when | Applies to |
|-----------|----------------|------------|
| **Patch** (`0.5.0` → `0.5.1`) | Bug fixes, regressions, docs-only releases, smoke hardening, small non-breaking CLI/SDK polish **within** the current roadmap release line | **All streams** |
| **Minor** (`0.4.x` → `0.5.0`) | A **roadmap release milestone** completes (exit criteria met) or substantial additive platform features | **All streams** |
| **Major** (`0.5.x` → `1.0.0`) | **v1.0 production positioning** or breaking language, `/v1` API, proto, or SDK contracts | **All streams** |

Pre-1.0: **minor** carries roadmap themes (`0.4` → `0.5` → …). Reserve **major** for v1.0 and explicit breaking-change releases.

## Milestone → bump mapping

| Milestone / phase | Bump | Streams |
|-------------------|------|---------|
| Single bug fix or regression | **patch** | All five |
| Architecture / hardening phase (no new release theme) | **patch** or defer to next milestone | All five if releasing |
| Stable tier promotion (within current release line) | **patch** | All five |
| SDK/API parity (new REST/gRPC methods, no milestone) | **patch** | All five |
| **Roadmap release milestone** exit criteria met (v0.5, v1.0, …) | **minor** | All five |
| **Breaking** public contract | **major** | All five |
| gRPC additive RPCs only | proto **minor** | Proto only (product streams unchanged unless shipping) |

### Current milestone status (2026-07-02)

| Milestone | Version | Tags |
|-----------|---------|------|
| v0.4 — Deploy path | **0.4.0** | `v0.4.0` |
| v0.5 — Beta credibility | **0.5.0** | `v0.5.0`, `crates-sdk-v0.5.0`, `sdk-python-v0.5.0`, `npm-sdk-v0.5.0`, `desktop-v0.5.0` |

## Release workflow

### 1. Prepare

1. Ensure `CHANGELOG.md` has `[Unreleased]` entries for the milestone.
2. Dry run:

```bash
python3 scripts/bump_version.py minor --dry-run   # or patch / major
```

3. Verify desktop and SDK readiness:

```bash
./scripts/verify_desktop_release_ready.sh
./scripts/verify_sdk_publish_ready.sh
```

### 2. Bump (local or CI)

```bash
python3 scripts/bump_version.py minor
```

Or merge with label `release:minor` / `release:patch` / `release:major` (triggers **Auto release** → `scripts/bump_version.py`).

### 3. Tag and push (all five streams)

```bash
git push origin main
git tag v0.5.0 crates-sdk-v0.5.0 sdk-python-v0.5.0 npm-sdk-v0.5.0 desktop-v0.5.0
git push origin v0.5.0 crates-sdk-v0.5.0 sdk-python-v0.5.0 npm-sdk-v0.5.0 desktop-v0.5.0
```

| Tag | Triggers |
|-----|----------|
| `vX.Y.Z` | cargo-dist **Release** (CLI installers) |
| `crates-sdk-vX.Y.Z` | [publish-sdk-rust.yml](../.github/workflows/publish-sdk-rust.yml) |
| `sdk-python-vX.Y.Z` | [publish-sdk-python.yml](../.github/workflows/publish-sdk-python.yml) |
| `npm-sdk-vX.Y.Z` | [publish-sdk-typescript.yml](../.github/workflows/publish-sdk-typescript.yml) |
| `desktop-vX.Y.Z` | [desktop-release.yml](../.github/workflows/desktop-release.yml) |

## Checklist after a milestone

- [ ] Choose bump level (patch / minor / major)
- [ ] Run `python3 scripts/bump_version.py <component>`
- [ ] Update `ROADMAP.md` **Current release**
- [ ] Update `docs/feature-status.md` if tiers changed
- [ ] Push **all five** release tags
- [ ] Do not bump version in the same commit as unrelated WIP unless the milestone is complete

## Related docs

- [CONTRIBUTING.md](../CONTRIBUTING.md#releases) — PR labels and CI auto release
- [sdk-publishing.md](./sdk-publishing.md) — registry secrets and workflows
- [desktop-release-runbook.md](./desktop-release-runbook.md) — Tauri signing and artifacts
- [design-principles.md](./design-principles.md) — API surface versioning
