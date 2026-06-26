#!/usr/bin/env bash
# Verify bundled registry resolves trust and spoofing packages without SPANDA_REGISTRY_URL.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="${ROOT}/crates/spanda-cli/Cargo.toml"

echo "== bundled demo trust (no SPANDA_ROOT) =="
cd /tmp
unset SPANDA_ROOT
unset SPANDA_REGISTRY_URL
OUT="$(cargo run --manifest-path "${CLI}" -q -- demo trust 2>&1 || true)"
echo "$OUT" | grep -q "Trust & tamper"
echo "$OUT" | grep -q "Secure boot contracts"
echo "$OUT" | grep -q "trust.jetson"
echo "$OUT" | grep -q "Demo complete"

echo "== bundled registry trust packages (no SPANDA_REGISTRY_URL) =="
cd /tmp
unset SPANDA_REGISTRY_URL
TRUST_GPS="$(cargo run --manifest-path "${CLI}" -q -- trust spanda-gps 2>&1 || true)"
echo "$TRUST_GPS" | grep -q "spanda-gps"
TRUST_FUSION="$(cargo run --manifest-path "${CLI}" -q -- trust spanda-fusion 2>&1 || true)"
echo "$TRUST_FUSION" | grep -q "spanda-fusion"

echo "== bundled spoof-check imports (no SPANDA_REGISTRY_URL) =="
SPOOF="$(cargo run --manifest-path "${CLI}" -q -- spoof-check "${ROOT}/examples/showcase/gps_spoofing/rover.sd" 2>&1 || true)"
echo "$SPOOF" | grep -q "spanda-gps package"
echo "$SPOOF" | grep -q "spanda-fusion package"

echo "bundled trust smoke ok"
