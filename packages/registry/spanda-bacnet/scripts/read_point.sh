#!/usr/bin/env bash
# BACnet read helper for SPANDA_BACNET_CMD — prints point value to stdout.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "${SCRIPT_DIR}/read_point.py" "$1" "$2"
