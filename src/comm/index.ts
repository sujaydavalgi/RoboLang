/**
 * index module (comm/index.ts).
 * @module
 */

import type { RuntimeValue } from "../runtime/values.js";
import type { TransportKind, DiscoverFilter, DiscoverTarget } from "./decls.js";

export type {
  AgentChannelDecl,
  BusDecl,
  DiscoverFilter,
  DiscoverTarget,
  MessageDecl,
  MessageSchema,
  PeerRobotDecl,
  DeviceDecl,
  QosDecl,
  QosReliability,
  TopicRole,
  TransportKind,
  TwinSyncDecl,
} from "./decls.js";
export {
  COMM_CAPABILITIES,
  MessageRegistry,
  isCommCapability,
  transportAsStr,
  transportFromIdent,
} from "./decls.js";

export type CommEnvelope = {
  value: RuntimeValue;
  sourceId: string | null;
};

export type PublishedCommMessage = {
  topicPath: string;
  messageType: string;
  value: RuntimeValue;
  transport: TransportKind;
  sourceId?: string | null;
};

export type SimNetworkConfig = {
  delayMs: number;
  packetLoss: number;
};

export class InMemoryCommBus {
  private subscriptions = new Map<string, string[]>();
  private buffers = new Map<string, CommEnvelope[]>();
  private published: PublishedCommMessage[] = [];
  private discoveredRobots = ["RoverA", "RoverB"];
  private discoveredAgents = ["Vision", "Planner", "Navigator"];
  private discoveredDevices = ["Camera", "IMU", "Lidar"];
  private network: SimNetworkConfig = { delayMs: 0, packetLoss: 0 };
  private faults: string[] = [];

  publish(
    topicPath: string,
    messageType: string,
    value: RuntimeValue,
    transport: TransportKind,
    sourceId?: string | null,
  ): void {
    // Publish a message to subscribers and record it in the published log.
    if (this.faults.includes("NetworkOutage")) return;
    if (this.network.packetLoss > 0) {
      const hash = topicPath.length + messageType.length;
      if (((hash * 0.13) % 1) < this.network.packetLoss) return;
    }
    this.published.push({ topicPath, messageType, value, transport, sourceId: sourceId ?? null });
    const buf = this.buffers.get(topicPath);
    if (buf) buf.push({ value, sourceId: sourceId ?? null });
  }

  pushInbound(topicPath: string, value: RuntimeValue, sourceId?: string | null): void {
    // Push an inbound message with optional publisher identity into the topic buffer.
    if (!this.buffers.has(topicPath)) this.buffers.set(topicPath, []);
    this.buffers.get(topicPath)!.push({ value, sourceId: sourceId ?? null });
  }

  subscribe(topicPath: string, handler: string): void {
    // Register a handler subscription for the given topic path.
    const subs = this.subscriptions.get(topicPath) ?? [];
    subs.push(handler);
    this.subscriptions.set(topicPath, subs);
    if (!this.buffers.has(topicPath)) this.buffers.set(topicPath, []);
  }

  receiveEnvelope(topicPath: string): CommEnvelope | null {
    // Receive the next inbound envelope including publisher source_id when present.
    const buf = this.buffers.get(topicPath);
    return buf?.shift() ?? null;
  }

  receive(topicPath: string): RuntimeValue | null {
    // Receive the next inbound payload value from the topic buffer.
    return this.receiveEnvelope(topicPath)?.value ?? null;
  }

  callService(serviceType: string): RuntimeValue {
    // Simulate a successful service call response for the given service type.
    return {
      kind: "object",
      typeName: serviceType,
      fields: { ok: { kind: "bool", value: true } },
    };
  }

  sendAction(actionType: string): RuntimeValue {
    // Simulate a successful action execution result for the given action type.
    return {
      kind: "object",
      typeName: actionType,
      fields: { success: { kind: "bool", value: true } },
    };
  }

  discover(target: DiscoverTarget, filter: DiscoverFilter): string[] {
    // Discover robots, agents, or devices optionally filtered by capability substring.
    const base =
      target === "robots"
        ? this.discoveredRobots
        : target === "agents"
          ? this.discoveredAgents
          : this.discoveredDevices;
    if (filter.capability) {
      const cap = filter.capability.toLowerCase();
      return base.filter((n) => n.toLowerCase().includes(cap));
    }
    return [...base];
  }

  registerRobot(name: string): void {
    this.discoveredRobots.push(name);
  }

  registerAgent(name: string): void {
    this.discoveredAgents.push(name);
  }

  registerDevice(name: string): void {
    this.discoveredDevices.push(name);
  }

  publishPeer(
    peer: string,
    topic: string,
    value: RuntimeValue,
    transport: TransportKind,
    sourceId?: string | null,
  ): void {
    const path = `/${peer}/${topic}`;
    this.publish(path, "PeerMessage", value, transport, sourceId ?? peer);
  }

  publishedMessages(): PublishedCommMessage[] {
    return [...this.published];
  }

  injectFault(fault: string): void {
    this.faults.push(fault);
  }

  activeFaults(): string[] {
    // Return injected simulation faults currently affecting the comm bus.
    return [...this.faults];
  }

  subscriptionPaths(): string[] {
    // List topic paths with active in-memory subscriptions.
    return [...this.subscriptions.keys()];
  }
}

export function estimateTopicBandwidthMbps(rateHz: number, messageSizeBytes: number): number {
  // Estimate topic bandwidth in megabits per second from rate and message size.
  return (rateHz * messageSizeBytes * 8) / 1_000_000;
}

export function defaultMessageSize(messageType: string): number {
  // Return a default serialized message size in bytes for bandwidth estimation.
  switch (messageType) {
    case "Scan":
    case "LidarScan":
    case "LidarReading":
      return 64_000;
    case "Pose":
    case "Velocity":
      return 128;
    case "PathPlan":
    case "NavigationFeedback":
      return 4_096;
    default:
      return 512;
  }
}
