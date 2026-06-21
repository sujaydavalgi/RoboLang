import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";

const examplesDir = join(import.meta.dirname, "..", "examples", "connectivity");

describe("connectivity parser", () => {
  it("parses requires_connectivity and hardware connectivity list", () => {
    const source = readFileSync(join(examplesDir, "connectivity_hardware_verify.sd"), "utf8");
    const program = parse(tokenize(source));
    expect(program.requiresConnectivity?.channels).toEqual([
      ["gps", "required"],
      ["wifi", "optional"],
      ["cellular", "required"],
    ]);
    expect(program.requiresConnectivity?.latencyMsMax).toBe(100);
    expect(program.hardwareProfiles[0]?.connectivity).toEqual([
      "WiFi6",
      "Bluetooth5",
      "LTE",
      "GPS",
    ]);
  });

  it("parses geofence and geofence exit trigger", () => {
    const source = readFileSync(join(examplesDir, "geofence_safety.sd"), "utf8");
    const program = parse(tokenize(source));
    expect(program.geofences).toHaveLength(1);
    expect(program.geofences[0]?.name).toBe("SafeZone");
    expect(program.geofences[0]?.centerLat).toBeCloseTo(30.2672);
    expect(program.geofences[0]?.centerLon).toBeCloseTo(-97.7431);
    expect(program.robots[0]?.eventHandlers[0]?.eventName).toBe("geofence:SafeZone:exited");
  });

  it("parses connectivity_policy failover block", () => {
    const source = readFileSync(join(examplesDir, "wifi_lte_failover.sd"), "utf8");
    const program = parse(tokenize(source));
    expect(program.connectivityPolicies[0]?.preferred).toBe("wifi");
    expect(program.connectivityPolicies[0]?.fallback).toBe("cellular");
    expect(program.connectivityPolicies[0]?.emergency).toBe("bluetooth");
    expect(program.robots[0]?.eventHandlers[0]?.eventName).toBe("network.disconnected");
  });

  it("parses bluetooth config and ble_service", () => {
    const source = readFileSync(join(examplesDir, "bluetooth_sensor.sd"), "utf8");
    const program = parse(tokenize(source));
    expect(program.bleServices[0]?.uuid).toBe("180D");
    expect(program.robots[0]?.bluetooth?.pairMode).toBe("trusted_only");
    expect(program.robots[0]?.eventHandlers[0]?.eventName).toBe("bluetooth.device_connected");
  });

  it("parses GPS dot-notation triggers", () => {
    const source = readFileSync(join(examplesDir, "gps_navigation.sd"), "utf8");
    const program = parse(tokenize(source));
    const names = program.robots[0]?.eventHandlers.map((h) => h.eventName) ?? [];
    expect(names).toContain("gps.fix");
    expect(names).toContain("gps.lost");
    expect(names).toContain("gps.acquired");
  });
});
