import { describe, expect, it } from "vitest";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { TypeCheckError } from "../src/types/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("trait objects (dyn Trait)", () => {
  it("type-checks and runs trait object dispatch", () => {
    const source = `
trait Greeter {
  fn greet() -> Void;
}

robot R {
  actuator wheels: DifferentialDrive;

  agent Nav {
    plan { wheels.stop(); }
  }

  impl Greeter for Nav {
    fn greet() -> Void { wheels.stop(); }
  }

  behavior run() {
    let handler: dyn Greeter = Nav;
    handler.greet();
  }
}
`;
    const { program } = compile(source);
    run(program, { backend: createDefaultSimulator(), maxLoopIterations: 5 });
  });

  it("rejects agent without trait impl", () => {
    const source = `
trait Worker {
  fn work() -> Void;
}

robot R {
  agent Helper {
    plan { }
  }

  behavior run() {
    let w: dyn Worker = Helper;
    w.work();
  }
}
`;
    expect(() => compile(source)).toThrow(TypeCheckError);
    try {
      compile(source);
    } catch (e) {
      const err = e as TypeCheckError;
      expect(err.errors.some((d) => d.message.includes("does not implement trait"))).toBe(true);
    }
  });
});
