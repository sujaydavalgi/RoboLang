#!/usr/bin/env bash
# Verify PyPI and npm SDK publish readiness without publishing.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if command -v python3 >/dev/null 2>&1; then
  PYTHON=python3
elif command -v python >/dev/null 2>&1; then
  PYTHON=python
else
  echo "python3 or python required on PATH" >&2
  exit 1
fi

VERIFY_VENV="$(mktemp -d "${TMPDIR:-/tmp}/spanda-sdk-verify-venv.XXXXXX")"
cleanup() {
  deactivate 2>/dev/null || true
  rm -rf "$VERIFY_VENV"
}
trap cleanup EXIT

"$PYTHON" -m venv "$VERIFY_VENV"
# shellcheck source=/dev/null
source "$VERIFY_VENV/bin/activate"
PIP="python -m pip"
PYTEST="python -m pytest"

echo "== Python SDK (canonical sdk/python) =="
$PIP install -q -e "sdk/python[dev]"
$PYTEST sdk/python/tests -q
$PIP install -q build
(cd sdk/python && python -m build >/dev/null)
echo "Python wheel OK ($(ls sdk/python/dist/*.whl | tail -1))"
echo "== Python SDK (legacy packages/sdk-python) =="
$PIP install -q -e "packages/sdk-python[dev]"
$PYTEST packages/sdk-python/tests -q
echo "== Rust spanda-sdk (cargo package) =="
cargo package -p spanda-sdk --allow-dirty >/dev/null
echo "Rust crate package OK"
echo "== TypeScript @davalgi-spanda/sdk =="
npm ci --prefix sdk/typescript
npm test --prefix sdk/typescript
npm run build --prefix sdk/typescript
echo "== npm @davalgi-spanda/web =="
npm ci
npm run build --workspace=@davalgi-spanda/web
(cd packages/web && npm pack >/dev/null)
echo "npm pack OK"
echo "Publish readiness verified. Tag with: ./scripts/publish_sdk_release.sh [--dry-run]"
