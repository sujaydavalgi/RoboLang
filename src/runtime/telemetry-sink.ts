/**
 * Injectable telemetry persistence boundary for interpreter mission recording.
 * @module
 */

import type { RuntimeValue } from "./interpreter.js";

/** Extension points for telemetry persistence during interpreter runs. */
export interface TelemetrySink {
  configureSessionPersist(enabled: boolean): void;
  beginRunSession(source?: string): void;
  endRunSession(
    missionTracePath?: string,
    metrics?: unknown,
    timestampMs?: number,
  ): void;
  recordSensorReading(
    sensorId: string,
    sensorType: string,
    value: RuntimeValue,
    timestampMs: number,
    robotId?: string,
  ): void;
  recordTopicPublish(
    robotId: string | undefined,
    topicPath: string,
    value: RuntimeValue,
    timestampMs: number,
  ): void;
  recordHealthEvent(target: string, status: string, timestampMs: number): void;
  recordTaskHeartbeat(
    taskName: string,
    timestampMs: number,
    robotId?: string,
    historyIntervalMs?: number,
  ): void;
}

/** No-op telemetry sink for tests and runs without persistence. */
export const noopTelemetrySink: TelemetrySink = {
  configureSessionPersist() {},
  beginRunSession() {},
  endRunSession() {},
  recordSensorReading() {},
  recordTopicPublish() {},
  recordHealthEvent() {},
  recordTaskHeartbeat() {},
};

/** Default no-op telemetry sink for direct interpreter use without telemetry store. */
export function defaultTelemetrySink(): TelemetrySink {
  return noopTelemetrySink;
}
