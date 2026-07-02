#!/usr/bin/env bash
# Smoke Recovery Orchestrator (Phase 2) — crate, CLI, REST API, and gRPC parity.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
HEALING="${ROOT}/examples/showcase/self_healing/rover.sd"

resolve_spanda_bin() {
  if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
    echo "${SPANDA_BIN}"
    return
  fi
  local bases=("${ROOT}/target")
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    bases+=("${CARGO_TARGET_DIR}")
  fi
  for base in "${bases[@]}"; do
    for candidate in "${base}/debug/spanda" "${base}/release/spanda"; do
      if [[ -x "${candidate}" ]]; then
        echo "${candidate}"
        return
      fi
    done
  done
}

if resolved="$(resolve_spanda_bin)"; [[ -n "${resolved}" ]]; then
  export SPANDA_BIN="${resolved}"
  run_spanda() { "${SPANDA_BIN}" "$@"; }
else
  unset SPANDA_BIN
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== spanda-recovery crate tests =="
cargo test -p spanda-recovery --quiet

echo "== recovery REST API tests =="
cargo test -p spanda-api --test recovery_api_tests --quiet

echo "== recovery gRPC tests =="
cargo test -p spanda-api --test grpc_tests grpc_recovery --quiet

echo "== recovery orchestrator CLI =="
run_spanda recovery plan "$HEALING" --failure gps >/dev/null
run_spanda recovery simulate "$HEALING" --failure gps >/dev/null
run_spanda recovery dry-run "$HEALING" >/dev/null
run_spanda recovery validate "$HEALING" >/dev/null
run_spanda recovery playbooks >/dev/null
run_spanda recovery history >/dev/null
run_spanda recovery metrics "$HEALING" >/dev/null
run_spanda recovery graph "$HEALING" >/dev/null

echo "Recovery orchestrator smoke OK"
