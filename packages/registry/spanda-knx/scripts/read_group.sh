#!/usr/bin/env bash
# KNX read helper for SPANDA_KNX_CMD — prints group value to stdout.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "${SCRIPT_DIR}/read_group.py" "$1"
