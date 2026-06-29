import { describe, it, expect } from "vitest";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { SafetyMonitor, createSafetyConfigFromRobot } from "../src/safety/index.js";
import { Environment } from "../src/runtime/index.js";

describe("safety", () => {
  it("blocks motion when stop_if rule triggers", () => {
    const env = new Environment();
    env.define("obstacle", { kind: "number", value: 0.3, unit: "m" });

    const monitor = new SafetyMonitor(
      createSafetyConfigFromRobot(1.5, [
        (e) => {
          const v = e.get("obstacle");
          return v?.kind === "number" && v.value < 0.5;
        },
      ]),
    );

    const result = monitor.evaluateBeforeMotion(env, { x: 0, y: 0 });
    expect(result.allowed).toBe(false);
    expect(result.emergencyStop).toBe(true);
  });

  it("allows motion when safety rules pass", () => {
    const env = new Environment();
    env.define("obstacle", { kind: "number", value: 2.0, unit: "m" });

    const monitor = new SafetyMonitor(
      createSafetyConfigFromRobot(1.5, [
        (e) => {
          const v = e.get("obstacle");
          return v?.kind === "number" && v.value < 0.5;
        },
      ]),
    );

    const result = monitor.evaluateBeforeMotion(env, { x: 0, y: 0 });
    expect(result.allowed).toBe(true);
  });

  it("detects safety zone entry", () => {
    const monitor = new SafetyMonitor(
      createSafetyConfigFromRobot(1.5, [], [
        { name: "keepout", shape: "circle", x: 0, y: 0, radius: 1.0 },
      ]),
    );
    expect(monitor.isInZone("keepout", { x: 0.5, y: 0 })).toBe(true);
    expect(monitor.isInZone("keepout", { x: 5, y: 5 })).toBe(false);
  });

  it("clamps speed to max_speed", () => {
    const monitor = new SafetyMonitor(createSafetyConfigFromRobot(1.0, []));
    expect(monitor.clampSpeed(2.0)).toBe(1.0);
    expect(monitor.clampSpeed(-3.0)).toBe(-1.0);
  });

  it("enforces safety in interpreter — blocks drive on obstacle", () => {
    const source = `
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        safety {
          stop_if lidar.read().nearest_distance < 1.0 m;
        }
        behavior go() {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator({ obstacles: [{ x: 0.5, y: 0, radius: 0.1 }] });
    const blocked: string[] = [];

    const state = run(program, {
      backend: sim,
      maxLoopIterations: 1,
      onMotionBlocked: (r) => blocked.push(r),
    });

    expect(blocked.length).toBeGreaterThan(0);
    expect(state.emergencyStop).toBe(true);
  });
});
