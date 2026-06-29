/**
 * Comm declaration AST types and compile-time message registry (compiler layer).
 * @module
 */

import type { FieldDecl, StructDecl } from "../foundations.js";
import type { SpandaType, Span } from "../ast/nodes.js";

export type TransportKind = "local" | "ros2" | "mqtt" | "dds" | "websocket" | "sim";

export function transportFromIdent(s: string): TransportKind | null {
  // Map a transport identifier string to a TransportKind enum value.
  switch (s) {
    case "local":
      return "local";
    case "ros2":
      return "ros2";
    case "mqtt":
      return "mqtt";
    case "dds":
      return "dds";
    case "websocket":
      return "websocket";
    case "sim":
      return "sim";
    default:
      return null;
  }
}

export function transportAsStr(t: TransportKind): string {
  // Return the canonical string form of a transport kind.
  return t;
}

export type QosReliability = "reliable" | "best_effort";
export type TopicRole = "publish" | "subscribe" | "both";

export type QosDecl = {
  reliability: QosReliability | null;
  rateHz: number | null;
  deadlineMs: number | null;
  history: string | null;
  span: Span;
};

export type MessageDecl = {
  kind: "MessageDecl";
  name: string;
  fields: FieldDecl[];
  version: number | null;
  span: Span;
};

export type MessageSchema = {
  name: string;
  fields: [string, string][];
  version: number | null;
};

export type BusDecl = {
  kind: "BusDecl";
  name: string;
  transport: TransportKind;
  transportName?: string | null;
  brokerUrl?: string | null;
  encryption?: string | null;
  authentication?: string | null;
  integrity?: string | null;
  span: Span;
};

export type PeerRobotDecl = {
  kind: "PeerRobotDecl";
  name: string;
  span: Span;
};

export type DeviceDecl = {
  kind: "DeviceDecl";
  name: string;
  deviceType: string;
  span: Span;
};

export type AgentChannelDecl = {
  kind: "AgentChannelDecl";
  fromAgent: string;
  toAgent: string;
  messageType: string;
  span: Span;
};

export type TwinSyncDecl = {
  kind: "TwinSyncDecl";
  telemetry: boolean;
  replay: boolean;
  faults: boolean;
  events: boolean;
  span: Span;
};

export type DiscoverTarget = "robots" | "agents" | "devices";

export type DiscoverFilter = {
  capability: string | null;
};

export const COMM_CAPABILITIES = ["subscribe", "publish", "call", "execute", "discover"] as const;

export function isCommCapability(action: string): boolean {
  // Return true when the action name is a built-in comm capability.
  return (COMM_CAPABILITIES as readonly string[]).includes(action);
}

export class MessageRegistry {
  private schemas = new Map<string, MessageSchema>();
  private builtin = new Set(["Velocity", "Pose", "Scan", "String"]);

  static new(): MessageRegistry {
    // Construct an empty message registry with built-in message types.
    return new MessageRegistry();
  }

  register(decl: MessageDecl): void {
    // Register a message declaration schema for type checking.
    this.schemas.set(decl.name, {
      name: decl.name,
      fields: decl.fields.map((f) => [f.name, f.typeName]),
      version: decl.version,
    });
  }

  static fromProgram(messages: MessageDecl[], structs: StructDecl[]): MessageRegistry {
    // Build a registry from program message and struct declarations.
    const reg = MessageRegistry.new();
    for (const msg of messages) reg.register(msg);
    for (const s of structs) {
      reg.schemas.set(s.name, {
        name: s.name,
        fields: s.fields.map((f) => [f.name, f.typeName]),
        version: null,
      });
    }
    return reg;
  }

  isKnown(name: string): boolean {
    // Return true when the message type is built-in or registered.
    return this.builtin.has(name) || this.schemas.has(name);
  }

  resolveType(name: string): SpandaType | null {
    // Resolve a message type name to a SpandaType for the checker.
    switch (name) {
      case "Velocity":
        return { kind: "velocity" };
      case "Pose":
        return { kind: "pose" };
      case "Scan":
        return { kind: "scan" };
      case "String":
        return { kind: "string" };
      case "Command":
      case "Conversation":
      case "Feedback":
      case "Approval":
      case "Intent":
      case "SafeMessage":
      case "VerifiedMessage":
      case "TrustedSource":
      case "ActionProposal":
      case "SafeAction":
      case "CommandMessage":
      case "BatteryRequest":
      case "BatteryStatus":
      case "NavigationFeedback":
      case "NavigationResult":
      case "LidarReading":
      case "LidarScan":
      case "Timestamp":
      case "PathPlan":
        return { kind: "named", name };
      default:
        if (this.schemas.has(name)) return { kind: "named", name };
        return null;
    }
  }
}
