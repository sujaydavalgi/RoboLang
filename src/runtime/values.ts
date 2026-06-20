import type { RuntimeValue } from "../runtime/interpreter.js";

export type { RuntimeValue } from "../runtime/interpreter.js";
import type { SafetyZoneRuntime } from "../safety/index.js";

export type PoseValue = { x: number; y: number; theta: number; z: number };
export type VelocityValue = { linear: number; angular: number };

export function runtimePose(x = 0, y = 0, theta = 0, z = 0): RuntimeValue {
  return { kind: "pose", x, y, theta, z };
}

export function runtimeVelocity(linear = 0, angular = 0): RuntimeValue {
  return { kind: "velocity", linear, angular };
}

export function runtimeTrajectory(waypoints: PoseValue[]): RuntimeValue {
  return { kind: "trajectory", waypoints };
}

export function poseFromState(state: { x: number; y: number; theta: number; z?: number }): RuntimeValue {
  return runtimePose(state.x, state.y, state.theta, state.z ?? 0);
}

export function velocityFromState(state: { linear: number; angular: number }): RuntimeValue {
  return runtimeVelocity(state.linear, state.angular);
}

export function getPoseFields(val: RuntimeValue): PoseValue | null {
  if (val.kind !== "pose") return null;
  return { x: val.x, y: val.y, theta: val.theta, z: val.z };
}

export function getVelocityFields(val: RuntimeValue): VelocityValue | null {
  if (val.kind !== "velocity") return null;
  return { linear: val.linear, angular: val.angular };
}

export function getTrajectoryWaypoints(val: RuntimeValue): PoseValue[] | null {
  if (val.kind !== "trajectory") return null;
  return val.waypoints;
}

export function getNumber(val: RuntimeValue, defaultVal = 0): number {
  return val.kind === "number" ? val.value : defaultVal;
}

export function getString(val: RuntimeValue, defaultVal = ""): string {
  return val.kind === "string" ? val.value : defaultVal;
}

export type TopicBinding = {
  name: string;
  messageType: string;
  topic: string;
};

export type ServiceBinding = {
  name: string;
  serviceType: string;
};

export type ActionBinding = {
  name: string;
  actionType: string;
};

export type ZoneBinding = SafetyZoneRuntime;
