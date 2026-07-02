#!/usr/bin/env bash
# LATER differentiation Stable tier promotion gate (all five pillars).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SOAK_FILE="${SPANDA_LATER_FIELD_SOAK_START_FILE:-$ROOT/.spanda/later-field-soak-start.txt}"
MIN_DAYS="${SPANDA_LATER_FIELD_SOAK_MIN_DAYS:-30}"

echo "== LATER differentiation stable promotion gate =="

if [[ "${SPANDA_LATER_SKIP_SOAK:-1}" != "1" ]]; then
  if [[ ! -f "$SOAK_FILE" ]]; then
    echo "missing soak file: $SOAK_FILE — run ./scripts/later_field_soak_init.sh" >&2
    exit 1
  fi
  START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
  if date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
    START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
  else
    START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
  fi
  ELAPSED_DAYS=$(( ($(date -u "+%s") - START_EPOCH) / 86400 ))
  if (( ELAPSED_DAYS < MIN_DAYS )); then
    echo "LATER field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
    exit 1
  fi
else
  echo "Skipping field soak (SPANDA_LATER_SKIP_SOAK=1)"
fi

if [[ "${SPANDA_LATER_SKIP_SMOKE:-0}" != "1" ]]; then
  ./scripts/later_differentiation_smoke.sh
  ./scripts/later_analytics_smoke.sh
else
  echo "Skipping smoke (SPANDA_LATER_SKIP_SMOKE=1)"
fi

echo "--- Topic guides (Stable) ---"
for doc in \
  docs/digital-mission-twin.md \
  docs/certification-packs.md \
  docs/mission-time-travel.md \
  docs/human-robot-teaming.md \
  docs/autonomous-governance.md
do
  test -f "$doc" || { echo "missing $doc" >&2; exit 1; }
  grep -q '^\*\*Status:\*\* Stable' "$doc" || {
    echo "$doc must declare Status: Stable" >&2
    exit 1
  }
done

echo ""
echo "LATER differentiation stable promotion gate passed."
