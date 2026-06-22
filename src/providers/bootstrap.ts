/**
 * Project-scoped provider bootstrap mirrored from Rust `providers/bootstrap.rs`.
 * @module
 */

import type { TransportKind } from "../comm/index.js";
import {
  createTransportStub,
  defaultTransportSecurity,
  type RoutingCommBus,
  type TransportConfig,
  TlsTransportSession,
} from "../transport/index.js";
import { ProviderRegistry, transportRegistryKey } from "./registry.js";

/** Map a transport kind to the official package that backs it when installed. */
export function officialPackageForTransport(kind: TransportKind): string | null {
  switch (kind) {
    case "ros2":
      return "spanda-ros2";
    case "mqtt":
      return "spanda-mqtt";
    case "dds":
      return "spanda-dds";
    case "websocket":
      return "spanda-ble";
    default:
      return null;
  }
}

function registerTransportStub(
  registry: ProviderRegistry,
  packageName: string,
  kind: TransportKind,
): void {
  registry.registerTransport(transportRegistryKey(packageName), createTransportStub(kind));
}

/** Build a provider registry from installed official package names. */
export function bootstrapProvidersForPackages(packageNames: readonly string[]): ProviderRegistry {
  const registry = new ProviderRegistry();
  registry.setOfficialPackages([...packageNames]);
  registry.grantCapability("mqtt.publish");
  registry.grantCapability("mqtt.subscribe");
  registry.grantCapability("comm.ros2.publish");
  registry.grantCapability("comm.ros2.subscribe");

  const names = new Set(packageNames);
  const includeAll = names.size === 0;

  if (includeAll || names.has("spanda-mqtt")) {
    registerTransportStub(registry, "spanda-mqtt", "mqtt");
  }
  if (includeAll || names.has("spanda-ros2")) {
    registerTransportStub(registry, "spanda-ros2", "ros2");
  }
  if (names.has("spanda-dds")) {
    registry.grantCapability("dds.publish");
    registry.grantCapability("dds.subscribe");
    registerTransportStub(registry, "spanda-dds", "dds");
  }
  if (names.has("spanda-ble") || names.has("spanda-wifi")) {
    registry.grantCapability("connectivity.wifi");
    registry.grantCapability("connectivity.ble");
    registerTransportStub(registry, "spanda-ble", "websocket");
  }
  if (names.has("spanda-gps")) {
    registry.grantCapability("positioning.read");
  }
  if (names.has("spanda-nav") || names.has("spanda-nav2")) {
    registry.grantCapability("navigation.plan");
  }
  if (names.has("spanda-slam")) {
    registry.grantCapability("slam.localize");
    registry.grantCapability("slam.map");
  }
  if (names.has("spanda-fleet")) {
    registry.grantCapability("fleet.orchestrate");
  }
  if (names.has("spanda-ota")) {
    registry.grantCapability("deploy.rollout");
  }
  if (names.has("spanda-ledger")) {
    registry.grantCapability("audit.append");
  }
  if (names.has("spanda-cloud")) {
    registry.grantCapability("cloud.invoke");
  }

  return registry;
}

/** Register default compatibility-shim providers when no project manifest is available. */
export function bootstrapDefaultProviders(): ProviderRegistry {
  return bootstrapProvidersForPackages([]);
}

function connectRegistryTransport(
  commBus: RoutingCommBus,
  registry: ProviderRegistry,
  kind: TransportKind,
  packageName: string,
  config: TransportConfig,
): void {
  const key = transportRegistryKey(packageName);
  const connected = registry.withTransport(key, (provider) => {
    provider.connect(config);
    return true;
  });
  if (connected) {
    commBus.markRegistryBacked(kind, key);
  }
}

/** Connect comm-bus transports through installed official package providers. */
export function syncCommBusForOfficialPackages(
  commBus: RoutingCommBus,
  registry: ProviderRegistry,
): void {
  commBus.clearRegistryBacked();
  const base: TransportConfig = {
    security: defaultTransportSecurity(),
    tls: new TlsTransportSession(),
  };
  for (const name of registry.officialPackages()) {
    switch (name) {
      case "spanda-ros2":
        connectRegistryTransport(commBus, registry, "ros2", name, base);
        break;
      case "spanda-mqtt":
        connectRegistryTransport(commBus, registry, "mqtt", name, {
          ...base,
          brokerUrl: "mqtt://localhost:1883",
          clientId: "spanda",
        });
        break;
      case "spanda-dds":
        connectRegistryTransport(commBus, registry, "dds", name, {
          ...base,
          domainId: 0,
        });
        break;
      case "spanda-ble":
      case "spanda-wifi":
        connectRegistryTransport(commBus, registry, "websocket", "spanda-ble", {
          ...base,
          brokerUrl: "ws://localhost:9090",
        });
        break;
      default:
        break;
    }
  }
}
