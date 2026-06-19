import type { UnitKind, SynapseType } from "../ast/nodes.js";
import { allLibrarySensorTypes } from "../lib/registry.js";

export type TypeError = {
  message: string;
  line: number;
  column: number;
};

export class TypeCheckError extends Error {
  constructor(public errors: TypeError[]) {
    super(errors.map((e) => e.message).join("\n"));
    this.name = "TypeCheckError";
  }
}

export function unitsCompatible(a: UnitKind, b: UnitKind): boolean {
  if (a === b) return true;
  if (a === "none" || b === "none") return true;
  if ((a === "deg" && b === "rad") || (a === "rad" && b === "deg")) return true;
  return false;
}

export function resultUnitForBinary(
  op: string,
  left: SynapseType,
  right: SynapseType,
): SynapseType | null {
  if (op === "and" || op === "or") {
    if (left.kind === "bool" && right.kind === "bool") return { kind: "bool" };
    return null;
  }

  if (["<", "<=", ">", ">=", "==", "!="].includes(op)) {
    if (left.kind === "number" && right.kind === "number") {
      if (unitsCompatible(left.unit, right.unit)) return { kind: "bool" };
    }
    if (left.kind === "bool" && right.kind === "bool") return { kind: "bool" };
    return null;
  }

  if (op === "+" || op === "-") {
    if (left.kind === "number" && right.kind === "number") {
      if (unitsCompatible(left.unit, right.unit)) {
        return { kind: "number", unit: left.unit !== "none" ? left.unit : right.unit };
      }
    }
    return null;
  }

  if (op === "*" || op === "/") {
    if (left.kind === "number" && right.kind === "number") {
      return { kind: "number", unit: "none" };
    }
    return null;
  }

  return null;
}

export const MESSAGE_TYPES: Record<string, SynapseType> = {
  Velocity: { kind: "velocity" },
  Pose: { kind: "pose" },
  Scan: { kind: "scan" },
  String: { kind: "string" },
};

export const SERVICE_TYPES: Record<string, SynapseType> = {
  ResetCostmap: { kind: "named", name: "ResetCostmap" },
  ClearCostmap: { kind: "named", name: "ClearCostmap" },
  SetPose: { kind: "named", name: "SetPose" },
};

export const ACTION_TYPES: Record<string, SynapseType> = {
  NavigateTo: { kind: "named", name: "NavigateTo" },
  FollowPath: { kind: "named", name: "FollowPath" },
  PickObject: { kind: "named", name: "PickObject" },
};

export const SENSOR_TYPES: Record<string, SynapseType> = {
  Lidar: { kind: "named", name: "Lidar" },
  IMU: { kind: "named", name: "IMU" },
  GPS: { kind: "named", name: "GPS" },
  Camera: { kind: "named", name: "Camera" },
  AltitudeSensor: { kind: "named", name: "AltitudeSensor" },
  ForceTorque: { kind: "named", name: "ForceTorque" },
  ...Object.fromEntries(
    Object.entries(allLibrarySensorTypes()).map(([k, v]) => [k, v.roboType]),
  ),
};

export function getLibraryForSensorType(sensorType: string): string | undefined {
  return allLibrarySensorTypes()[sensorType]?.library;
}

function inferReadReturn(typeName: string): SynapseType {
  if (typeName.includes("Lidar") || typeName.includes("Velodyne") || typeName.includes("Hokuyo") || typeName.includes("Ydlidar") || typeName.includes("RealSense")) {
    return { kind: "scan" };
  }
  if (typeName.includes("BNO") || typeName.includes("LSM9") || typeName.includes("IMU")) {
    return { kind: "named", name: "IMUReading" };
  }
  if (typeName.includes("BMP") || typeName.includes("VL53") || typeName.includes("UWMF")) {
    return { kind: "number", unit: "m" };
  }
  return { kind: "void" };
}

export function mergeLibraryMethods(): void {
  for (const [typeName, info] of Object.entries(allLibrarySensorTypes())) {
    if (!BUILTIN_METHODS[typeName]) {
      BUILTIN_METHODS[typeName] = {
        read: { params: [], returns: inferReadReturn(info.roboType.name) },
        calibrate: { params: [], returns: { kind: "void" } },
      };
    }
  }
}

export const ACTUATOR_TYPES: Record<string, SynapseType> = {
  DifferentialDrive: { kind: "named", name: "DifferentialDrive" },
  RoboticArm: { kind: "named", name: "RoboticArm" },
  DroneRotors: { kind: "named", name: "DroneRotors" },
  Gripper: { kind: "named", name: "Gripper" },
};

export const AI_OUTPUT_TYPES: Record<string, SynapseType> = {
  Velocity: { kind: "velocity" },
  NavigationPolicy: { kind: "named", name: "NavigationPolicy" },
  Detections: { kind: "named", name: "Detections" },
  Classification: { kind: "named", name: "Classification" },
};

/** Maps sensor read types and runtime values to AI input type names. */
export const AI_INPUT_TYPES: Record<string, SynapseType> = {
  Scan: { kind: "scan" },
  Lidar: { kind: "scan" },
  Camera: { kind: "named", name: "CameraFrame" },
  IMU: { kind: "named", name: "IMUReading" },
  AltitudeSensor: { kind: "number", unit: "m" },
  Image: { kind: "named", name: "CameraFrame" },
};

