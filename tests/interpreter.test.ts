import { describe, it, expect } from "vitest";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("interpreter", () => {
  it("executes let bindings and if/else", () => {
    const source = `
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        behavior test() {
          let scan = lidar.read();
          if scan.nearest_distance < 0.5 m {
            wheels.stop();
          } else {
            wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
          }
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator({ obstacles: [{ x: 100, y: 100, radius: 0.1 }] });
    const state = run(program, { backend: sim, maxLoopIterations: 1 });
    expect(state.velocity.linear).toBeGreaterThan(0);
  });

  it("runs deterministic loop every N ms", () => {
    const source = `
      robot R {
        actuator wheels: DifferentialDrive;
        behavior tick() {
          loop every 100ms {
            wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
          }
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    const state = run(program, { backend: sim, maxLoopIterations: 5 });
    expect(state.pose.x).toBeGreaterThan(0);
  });

  it("stops robot on close obstacle in behavior", () => {
    const source = `
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        behavior avoid() {
          loop every 50ms {
            let scan = lidar.read();
            if scan.nearest_distance < 0.5 m {
              wheels.stop();
            } else {
              wheels.drive(linear: 0.8 m/s, angular: 0.0 rad/s);
            }
          }
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator({ obstacles: [{ x: 0.3, y: 0, radius: 0.1 }] });
    const state = run(program, { backend: sim, maxLoopIterations: 3 });
    expect(state.velocity.linear).toBe(0);
  });
});
