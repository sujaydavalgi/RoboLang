import type { SpandaType, UnitKind } from "./ast/nodes.js";

export type PhysicalCategory =
  | "scalar"
  | "distance"
  | "duration"
  | "velocity"
  | "acceleration"
  | "angle"
  | "angular_velocity"
  | "mass"
  | "force"
  | "power"
  | "voltage"
  | "current"
  | "temperature"
  | "pressure"
  | "frequency";

const KNOWN_DOMAIN_TYPES = new Set([
  "Mass", "Force", "Power", "Voltage", "Current", "Temperature", "Pressure", "Time",
  "Timestamp", "Interval", "Waypoint", "MotionCommand", "ControlSignal", "PIDConfig", "GpsFix",
  "ImuData", "AudioFrame", "Prompt", "Completion", "Embedding", "Token", "Context", "Memory",
  "Plan", "ReasoningTrace", "Agent", "Goal", "Task", "Skill", "Capability", "Intent", "Command",
  "Conversation", "Speech", "Gesture", "Emotion", "Feedback", "Approval", "Risk", "Hazard",
  "Confidence", "Prediction", "Probability",
  "SafetyConstraint", "Twin", "SimulationState", "Telemetry", "Replay", "Fault", "Scenario",
  "KnowledgeGraph", "Belief", "Observation", "WorldModel", "Policy", "Reward", "StateEstimate",
  "SensorFusion", "FusedObservation",
  "LLM", "VisionModel", "EmbeddingModel", "CameraFrame", "Image", "DepthImage", "PointCloud",
  "LidarScan", "Goal",
]);

function genericArity(name: string): number | undefined {
  switch (name) {
    case "Array":
    case "Set":
    case "Queue":
    case "Stack":
    case "Topic":
    case "Message":
    case "Endpoint":
      return 1;
    case "Map":
    case "Service":
    case "Tuple":
      return 2;
    case "Action":
      return 3;
    default:
      return undefined;
  }
}

export function resolveTypeName(name: string): SpandaType {
  const short = name.replace(/^std\./, "").split(".").pop() ?? name;
  switch (short) {
    case "Int":
    case "int":
      return { kind: "int" };
    case "Float":
    case "float":
      return { kind: "float" };
    case "Bool":
    case "bool":
      return { kind: "bool" };
    case "String":
    case "string":
      return { kind: "string" };
    case "Char":
    case "char":
      return { kind: "char" };
    case "Bytes":
    case "bytes":
      return { kind: "bytes" };
    case "Null":
    case "null":
      return { kind: "null" };
    case "Void":
    case "void":
      return { kind: "void" };
    case "Time":
      return { kind: "named", name: "Time" };
    case "Duration":
      return { kind: "number", unit: "ms" };
    case "Timestamp":
      return { kind: "named", name: "Timestamp" };
    case "Interval":
      return { kind: "named", name: "Interval" };
    case "Distance":
      return { kind: "number", unit: "m" };
    case "Velocity":
      return { kind: "velocity" };
    case "Acceleration":
      return { kind: "number", unit: "m/s²" };
    case "Angle":
      return { kind: "number", unit: "rad" };
    case "AngularVelocity":
      return { kind: "number", unit: "rad/s" };
    case "Mass":
    case "Force":
    case "Power":
    case "Voltage":
    case "Current":
    case "Temperature":
    case "Pressure":
      return { kind: "named", name: short };
    case "Point2D":
    case "Point3D":
    case "Vector2D":
    case "Vector3D":
    case "Quaternion":
    case "Pose":
      return { kind: "pose" };
    case "Transform":
      return { kind: "transform" };
    case "Trajectory":
    case "Path":
      return { kind: "trajectory" };
    case "Waypoint":
    case "MotionCommand":
    case "ControlSignal":
    case "PIDConfig":
      return { kind: "named", name: short };
    case "CameraFrame":
    case "Image":
    case "DepthImage":
    case "PointCloud":
    case "LidarScan":
    case "Scan":
      return { kind: "scan" };
    case "GpsFix":
    case "ImuData":
    case "AudioFrame":
    case "LLM":
    case "VisionModel":
    case "EmbeddingModel":
    case "Prompt":
    case "Completion":
    case "Embedding":
    case "Token":
    case "Context":
    case "Memory":
    case "Plan":
    case "ReasoningTrace":
    case "Agent":
    case "Goal":
    case "Task":
    case "Skill":
    case "Capability":
    case "Intent":
      return { kind: "named", name: short };
    case "ActionProposal":
      return { kind: "named", name: "ActionProposal" };
    case "SafeAction":
      return { kind: "named", name: "SafeAction" };
    case "Command":
    case "Conversation":
    case "Speech":
    case "Gesture":
    case "Emotion":
    case "Feedback":
    case "Approval":
    case "Confidence":
    case "Prediction":
    case "Probability":
    case "Risk":
    case "Hazard":
    case "SafetyConstraint":
    case "EmergencyStop":
    case "Twin":
    case "SimulationState":
    case "Telemetry":
    case "Replay":
    case "Fault":
    case "Scenario":
    case "KnowledgeGraph":
    case "Belief":
    case "Observation":
    case "WorldModel":
    case "Policy":
    case "Reward":
    case "StateEstimate":
    case "SensorFusion":
    case "FusedObservation":
      return { kind: "named", name: short };
    default:
      if (KNOWN_DOMAIN_TYPES.has(short)) {
        return { kind: "named", name: short };
      }
      throw new Error(`Unknown type '${short}'`);
  }
}