export const BUILTIN_FUNCTIONS: Record<
  string,
  { namedParams: Record<string, SynapseType>; returns: SynapseType }
> = {
  pose: {
    namedParams: {
      x: { kind: "number", unit: "m" },
      y: { kind: "number", unit: "m" },
      theta: { kind: "number", unit: "rad" },
      z: { kind: "number", unit: "m" },
    },
    returns: { kind: "pose" },
  },
  velocity: {
    namedParams: {
      linear: { kind: "number", unit: "m/s" },
      angular: { kind: "number", unit: "rad/s" },
    },
    returns: { kind: "velocity" },
  },
  trajectory: {
    namedParams: {
      from: { kind: "pose" },
      to: { kind: "pose" },
      steps: { kind: "number", unit: "none" },
    },
    returns: { kind: "trajectory" },
  },
  transform: {
    namedParams: {
      from: { kind: "string" },
      to: { kind: "string" },
      pose: { kind: "pose" },
    },
    returns: { kind: "transform" },
  },
};

export const ROBOT_METHODS: Record<string, { params: SynapseType[]; returns: SynapseType }> = {
  pose: { params: [], returns: { kind: "pose" } },
  velocity: { params: [], returns: { kind: "velocity" } },
  in_zone: { params: [{ kind: "string" }], returns: { kind: "bool" } },
};

export const BUILTIN_METHODS: Record<
  string,
  Record<string, { params: SynapseType[]; namedParams?: Record<string, SynapseType>; returns: SynapseType }>
> = {
  Lidar: {
    read: { params: [], returns: { kind: "scan" } },
  },
  IMU: {
    read: { params: [], returns: { kind: "named", name: "IMUReading" } },
  },
  AltitudeSensor: {
    read: { params: [], returns: { kind: "number", unit: "m" } },
  },
  ForceTorque: {
    read: { params: [], returns: { kind: "named", name: "ForceTorqueReading" } },
  },
  DifferentialDrive: {
    drive: {
      params: [],
      namedParams: {
        linear: { kind: "number", unit: "m/s" },
        angular: { kind: "number", unit: "rad/s" },
      },
      returns: { kind: "void" },
    },
    follow: {
      params: [],
      namedParams: {
        path: { kind: "trajectory" },
      },
      returns: { kind: "void" },
    },
    stop: { params: [], returns: { kind: "void" } },
  },
  RoboticArm: {
    move_to: {
      params: [],
      namedParams: {
        x: { kind: "number", unit: "m" },
        y: { kind: "number", unit: "m" },
        z: { kind: "number", unit: "m" },
      },
      returns: { kind: "void" },
    },
    grip: { params: [], returns: { kind: "void" } },
    release: { params: [], returns: { kind: "void" } },
  },
  DroneRotors: {
    set_thrust: {
      params: [],
      namedParams: {
        thrust: { kind: "number", unit: "none" },
      },
      returns: { kind: "void" },
    },
    hover: { params: [], returns: { kind: "void" } },
  },
  Gripper: {
    close: { params: [], returns: { kind: "void" } },
    open: { params: [], returns: { kind: "void" } },
  },
  Scan: {
    nearest_distance: { params: [], returns: { kind: "number", unit: "m" } },
  },
  IMUReading: {
    yaw: { params: [], returns: { kind: "number", unit: "rad" } },
  },
  ForceTorqueReading: {
    force: { params: [], returns: { kind: "number", unit: "none" } },
  },
};

export const SCAN_PROPERTIES: Record<string, SynapseType> = {
  nearest_distance: { kind: "number", unit: "m" },
};

export const OBJECT_PROPERTIES: Record<string, Record<string, SynapseType>> = {
  IMUReading: { yaw: { kind: "number", unit: "rad" }, roll: { kind: "number", unit: "rad" }, pitch: { kind: "number", unit: "rad" } },
  ForceTorqueReading: { force: { kind: "number", unit: "none" } },
  GPSReading: { lat: { kind: "number", unit: "none" }, lon: { kind: "number", unit: "none" } },
  NavigationPolicy: {
    linear: { kind: "number", unit: "m/s" },
    angular: { kind: "number", unit: "rad/s" },
  },
  Detections: {
    count: { kind: "number", unit: "none" },
    nearest_distance: { kind: "number", unit: "m" },
    label: { kind: "string" },
  },
  Classification: {
    label: { kind: "string" },
    confidence: { kind: "number", unit: "none" },
  },
};

export const POSE_PROPERTIES: Record<string, SynapseType> = {
  x: { kind: "number", unit: "m" },
  y: { kind: "number", unit: "m" },
  theta: { kind: "number", unit: "rad" },
  z: { kind: "number", unit: "m" },
};

export const VELOCITY_PROPERTIES: Record<string, SynapseType> = {
  linear: { kind: "number", unit: "m/s" },
  angular: { kind: "number", unit: "rad/s" },
};

mergeLibraryMethods();
