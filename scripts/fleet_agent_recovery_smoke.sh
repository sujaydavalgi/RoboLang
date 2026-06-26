#!/usr/bin/env bash
# Fleet agent interpreter recovery smoke — unit + mesh integration coverage.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== fleet agent recovery unit tests =="
cargo test -p spanda-fleet recovery_agent::tests --quiet

echo "== fleet agent interpreter recovery via mesh =="
cargo test -p spanda-fleet --test mesh_integration mesh_coordinator_relays_fleet_recovery_to_agents --quiet

echo "Fleet agent interpreter recovery smoke OK"
