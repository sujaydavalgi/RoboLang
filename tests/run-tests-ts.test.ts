import { describe, it, expect } from "vitest";
import { runTests, runTestsWithRegistry } from "../src/cli/run-program.js";
import { parse } from "../src/parser/index.js";
import { tokenize } from "../src/lexer/index.js";
import { ModuleRegistry } from "../src/modules/index.js";

describe("TypeScript runTests", () => {
  it("runs in-language test blocks", () => {
    const source = `
module math;

export fn double(x: Int) -> Int {
  return x;
}

test "assert passes" {
  assert(true);
}
`;
    const result = runTests(source);
    expect(result.passed).toBe(1);
    expect(result.failed).toBe(0);
  });

  it("counts failure when assert fails", () => {
    const source = `
module math;

test "assert fails" {
  assert(false);
}
`;
    const result = runTests(source);
    expect(result.passed).toBe(0);
    expect(result.failed).toBe(1);
    expect(result.logs.some((l) => l.includes("Assertion failed"))).toBe(true);
  });

  it("runs tests with module registry", () => {
    const helper = `
module math.helpers;

export fn always_true() -> Bool {
  return true;
}
`;
    const main = `
module math;
import math.helpers;

test "imported helper" {
  assert(true);
}
`;
    const registry = new ModuleRegistry();
    registry.register("math.helpers", parse(tokenize(helper)));
    const result = runTestsWithRegistry(main, registry);
    expect(result.passed).toBe(1);
    expect(result.failed).toBe(0);
  });
});
