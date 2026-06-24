import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { collectContinuityDiagnostics } from "../src/continuity-diagnostics.js";
import { readinessDiagnostics } from "../src/readiness.js";

describe("continuity diagnostics", () => {
  it("warns when fleet lacks continuity_policy", () => {
    const program = parse(
      tokenize(`
fleet Patrol { RoverA; RoverB; }
robot RoverA {
  sensor gps: GPS;
  actuator w: DifferentialDrive;
  safety { max_speed = 1 m/s; }
  behavior b() {}
}
robot RoverB {
  sensor gps: GPS;
  actuator w: DifferentialDrive;
  safety { max_speed = 1 m/s; }
  behavior b() {}
}
`),
    );
    const diags = collectContinuityDiagnostics(program);
    expect(diags.some((d) => d.category === "continuity:policy")).toBe(true);
  });

  it("merges continuity diagnostics into readinessDiagnostics", () => {
    const source = `
continuity_policy Risky {
  on robot.failed { hot takeover; }
}
robot R {
  sensor gps: GPS;
  actuator w: DifferentialDrive;
  safety { max_speed = 1 m/s; }
  behavior b() {}
}
`;
    const items = readinessDiagnostics(source);
    expect(items.some((d) => d.category === "continuity:approval")).toBe(true);
  });
});
