#!/usr/bin/env bash
# 30-day field soak promotion gate for enterprise operations Stable tier.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOAK_FILE="${SPANDA_FIELD_SOAK_START_FILE:-$ROOT/.spanda/field-soak-start.txt}"
MIN_DAYS="${SPANDA_FIELD_SOAK_MIN_DAYS:-30}"

echo "== Field soak gate (min ${MIN_DAYS} days) =="

if [[ ! -f "$SOAK_FILE" ]]; then
  echo "missing soak start file: $SOAK_FILE" >&2
  echo "Create with: date -u +%Y-%m-%d > $SOAK_FILE" >&2
  exit 1
fi

START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
if ! date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
  if ! date -u -d "$START_DATE" "+%s" >/dev/null 2>&1; then
    echo "invalid soak start date in $SOAK_FILE: $START_DATE (expected YYYY-MM-DD)" >&2
    exit 1
  fi
  START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
else
  START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
fi

NOW_EPOCH="$(date -u "+%s")"
ELAPSED_DAYS=$(( (NOW_EPOCH - START_EPOCH) / 86400 ))

echo "Soak started: $START_DATE (${ELAPSED_DAYS} days elapsed)"
if (( ELAPSED_DAYS < MIN_DAYS )); then
  echo "field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
  exit 1
fi

echo "== Enterprise ops smoke =="
"$ROOT/scripts/enterprise_ops_smoke.sh"

echo "== Failover drill smoke =="
if [[ -x "$ROOT/scripts/failover_drill_smoke.sh" ]]; then
  "$ROOT/scripts/failover_drill_smoke.sh"
fi

echo "== OTA fleet soak =="
if [[ -x "$ROOT/scripts/ota_fleet_soak.sh" ]]; then
  SPANDA_OTA_FLEET_SOAK_QUICK=1 "$ROOT/scripts/ota_fleet_soak.sh"
fi

echo "Field soak gate passed (${ELAPSED_DAYS} days >= ${MIN_DAYS})."
