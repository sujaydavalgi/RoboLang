/**
 * Telemetry-store-backed implementation of the runtime telemetry boundary.
 * @module
 */

import type { TelemetrySink } from "./runtime/telemetry-sink.js";
import type { RuntimeValue } from "./runtime/interpreter.js";
import {
  beginRunSession,
  configureSessionPersist,
  endRunSession,
  recordHealthEvent,
  recordSensorReading,
  recordTaskHeartbeat,
  recordTopicPublish,
} from "./telemetry-store.js";

/** Full telemetry persistence delegating to telemetry-store. */
export const telemetryStoreSink: TelemetrySink = {
  configureSessionPersist(enabled: boolean): void {
    configureSessionPersist(enabled);
  },
  beginRunSession(source?: string): void {
    beginRunSession(source);
  },
  endRunSession(missionTracePath?: string, metrics?: unknown, timestampMs?: number): void {
    endRunSession(missionTracePath, metrics, timestampMs);
  },
  recordSensorReading(
    sensorId: string,
    sensorType: string,
    value: RuntimeValue,
    timestampMs: number,
    robotId?: string,
  ): void {
    recordSensorReading(sensorId, sensorType, value, timestampMs, robotId);
  },
  recordTopicPublish(
    robotId: string | undefined,
    topicPath: string,
    value: RuntimeValue,
    timestampMs: number,
  ): void {
    recordTopicPublish(robotId, topicPath, value, timestampMs);
  },
  recordHealthEvent(target: string, status: string, timestampMs: number): void {
    recordHealthEvent(target, status, timestampMs);
  },
  recordTaskHeartbeat(
    taskName: string,
    timestampMs: number,
    robotId?: string,
    historyIntervalMs?: number,
  ): void {
    recordTaskHeartbeat(taskName, timestampMs, robotId, historyIntervalMs);
  },
};

/** Create a telemetry sink backed by the persistent telemetry store. */
export function createTelemetryStoreSink(): TelemetrySink {
  return telemetryStoreSink;
}
