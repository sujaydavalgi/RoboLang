import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("AI-native Spanda", () => {
  it("tokenizes ai_model and agent keywords", () => {
    const tokens = tokenize("ai_model agent uses tools goal plan memory provider");
    const types = tokens.filter((t) => t.type !== "EOF").map((t) => t.type);
    expect(types).toContain("AI_MODEL");
    expect(types).toContain("AGENT");
    expect(types).toContain("USES");
    expect(types).toContain("TOOLS");
    expect(types).toContain("GOAL");
    expect(types).toContain("PLAN");
    expect(types).toContain("MEMORY");
  });

  it("parses ai_model declarations", () => {
    const source = `
      robot R {
        ai_model planner: LLM {
          provider: "mock";
          model: "safe-planner";
          temperature: 0.1;
        }
      }
    `;
    const ast = parse(tokenize(source));
    const model = ast.robots[0].ai_models[0];
    expect(model.name).toBe("planner");
    expect(model.modelType).toBe("LLM");
    expect(model.config.find((c) => c.key === "provider")?.value).toBe("mock");
  });

  it("parses agent declarations", () => {
    const source = `
      robot R {
        ai_model planner: LLM { provider: "mock"; model: "p"; }
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        agent Navigator {
          uses planner;
          tools [lidar, wheels];
          memory short_term;
          goal "Navigate safely";
          plan {
            let x = lidar.read();
          }
        }
      }
    `;
    const ast = parse(tokenize(source));
    const agent = ast.robots[0].agents[0];
    expect(agent.name).toBe("Navigator");
    expect(agent.usesAi).toEqual(["planner"]);
    expect(agent.tools).toEqual(["lidar", "wheels"]);
    expect(agent.memoryKind).toBe("short_term");
    expect(agent.goal).toBe("Navigate safely");
    expect(agent.planBody).toHaveLength(1);
  });

  it("type-checks mock LLM reason and safety validate flow", () => {
    expect(() => compile(`
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        ai_model planner: LLM { provider: "mock"; model: "p"; }
        safety { max_speed = 1.0 m/s; }
        behavior demo() {
          let scan = lidar.read();
          let proposal = planner.reason(prompt: "Go", input: scan);
          let action = safety.validate(proposal);
          wheels.execute(action);
        }
      }
    `)).not.toThrow();
  });

  it("runs mock LLM navigation through safety validation", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        ai_model planner: LLM { provider: "mock"; model: "p"; }
        safety { max_speed = 1.0 m/s; }
        behavior demo() {
          let scan = lidar.read();
          let proposal = planner.reason(prompt: "Go", input: scan);
          let action = safety.validate(proposal);
          wheels.execute(action);
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 1 });
    expect(sim.getEventLog().some((e) => e.includes("drive("))).toBe(true);
  });

  it("runs vision detect call", () => {
    const source = `
      robot R {
        sensor camera: Camera on "/camera";
        ai_model vision: VisionModel { provider: "mock"; model: "yolo"; }
        behavior demo() {
          let frame = camera.frame();
          let object = vision.detect(frame);
        }
      }
    `;
    expect(() => compile(source)).not.toThrow();
    const { program } = compile(source);
    run(program, { backend: createDefaultSimulator(), maxLoopIterations: 1 });
  });

  it("rejects direct ActionProposal execution at compile time", () => {
    expect(() =>
      compile(`
        robot R {
          sensor lidar: Lidar on "/scan";
          actuator wheels: DifferentialDrive;
          ai_model planner: LLM { provider: "mock"; model: "p"; }
          behavior demo() {
            let scan = lidar.read();
            let proposal = planner.reason(prompt: "Go", input: scan);
            wheels.execute(proposal);
          }
        }
      `),
    ).toThrow(/ActionProposal/);
  });

  it("rejects unsafe AI drive at compile time", () => {
    expect(() =>
      compile(`
        robot R {
          actuator wheels: DifferentialDrive;
          ai_model planner: LLM { provider: "mock"; model: "p"; }
          behavior demo() {
            planner.drive(wheels);
          }
        }
      `),
    ).toThrow(/cannot control actuators/);
  });

  it("rejects unvalidated ActionProposal passed to actuator.execute at compile time", () => {
    expect(() =>
      compile(`
        robot R {
          sensor lidar: Lidar on "/scan";
          actuator wheels: DifferentialDrive;
          ai_model planner: LLM { provider: "mock"; model: "p"; }
          behavior demo() {
            let proposal = planner.reason(prompt: "Go", input: lidar.read());
            wheels.execute(proposal);
          }
        }
      `),
    ).toThrow(/ActionProposal/);
  });

  it("runs agent plan from behavior loop", () => {
    const { program } = compile(`
      robot R {
        sensor lidar: Lidar on "/scan";
        sensor camera: Camera on "/camera";
        actuator wheels: DifferentialDrive;
        ai_model planner: LLM { provider: "mock"; model: "p"; }
        safety { max_speed = 1.0 m/s; }
        agent Nav {
          uses planner;
          tools [lidar, camera, wheels];
          goal "Go";
          plan {
            let scene = camera.analyze();
            let proposal = planner.reason(prompt: "Go", input: scene);
            let action = safety.validate(proposal);
            wheels.execute(action);
          }
        }
        behavior run() {
          loop every 50ms { Nav.plan(); }
        }
      }
    `);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 5 });
    expect(sim.getEventLog().length).toBeGreaterThan(0);
  });
});
