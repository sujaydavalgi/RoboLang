import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { tokenize } from "../src/lexer/index.js";
import { parse, ParseError } from "../src/parser/index.js";
import { typeCheck, TypeCheckError } from "../src/types/index.js";

const repoRoot = join(import.meta.dirname, "..");
const roverDeploy = readFileSync(join(repoRoot, "examples/hardware/rover_deploy.sd"), "utf-8");
const fullCompat = readFileSync(join(repoRoot, "examples/hardware/full_compat.sd"), "utf-8");

describe("hardware program parsing (TypeScript mirror)", () => {
  it("parses hardware profile and deploy", () => {
    const program = parse(tokenize(roverDeploy));
    expect(program.hardwareProfiles).toHaveLength(1);
    expect(program.hardwareProfiles[0]?.name).toBe("RoverV1");
    expect(program.hardwareProfiles[0]?.memoryMb).toBe(4096);
    expect(program.hardwareProfiles[0]?.sensors).toEqual(["Camera", "Lidar", "IMU"]);
    expect(program.deployments).toHaveLength(1);
    expect(program.deployments[0]?.robotName).toBe("RoverProgram");
    expect(program.deployments[0]?.targets).toEqual(["RoverV1"]);
  });

  it("parses requires_hardware, requires_network, and simulate_compatibility", () => {
    const program = parse(tokenize(fullCompat));
    expect(program.requiresHardware?.memoryMbMin).toBe(2048);
    expect(program.requiresHardware?.sensors).toEqual(["Camera", "Lidar"]);
    expect(program.requiresNetwork?.bandwidthMbpsMin).toBe(10);
    expect(program.requiresNetwork?.latencyMsMax).toBe(50);
    expect(program.simulateCompatibility?.faults).toHaveLength(1);
    expect(program.simulateCompatibility?.faults[0]?.faultType).toBe("BatteryDegradation");
    expect(program.deployments[0]?.targets).toEqual(["RoverV1", "ESP32", "JetsonOrin"]);
  });

  it("type-checks valid deploy against declared and built-in profiles", () => {
    expect(() => typeCheck(parse(tokenize(roverDeploy)))).not.toThrow();
    expect(() => typeCheck(parse(tokenize(fullCompat)))).not.toThrow();
  });

  it("rejects deploy to unknown hardware profile", () => {
    const source = `
robot R { actuator wheels: DifferentialDrive; behavior run() { wheels.stop(); } }
deploy R to UnknownBoard;
`;
    expect(() => typeCheck(parse(tokenize(source)))).toThrow(TypeCheckError);
    try {
      typeCheck(parse(tokenize(source)));
    } catch (e) {
      const err = e as TypeCheckError;
      expect(err.errors.some((d) => d.message.includes("UnknownBoard"))).toBe(true);
    }
  });

  it("rejects deploy referencing unknown robot", () => {
    const source = `
hardware Board { memory: 1 GB; }
deploy MissingRobot to Board;
`;
    expect(() => typeCheck(parse(tokenize(source)))).toThrow(TypeCheckError);
    try {
      typeCheck(parse(tokenize(source)));
    } catch (e) {
      const err = e as TypeCheckError;
      expect(err.errors.some((d) => d.message.includes("MissingRobot"))).toBe(true);
    }
  });

  it("reports parse error for malformed requires_hardware", () => {
    const source = `requires_hardware { memory >= ; }`;
    expect(() => parse(tokenize(source))).toThrow(ParseError);
  });
});
