import { describe, it, expect } from "vitest";
import {
  bootstrapProvidersForPackages,
  syncCommBusForOfficialPackages,
} from "../src/providers/bootstrap.js";
import { RoutingCommBus } from "../src/transport/index.js";

describe("provider comm-bus routing (TypeScript parity)", () => {
  it("sync_comm_bus routes mqtt through provider registry", () => {
    const registry = bootstrapProvidersForPackages(["spanda-mqtt"]);
    const commBus = new RoutingCommBus();
    commBus.attachProviderRegistry(registry);
    syncCommBusForOfficialPackages(commBus, registry);
    expect(commBus.isRegistryBacked("mqtt")).toBe(true);
    expect(commBus.isRegistryBacked("ros2")).toBe(false);
  });

  it("sync_comm_bus routes ros2 when official package installed", () => {
    const registry = bootstrapProvidersForPackages(["spanda-ros2"]);
    const commBus = new RoutingCommBus();
    commBus.attachProviderRegistry(registry);
    syncCommBusForOfficialPackages(commBus, registry);
    expect(commBus.isRegistryBacked("ros2")).toBe(true);
  });
});
