#!/usr/bin/env bash
# Copy minimal trust registry into the spanda CLI crate for offline secure-boot demos.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/crates/spanda-cli/bundled-registry"
mkdir -p "${DEST}/packages"

python3 - "${ROOT}" "${DEST}" <<'PY'
import json
import shutil
import sys
from pathlib import Path

root = Path(sys.argv[1])
dest = Path(sys.argv[2])
src_index = root / "registry" / "index.json"
names = {"spanda-trust-jetson", "spanda-trust-pi"}

entries = json.loads(src_index.read_text())
subset = [entry for entry in entries if entry.get("name") in names]
if len(subset) != len(names):
    missing = names - {entry["name"] for entry in subset}
    raise SystemExit(f"missing trust packages in index.json: {missing}")

(dest / "index.json").write_text(json.dumps(subset, indent=2) + "\n")

for name in sorted(names):
    src_pkg = root / "registry" / "packages" / name
    dst_pkg = dest / "packages" / name
    if dst_pkg.exists():
        shutil.rmtree(dst_pkg)
    shutil.copytree(src_pkg, dst_pkg)

print(f"✓ Bundled {len(subset)} trust packages to {dest}")
PY
