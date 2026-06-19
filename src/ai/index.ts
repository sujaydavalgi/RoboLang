import type { RuntimeValue } from "../runtime/interpreter.js";

export type AiInputBinding = {
  name: string;
  typeName: string;
};

export type AiModelRuntime = {
  name: string;
  outputType: string;
  path: string;
  library: string | null;
  inputs: AiInputBinding[];
};

export type InferenceContext = {
  pose: { x: number; y: number; theta: number; z: number };
  inputs: Record<string, RuntimeValue>;
};

function getScanDistance(value: RuntimeValue | undefined): number | null {
  if (!value) return null;
  if (value.kind === "scan") return value.nearestDistance;
  if (value.kind === "object" && value.typeName === "Detections") {
    const nearest = value.fields.nearest_distance;
    if (nearest?.kind === "number") return nearest.value;
  }
  return null;
}

function getNumberField(value: RuntimeValue | undefined, field: string): number | null {
  if (value?.kind === "object") {
    const f = value.fields[field];
    if (f?.kind === "number") return f.value;
  }
  if (value?.kind === "number" && field === "value") return value.value;
  return null;
}

export function runInference(model: AiModelRuntime, ctx: InferenceContext): RuntimeValue {
  switch (model.outputType) {
    case "Velocity":
      return inferNavigationPolicy(ctx, "Velocity");
    case "NavigationPolicy":
      return inferNavigationPolicy(ctx, "NavigationPolicy");
    case "Detections":
      return inferDetections(ctx);
    case "Classification":
      return inferClassification(ctx);
    default:
      return { kind: "void" };
  }
}

function inferNavigationPolicy(ctx: InferenceContext, outputType: string): RuntimeValue {
  const scanDist =
    getScanDistance(ctx.inputs.scan) ??
    getScanDistance(ctx.inputs.lidar) ??
    getScanDistance(Object.values(ctx.inputs)[0]);

  let linear = 0.6;
  let angular = 0.0;

  if (scanDist !== null) {
    if (scanDist < 0.4) {
      linear = 0.0;
      angular = 0.5;
    } else if (scanDist < 0.8) {
      linear = 0.2;
      angular = 0.3;
    } else {
      linear = Math.min(0.8, scanDist * 0.4);
    }
  }

  if (outputType === "Velocity") {
    return { kind: "velocity", linear, angular };
  }

  return {
    kind: "object",
    typeName: "NavigationPolicy",
    fields: {
      linear: { kind: "number", value: linear, unit: "m/s" },
      angular: { kind: "number", value: angular, unit: "rad/s" },
    },
  };
}

function inferDetections(ctx: InferenceContext): RuntimeValue {
  const scanDist =
    getScanDistance(ctx.inputs.scan) ??
    getScanDistance(ctx.inputs.image) ??
    getScanDistance(Object.values(ctx.inputs)[0]);
  const nearest = scanDist ?? 5.0;
  const count = nearest < 1.0 ? 2 : nearest < 3.0 ? 1 : 0;

  return {
    kind: "object",
    typeName: "Detections",
    fields: {
      count: { kind: "number", value: count, unit: "none" },
      nearest_distance: { kind: "number", value: nearest, unit: "m" },
      label: { kind: "string", value: count > 0 ? "obstacle" : "clear" },
    },
  };
}

function inferClassification(ctx: InferenceContext): RuntimeValue {
  const alt = getNumberField(ctx.inputs.altitude, "value") ?? getNumberField(ctx.inputs.reading, "value");
  let label = "unknown";
  let confidence = 0.5;

  if (alt !== null) {
    if (alt < 1.0) {
      label = "low_altitude";
      confidence = 0.92;
    } else if (alt < 3.0) {
      label = "cruise";
      confidence = 0.88;
    } else {
      label = "high_altitude";
      confidence = 0.85;
    }
  } else {
    const scanDist = getScanDistance(ctx.inputs.scan) ?? getScanDistance(Object.values(ctx.inputs)[0]);
    if (scanDist !== null) {
      label = scanDist < 0.5 ? "blocked" : "open";
      confidence = scanDist < 0.5 ? 0.95 : 0.8;
    }
  }

  return {
    kind: "object",
    typeName: "Classification",
    fields: {
      label: { kind: "string", value: label },
      confidence: { kind: "number", value: confidence, unit: "none" },
    },
  };
}
