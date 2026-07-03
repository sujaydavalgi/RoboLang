#!/usr/bin/env bash
# Tag and push official SDK release (Rust, Python, TypeScript) to trigger publish workflows.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

read_version() {
  grep '^version' "$ROOT/crates/spanda-sdk/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/'
}

VERSION="${1:-$(read_version)}"
PY_VERSION="$(grep '^version' "$ROOT/sdk/python/pyproject.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')"
TS_VERSION="$(python3 -c 'import json; print(json.load(open("'"$ROOT/sdk/typescript/package.json"'"))["version"])')"

if [[ "$VERSION" != "$PY_VERSION" || "$VERSION" != "$TS_VERSION" ]]; then
  echo "SDK version mismatch: rust=$VERSION python=$PY_VERSION typescript=$TS_VERSION" >&2
  exit 1
fi

DRY_RUN=0
if [[ "${2:-}" == "--dry-run" ]]; then
  DRY_RUN=1
fi

TAGS=(
  "crates-sdk-v${VERSION}"
  "sdk-python-v${VERSION}"
  "npm-sdk-v${VERSION}"
)

echo "SDK release ${VERSION}"
for tag in "${TAGS[@]}"; do
  if git rev-parse "$tag" >/dev/null 2>&1; then
    echo "Tag exists: $tag (skip)"
  else
    echo "Would create: $tag"
    if [[ "$DRY_RUN" != "1" ]]; then
      git tag "$tag"
    fi
  fi
done

if [[ "$DRY_RUN" == "1" ]]; then
  echo "Dry run — no tags pushed. Run: ./scripts/verify_sdk_publish_ready.sh first."
  exit 0
fi

echo "== verify publish readiness =="
"$ROOT/scripts/verify_sdk_publish_ready.sh"

git push origin "${TAGS[@]}"
echo "Pushed ${TAGS[*]} — GitHub Actions will publish to crates.io, PyPI, and npm."