export function resolveGenericType(name: string, args: SpandaType[]): SpandaType {
  const base = name.split(".").pop() ?? name;
  const expected = genericArity(base);
  if (expected === undefined) {
    throw new Error(`Unknown generic type '${base}'`);
  }
  if (args.length !== expected) {
    throw new Error(`Type '${base}' expects ${expected} type argument(s), got ${args.length}`);
  }
  return { kind: "generic", name: base, typeArgs: args };
}

export function physicalCategory(ty: SpandaType): PhysicalCategory {
  switch (ty.kind) {
    case "int":
    case "float":
      return "scalar";
    case "number":
      switch (ty.unit) {
        case "m":
          return "distance";
        case "s":
        case "ms":
          return "duration";
        case "m/s":
          return "velocity";
        case "m/s²":
          return "acceleration";
        case "rad":
        case "deg":
          return "angle";
        case "rad/s":
          return "angular_velocity";
        case "Hz":
          return "frequency";
        default:
          return "scalar";
      }
    case "velocity":
      return "velocity";
    case "pose":
      return "distance";
    case "named":
      switch (ty.name) {
        case "Distance":
          return "distance";
        case "Duration":
        case "Time":
        case "Timestamp":
        case "Interval":
          return "duration";
        case "Velocity":
          return "velocity";
        case "Acceleration":
          return "acceleration";
        case "Angle":
        case "AngularVelocity":
          return "angular_velocity";
        case "Mass":
          return "mass";
        case "Force":
          return "force";
        case "Power":
          return "power";
        case "Voltage":
          return "voltage";
        case "Current":
          return "current";
        case "Temperature":
          return "temperature";
        case "Pressure":
          return "pressure";
        default:
          return "scalar";
      }
    default:
      return "scalar";
  }
}

type BinaryOp = "add" | "sub" | "lt" | "lte" | "gt" | "gte" | "eq" | "neq" | "mul" | "div" | "and" | "or";

const OP_MAP: Record<string, BinaryOp> = {
  "+": "add",
  add: "add",
  "-": "sub",
  sub: "sub",
  "<": "lt",
  lt: "lt",
  "<=": "lte",
  lte: "lte",
  ">": "gt",
  gt: "gt",
  ">=": "gte",
  gte: "gte",
  "==": "eq",
  eq: "eq",
  "!=": "neq",
  neq: "neq",
  "*": "mul",
  mul: "mul",
  "/": "div",
  div: "div",
  and: "and",
  or: "or",
};

export function binaryPhysicalOpAllowed(opLexeme: string, left: SpandaType, right: SpandaType): boolean {
  const op = OP_MAP[opLexeme];
  if (!op) return true;
  const catL = physicalCategory(left);
  const catR = physicalCategory(right);
  switch (op) {
    case "add":
    case "sub":
      if (catL === "scalar" && catR === "scalar") return true;
      return catL === catR && catL !== "scalar";
    case "lt":
    case "lte":
    case "gt":
    case "gte":
    case "eq":
    case "neq":
      return catL === catR;
    case "mul":
    case "div":
      return true;
    case "and":
    case "or":
      return left.kind === "bool" && right.kind === "bool";
    default:
      return true;
  }
}

export function isActionProposalType(ty: SpandaType): boolean {
  return ty.kind === "named" && ty.name === "ActionProposal";
}

export function isSafeActionType(ty: SpandaType): boolean {
  return ty.kind === "named" && ty.name === "SafeAction";
}

export function typeKindName(ty: SpandaType): string {
  switch (ty.kind) {
    case "generic":
      return "generic";
    case "enum_variant":
      return "enum_variant";
    default:
      return ty.kind;
  }
}
