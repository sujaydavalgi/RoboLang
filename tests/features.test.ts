import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("language features", () => {
  it("tokenizes new robotics keywords", () => {
    const tokens = tokenize("node topic service action publish call send_goal zone emergency_stop");
    const types = tokens.filter((t) => t.type !== "EOF").map((t) => t.type);
    expect(types).toContain("NODE");
    expect(types).toContain("TOPIC");
    expect(types).toContain("SERVICE");
    expect(types).toContain("ACTION");
    expect(types).toContain("PUBLISH");
    expect(types).toContain("EMERGENCY_STOP");
  });

  it("parses node, topic, service, action declarations", () => {
    const source = `
      robot R {
        node nav on "/nav";
        topic cmd: Velocity publish on "/cmd_vel";
        service reset: ResetCostmap;
        action go: NavigateTo;
      }
    `;
    const ast = parse(tokenize(source));
    const robot = ast.robots[0];
    expect(robot.nodes).toHaveLength(1);
    expect(robot.topics).toHaveLength(1);
    expect(robot.services).toHaveLength(1);
    expect(robot.actions).toHaveLength(1);
  });

  it("parses safety zones", () => {
    const source = `
      robot R {
        safety {
          zone keepout circle at (1.0 m, 2.0 m) radius 0.5 m;
        }
      }
    `;
    const ast = parse(tokenize(source));
    expect(ast.robots[0].safety!.zones).toHaveLength(1);
    expect(ast.robots[0].safety!.zones[0].shape).toBe("circle");
  });

  it("type-checks pose, velocity, trajectory, and publish", () => {
    expect(() =>
      compile(`
        robot R {
          topic cmd: Velocity publish on "/cmd_vel";
          behavior demo() {
            let p = pose(x: 1.0 m, y: 0.0 m, theta: 0.0 rad);
            let v = velocity(linear: 0.5 m/s, angular: 0.0 rad/s);
            publish cmd with v;
            let path = trajectory(from: p, to: pose(x: 2.0 m, y: 0.0 m, theta: 0.0 rad), steps: 5);
          }
        }
      `),
    ).not.toThrow();
  });

  it("runs emergency_stop and reset_emergency_stop", () => {
    const source = `
      robot R {
        actuator wheels: DifferentialDrive;
        behavior test() {
          emergency_stop;
          reset_emergency_stop;
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 1 });
    expect(sim.getEventLog()).toContain("drive(0.50 m/s, 0.00 rad/s)");
  });

  it("runs publish and service call", () => {
    const source = `
      robot R {
        topic cmd: Velocity publish on "/cmd_vel";
        service reset: ResetCostmap;
        behavior test() {
          publish cmd with velocity(linear: 0.2 m/s, angular: 0.0 rad/s);
          call reset();
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 1 });
    expect(sim.getPublishedTopics()).toHaveLength(1);
    expect(sim.getServiceLog()).toContain("reset:ResetCostmap");
  });

  it("runs trajectory follow", () => {
    const source = `
      robot R {
        actuator wheels: DifferentialDrive;
        behavior test() {
          let start = robot.pose();
          let end = pose(x: 1.0 m, y: 0.0 m, theta: 0.0 rad);
          let path = trajectory(from: start, to: end, steps: 5);
          wheels.follow(path: path);
          loop every 100ms {
            let current = robot.pose();
          }
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    const state = run(program, { backend: sim, maxLoopIterations: 15 });
    expect(state.pose.x).toBeGreaterThan(0.5);
  });
});
