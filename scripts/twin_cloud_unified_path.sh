#!/usr/bin/env bash
# Unified Twin Cloud golden path — legacy replay upload + structured SaaS snapshots.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== Legacy replay export + SPANDA_CLOUD_UPLOAD_URL =="
"$ROOT/scripts/twin_cloud_golden_path.sh"

echo "== Twin Cloud SaaS structured snapshots =="
"$ROOT/scripts/twin_cloud_saas_smoke.sh"

echo "Twin Cloud unified path OK"
