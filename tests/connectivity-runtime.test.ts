import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { applyGpsPositionFaults } from "../src/connectivity-positioning.js";

describe("connectivity runtime", () => {
  it("failovers link and transport on network outage", () => {
    const source = readFileSync(
      join(import.meta.dirname, "..", "examples", "connectivity", "wifi_lte_failover.sd"),
      "utf8",
    );
    const augmented = `${source}\nsimulate_compatibility { fault NetworkOutage; }\n`;
    const program = parse(tokenize(augmented));
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("emit network.disconnected"))).toBe(true);
    expect(logs.some((l) => l.includes("failover wifi -> cellular"))).toBe(true);
  });

  it("dispatches gps.lost when GPSLost fault is simulated", () => {
    const program = parse(
      tokenize(`
simulate_compatibility { fault GPSLost; }
robot R {
  on gps.lost { }
  behavior idle() { }
}
`),
    );
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("emit gps.lost"))).toBe(true);
  });

  it("offsets GPS fix under GpsSpoofing and fires gps.spoofed", () => {
    const program = parse(
      tokenize(`
simulate_compatibility { fault GpsSpoofing; }
robot R {
  sensor gps: GPS;
  on gps.spoofed { }
  behavior idle() {
    let fix = gps.read();
    let _ = fix;
  }
}
`),
    );
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("emit gps.spoofed"))).toBe(true);
  });

  it("fires gps.drift when GpsDrift fault is simulated", () => {
    const program = parse(
      tokenize(`
simulate_compatibility { fault GpsDrift; }
robot R {
  on gps.drift { }
  behavior idle() { }
}
`),
    );
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("emit gps.drift"))).toBe(true);
  });

  it("applyGpsPositionFaults drifts coordinates over time", () => {
    const faults = new Set(["GpsDrift"]);
    const a = applyGpsPositionFaults(faults, 30.0, -97.0, 0);
    const b = applyGpsPositionFaults(faults, 30.0, -97.0, 60_000);
    expect(b.lat).toBeGreaterThan(a.lat);
  });

  it("returns attested SimIdentity when on cellular link", () => {
    const program = parse(
      tokenize(`
connectivity_policy Net { preferred: cellular; fallback: wifi; }
robot R {
  permissions [ cellular.connect ];
  behavior run() {
    let sim = robot.sim_identity();
    let _ = sim;
  }
}
`),
    );
    run(program, { backend: createDefaultSimulator() });
  });

  it("cascades to satellite emergency when fallback is impaired", () => {
    const program = parse(
      tokenize(`
requires_connectivity { gps: required; cellular: required; satellite: optional; }
connectivity_policy RemoteOps { preferred: wifi; fallback: cellular; emergency: satellite; }
hardware H { connectivity [ WiFi6, LTE, Satellite, GPS ]; sensors [ GPS ]; actuators [ DifferentialDrive ]; network { bandwidth: 100 Mbps; latency: 15 ms; } }
robot R {
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  permissions [ gps.read, network.status, wifi.connect, cellular.connect, network.failover ];
  behavior run() { wheels.stop(); }
}
simulate_compatibility { fault NetworkOutage; }
`),
    );
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("emit network.disconnected"))).toBe(true);
    expect(logs.some((l) => l.includes("failover wifi -> cellular"))).toBe(true);
    expect(logs.some((l) => l.includes("emergency link satellite"))).toBe(true);
  });

  it("stays on cellular fallback when only Wi-Fi is impaired", () => {
    const program = parse(
      tokenize(`
connectivity_policy Net { preferred: wifi; fallback: cellular; emergency: satellite; }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
simulate_compatibility { fault WeakWifi; }
`),
    );
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("failover wifi -> cellular"))).toBe(true);
    expect(logs.some((l) => l.includes("emergency link satellite"))).toBe(false);
  });
});
