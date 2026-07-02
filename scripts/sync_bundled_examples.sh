#!/usr/bin/env bash
# Copy showcase examples into the spanda crate for cargo install / crates.io publish.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/crates/spanda-cli/bundled-examples/examples/showcase"
mkdir -p "${DEST}"

for d in unsafe_ai hardware_verification capability_verification health_monitoring fleet_management replay readiness assurance; do
  rm -rf "${DEST}/${d}"
  cp -R "${ROOT}/examples/showcase/${d}" "${DEST}/"
done

# Trust & tamper showcases (spanda demo trust, tamper-check, spoof-check).
for d in gps_spoofing package_tampering mission_tampering runtime_intrusion tamper_policy secure_boot compliance; do
  rm -rf "${DEST}/${d}"
  cp -R "${ROOT}/examples/showcase/${d}" "${DEST}/"
done

# Autonomous rover: source only (no vendored .spanda/ — `spanda demo rover` runs install).
rm -rf "${DEST}/autonomous_rover"
mkdir -p "${DEST}/autonomous_rover"
cp "${ROOT}/examples/showcase/autonomous_rover/spanda.toml" "${DEST}/autonomous_rover/"
if [[ -f "${ROOT}/examples/showcase/autonomous_rover/spanda.lock" ]]; then
  cp "${ROOT}/examples/showcase/autonomous_rover/spanda.lock" "${DEST}/autonomous_rover/"
fi
cp -R "${ROOT}/examples/showcase/autonomous_rover/src" "${DEST}/autonomous_rover/"
cp "${ROOT}/examples/showcase/autonomous_rover/README.md" "${DEST}/autonomous_rover/"

for f in killer_demo.sd ai_safety_violation.sd hardware_compatibility.sd README.md; do
  cp "${ROOT}/examples/showcase/${f}" "${DEST}/"
done

# Differentiation NOW/NEXT/LATER showcases (demo differentiation|later, smokes).
for d in differentiation what_if risk trust_graph scorecard forecast mission_twin certify human_robot governance policy distributed_decisions; do
  if [[ -d "${ROOT}/examples/showcase/${d}" ]]; then
    rm -rf "${DEST}/${d}"
    cp -R "${ROOT}/examples/showcase/${d}" "${DEST}/"
  fi
done

# ADAS solution blueprint (spanda demo adas).
ADAS_DEST="${ROOT}/crates/spanda-cli/bundled-examples/examples/solutions/adas"
rm -rf "${ADAS_DEST}"
mkdir -p "${ADAS_DEST}"
cp -R "${ROOT}/examples/solutions/adas/"* "${ADAS_DEST}/"

# Smart Spaces solution blueprint (spanda demo smart-spaces).
SMART_SPACES_DEST="${ROOT}/crates/spanda-cli/bundled-examples/examples/solutions/smart-spaces"
rm -rf "${SMART_SPACES_DEST}"
mkdir -p "${SMART_SPACES_DEST}"
cp -R "${ROOT}/examples/solutions/smart-spaces/"* "${SMART_SPACES_DEST}/"

echo "✓ Synced bundled examples to crates/spanda-cli/bundled-examples/"

if [[ -x "${ROOT}/scripts/sync_bundled_registry.sh" ]]; then
  "${ROOT}/scripts/sync_bundled_registry.sh"
fi
