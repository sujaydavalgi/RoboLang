import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { typeCheck } from "../src/types/index.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("generic struct literals", () => {
  it("type-checks and runs Box<Int> literal", () => {
    const source = `
struct Box<T> {
  value: T;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let b = Box<Int> { value: 42 };
    let _v = b.value;
    wheels.stop();
  }
}
`;
    expect(() => typeCheck(parse(tokenize(source)))).not.toThrow();
    const program = parse(tokenize(source));
    typeCheck(program);
    run(program, { backend: createDefaultSimulator() });
  });

  it("rejects generic arity mismatch", () => {
    const source = `
struct Box<T> { value: T; }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { let b = Box<Int, Float> { value: 1 }; }
}
`;
    expect(() => typeCheck(parse(tokenize(source)))).toThrow();
  });
});
