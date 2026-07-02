#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOAK_FILE="${SPANDA_TRUST_GRAPH_FIELD_SOAK_START_FILE:-$ROOT/.spanda/trust-graph-field-soak-start.txt}"
mkdir -p "$(dirname "$SOAK_FILE")"
if [[ -f "$SOAK_FILE" ]]; then echo "Trust graph soak already started" >&2; exit 1; fi
date -u +%Y-%m-%d > "$SOAK_FILE"
