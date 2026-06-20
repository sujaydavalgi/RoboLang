import { describe, it, expect } from "vitest";
import { spawnSync } from "node:child_process";
import { join } from "node:path";

const repoRoot = join(import.meta.dirname, "..");
const cliEntry = join(repoRoot, "src/cli/index.ts");

function runCli(args: string[]): { status: number | null; stdout: string; stderr: string } {
  const result = spawnSync("node", ["--import", "tsx", cliEntry, ...args], {
    encoding: "utf-8",
    cwd: repoRoot,
  });
  return { status: result.status, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
}

describe("TypeScript CLI", () => {
  it("prints help for --help", () => {
    const { status, stdout } = runCli(["--help"]);
    expect(status).toBe(0);
    expect(stdout).toContain("spanda verify");
    expect(stdout).toContain("spanda fmt");
  });

  it("type-checks rover example", () => {
    const { status, stdout } = runCli(["check", "examples/rover.sd"]);
    expect(status).toBe(0);
    expect(stdout).toMatch(/no type errors|✓/);
  });

  it("rejects unknown command", () => {
    const { status, stderr } = runCli(["not-a-command"]);
    expect(status).toBe(1);
    expect(stderr + runCli(["not-a-command"]).stdout).toContain("Unknown command");
  });
});
