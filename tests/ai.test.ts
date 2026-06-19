import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { compile, run } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("AI support", () => {
  it("tokenizes ai keywords", () => {
    const tokens = tokenize("ai model file infer output input");
    const types = tokens.filter((t) => t.type !== "EOF").map((t) => t.type);
    expect(types).toContain("AI");
    expect(types).toContain("MODEL");
    expect(types).toContain("FILE");
    expect(types).toContain("INFER");
    expect(types).toContain("OUTPUT");
    expect(types).toContain("INPUT");
  });

  it("parses ai block with model declarations", () => {
    const source = `
      robot R {
        ai {
          model nav output NavigationPolicy file "nav.onJsii" input Lidar;
        }
      }
    `;
    const ast = parse(tokenize(source));
    const ai = ast.robots[0].ai!;
    expect(ai.models).toHaveLength(1);
    expect(ai.models[0].name).toBe("nav");
    expect(ai.models[0].outputType).toBe("NavigationPolicy");
    expect(ai.models[0].path).toBe("nav.onJsii");
    expect(ai.models[0].inputs).toEqual(["Lidar"]);
  });

  it("parses infer expressions", () => {
    const source = `
      robot R {
        ai {
          model nav output Velocity file "nav.onJSImport";
        }
        behavior demo() {
          let cmd = infer nav with scan: lidar.read();
        }
      }
    `;
    const ast = parse(tokenize(source));
    const init = ast.robots[0].behaviors[0].body[0];
    expect(init.kind).toBe("VarDecl");
    if (init.kind === "VarDecl") {
      expect(init.init.kind).toBe("InferExpr");
    }
  });

  it("type-checks ai models and infer calls", () => {
    expect(() =>
      compile(`
        import onnx.runtime;
        robot R {
          sensor lidar: Lidar on "/scan";
          ai {
            model nav from onnx.runtime output NavigationPolicy file "nav.onJsii" input Lidar;
          }
          behavior demo() {
            let cmd = infer nav with scan: lidar.read();
          }
        }
      `),
    ).not.toThrow();
  });

  it("runs ai-driven navigation in simulation", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        ai {
          model nav output NavigationPolicy file "nav.onJsii" input Lidar;
        }
        behavior demo() {
          loop every 50ms {
            let scan = lidar.read();
            let cmd = infer nav with scan: scan;
            wheels.drive(linear: cmd.linear, angular: cmd.angular);
          }
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 10 });
    const log = sim.getEventLog();
    expect(log.some((e) => e.includes("drive("))).toBe(true);
  });

  it("rejects unknown ai output types", () => {
    expect(() =>
      compile(`
        robot R {
          ai {
            model bad output UnknownType file "x.onJsii";
          }
        }
      `),
    ).toThrow(/Unknown AI output type/);
  });
});
