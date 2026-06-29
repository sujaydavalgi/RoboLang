/**
 * Runtime health polling mirror for telemetry persistence.
 * @module
 */

import type { Program } from "../ast/nodes.js";
import type { TelemetrySink } from "./telemetry-sink.js";
import { noopTelemetrySink } from "./telemetry-sink.js";

export type HealthPollState = {
  lastOverall?: string;
  lastChecks: Map<string, string>;
};

export function createHealthPollState(): HealthPollState {
  return { lastChecks: new Map() };
}

/**
 * Record overall and per-check health transitions from runtime fault state.
 */
export function pollRuntimeHealthChanges(
  program: Program | null,
  injectedFaults: ReadonlySet<string>,
  simTimeMs: number,
  state: HealthPollState,
  telemetrySink: TelemetrySink = noopTelemetrySink,
): void {
  if (!program?.healthChecks?.length) {
    return;
  }

  const overall = injectedFaults.size > 0 ? "Degraded" : "Healthy";
  if (state.lastOverall !== overall) {
    telemetrySink.recordHealthEvent("overall", overall, simTimeMs);
    state.lastOverall = overall;
  }

  for (const check of program.healthChecks) {
    const status = injectedFaults.size > 0 ? "Degraded" : "Healthy";
    const key = `${check.target}:${check.name}`;
    if (state.lastChecks.get(key) === status) {
      continue;
    }
    telemetrySink.recordHealthEvent(key, status, simTimeMs);
    state.lastChecks.set(key, status);
  }
}
