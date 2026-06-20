import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { TypeCheckError } from "../src/types/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("enum payloads", () => {
  it("type-checks and runs enum payload constructor and match bindings", () => {
    const source = `
enum Command {
  Stop,
  Drive(Float, Float)
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let cmd = Drive(0.3, 0.0);
    match cmd {
      Stop => wheels.stop();
      Drive(_speed, _turn) => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    };
  }
}
`;
    expect(() => compile(source)).not.toThrow();
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() => run(program, { backend: sim, maxLoopIterations: 1 })).not.toThrow();
  });

  it("rejects enum payload arity mismatch", () => {
    const source = `
enum Command { Drive(Float, Float) }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let x = Drive(1.0);
  }
}
`;
    try {
      compile(source);
      expect.fail("expected type check to fail");
    } catch (err) {
      expect(err).toBeInstanceOf(TypeCheckError);
      const tcErr = err as TypeCheckError;
      expect(
        tcErr.errors.some((d) => d.message.includes("expects 2 payload")),
      ).toBe(true);
    }
  });
});
