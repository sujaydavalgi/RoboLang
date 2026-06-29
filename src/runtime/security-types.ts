/**
 * Minimal security types for the language runtime layer (no platform_services imports).
 * @module
 */

export type TrustLevel = "untrusted" | "restricted" | "trusted" | "certified";

export type TrustBoundaryKind =
  | "robot_internal"
  | "robot_to_robot"
  | "robot_to_cloud"
  | "operator_to_robot";

export type EncryptionMode = "none" | "optional" | "required";
export type AuthenticationMode = "none" | "signed" | "mutual";
export type IntegrityMode = "none" | "required";

export type SecurePolicyConfig = {
  signed: boolean;
  minTrust: TrustLevel | null;
  requires: string[];
  encryption: EncryptionMode;
  authentication: AuthenticationMode;
  integrity: IntegrityMode;
  trustedSources: string[];
  rejectUntrusted: boolean;
};

export type SecretSource =
  | { source: "env"; var: string }
  | { source: "literal"; value: string }
  | { source: "file"; path: string };

export function parseTrustLevel(level: string): TrustLevel | null {
  switch (level.toLowerCase()) {
    case "untrusted":
      return "untrusted";
    case "restricted":
      return "restricted";
    case "trusted":
      return "trusted";
    case "certified":
      return "certified";
    default:
      return null;
  }
}

export function boundaryForTransportName(transport: string): TrustBoundaryKind | null {
  switch (transport) {
    case "local":
    case "sim":
    case "ble":
      return "robot_internal";
    case "ros2":
    case "dds":
    case "mqtt":
      return "robot_to_robot";
    case "websocket":
      return "operator_to_robot";
    case "wifi":
    case "cellular":
      return "robot_to_cloud";
    default:
      return null;
  }
}

export function parseTrustBoundary(name: string): TrustBoundaryKind {
  switch (name) {
    case "robot_internal":
      return "robot_internal";
    case "robot_to_robot":
      return "robot_to_robot";
    case "robot_to_cloud":
      return "robot_to_cloud";
    case "operator_to_robot":
      return "operator_to_robot";
    default:
      throw new Error(`unknown trust boundary '${name}'`);
  }
}
