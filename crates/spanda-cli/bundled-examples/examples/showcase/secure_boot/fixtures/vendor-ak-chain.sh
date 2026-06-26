#!/usr/bin/env bash
# Vendor TPM SDK adapter with optional remote AK cert chain for attestation demos.
# Use with: SPANDA_TPM_BACKEND=vendor SPANDA_TPM_VENDOR_SDK=.../vendor-ak-chain.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
contract="${SPANDA_ATTESTATION_CONTRACT:-trust.jetson}"
detail="vendor sdk quote with ak chain for ${contract}"

python3 - "${ROOT}/trust-store/anchor.pem" "$detail" <<'PY'
import json
import pathlib
import sys

anchor_path = pathlib.Path(sys.argv[1])
detail = sys.argv[2]
anchor = anchor_path.read_text().strip()
print(json.dumps({
    "attested": True,
    "boot_state": "verified",
    "score": 97,
    "detail": detail,
    "ak_cert_chain": [anchor],
}))
PY
