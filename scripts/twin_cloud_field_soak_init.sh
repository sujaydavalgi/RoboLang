#!/usr/bin/env bash
# Start the 30-day Twin Cloud SaaS field soak clock for Stable promotion.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOAK_FILE="${SPANDA_TWIN_CLOUD_FIELD_SOAK_START_FILE:-$ROOT/.spanda/twin-cloud-field-soak-start.txt}"
mkdir -p "$(dirname "$SOAK_FILE")"

if [[ -f "$SOAK_FILE" ]]; then
  echo "Twin Cloud field soak already started: $(tr -d '[:space:]' < "$SOAK_FILE")" >&2
  exit 1
fi

date -u +%Y-%m-%d > "$SOAK_FILE"
echo "Twin Cloud field soak started: $(cat "$SOAK_FILE")"
echo "Wrote $SOAK_FILE"
