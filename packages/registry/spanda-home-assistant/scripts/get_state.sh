#!/usr/bin/env bash
# Home Assistant state helper for SPANDA_HOME_ASSISTANT_CMD — prints entity state to stdout.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "${SCRIPT_DIR}/get_state.py" "$1"
