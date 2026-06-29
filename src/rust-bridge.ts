/**
 * rust bridge module (rust-bridge.ts).
 * @module
 */

import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { existsSync, readFileSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

export type Diagnostic = { message: string; line: number; column: number };
export type CheckResult = { ok: boolean; diagnostics: Diagnostic[] };

export type {
  CompatItem,
  CompatSeverity,
  MatrixCell,
  VerifyResult,
} from "./rust-bridge-types.js";
import type { VerifyResult } from "./rust-bridge-types.js";
export type RunResult = {
  state: {
    pose: { x: number; y: number; theta: number; z?: number };
    velocity: { linear: number; angular: number };
    emergency_stop: boolean;
  };
  events: string[];
  logs: string[];
};

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

function cliPath(): string | null {
  // Description:
  //     CliPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `cliPath`.
  //
  // Example:
  //     const result = cliPath();
  // Description:
  //     CliPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `cliPath`.
  //
  // Example:
  //     const result = cliPath();

  // const result = cliPath();
  const release = join(repoRoot, "target/release/spanda");
  const debug = join(repoRoot, "target/debug/spanda");
  const cargoTarget = process.env.CARGO_TARGET_DIR;
  const candidates = [
    ...(cargoTarget
      ? [join(cargoTarget, "release/spanda"), join(cargoTarget, "debug/spanda")]
      : []),
    release,
    debug,
  ].filter((p) => existsSync(p));

  // continue when length equals 0.
  if (candidates.length === 0) return null;

  // continue when length equals 1.
  if (candidates.length === 1) return candidates[0]!;
  const newest = candidates.reduce((a, b) =>
    statSync(a).mtimeMs >= statSync(b).mtimeMs ? a : b,
  );
  return newest;
}

export function isCliAvailable(): boolean {
  // Description:
  //     IsCliAvailable.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isCliAvailable`.
  //
  // Example:
  //     const result = isCliAvailable();
  // Description:
  //     IsCliAvailable.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isCliAvailable`.
  //
  // Example:
  //     const result = isCliAvailable();

  // const result = isCliAvailable();
  return cliPath() !== null;
}

export function checkViaCli(source: string): CheckResult {
  // Description:
  //     CheckViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: CheckResult
  //         Return value from `checkViaCli`.
  //
  // Example:
  //     const result = checkViaCli(source);
  // Description:
  //     CheckViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: CheckResult
  //         Return value from `checkViaCli`.
  //
  // Example:
  //     const result = checkViaCli(source);

  // const result = checkViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      ok: false,
      diagnostics: [{ message: "Rust CLI not built (run: npm run build:rust)", line: 1, column: 1 }],
    };
  }
  const tmp = join(repoRoot, `.spanda-check-${process.pid}-${Date.now()}.sd`);
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["check", "--json", tmp], { encoding: "utf-8" });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      diagnostics: [{ message: result.stderr || "CLI check failed", line: 1, column: 1 }],
    };
  }
  return JSON.parse(result.stdout) as CheckResult;
}

export function verifyViaCli(
  source: string,
  args: string[] = [],
): VerifyResult {
  // Description:
  //     VerifyViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     args: string[] = []
  //         Caller-supplied args.
  //
  // Outputs:
  //     result: VerifyResult
  //         Return value from `verifyViaCli`.
  //
  // Example:
  //     const result = verifyViaCli(source, args);
  // Description:
  //     VerifyViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     args: string[] = []
  //         Caller-supplied args.
  //
  // Outputs:
  //     result: VerifyResult
  //         Return value from `verifyViaCli`.
  //
  // Example:
  //     const result = verifyViaCli(source, args);

  // const result = verifyViaCli(source, args);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      ok: false,
      items: [
        {
          category: "error",
          message: "Rust CLI not built (run: npm run build:rust)",
          severity: "error",
          line: 1,
          column: 1,
        },
      ],
    };
  }
  const tmp = join(repoRoot, `.spanda-verify-${process.pid}-${Date.now()}.sd`);
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["verify", tmp, "--json", ...args], { encoding: "utf-8" });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      items: [
        {
          category: "error",
          message: result.stderr || "CLI verify failed",
          severity: "error",
          line: 1,
          column: 1,
        },
      ],
    };
  }
  return JSON.parse(result.stdout) as VerifyResult;
}

export function runViaCli(source: string): RunResult {
  // Description:
  //     RunViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: RunResult
  //         Return value from `runViaCli`.
  //
  // Example:
  //     const result = runViaCli(source);
  // Description:
  //     RunViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: RunResult
  //         Return value from `runViaCli`.
  //
  // Example:
  //     const result = runViaCli(source);

  // const result = runViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const tmp = join(repoRoot, `.spanda-run-${process.pid}-${Date.now()}.sd`);
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["run", "--json", tmp], { encoding: "utf-8" });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  const parsed = JSON.parse(result.stdout || "{}") as {
    ok: boolean;
    result?: RunResult;
    diagnostics?: Diagnostic[];
  };

  // continue when result is falsy.
  if (!parsed.ok || !parsed.result) {
    throw new Error(parsed.diagnostics?.[0]?.message ?? "Run failed");
  }
  return parsed.result;
}

export type FormatResult = { ok: boolean; changed: boolean; formatted: string };
export type LintIssue = {
  rule: string;
  message: string;
  line: number;
  column: number;
  severity: "warning" | "error";
};
export type LintResult = { ok: boolean; issues: LintIssue[] };
export type DocResult = { ok: boolean; markdown: string };
export type CodegenTarget = "native" | "wasm" | "esp32";
export type DebugPause = { line: number; reason: string };
export type DebugResult = { ok: boolean; pauses: DebugPause[] };

function withTempSource(
  source: string,
  suffix: string,
  run: (file: string) => SpawnSyncReturns<string>,
): SpawnSyncReturns<string> {
  // Description:
  //     WithTempSource.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     suffix: string
  //         Caller-supplied suffix.
  //
  // Outputs:
  //     result: SpawnSyncReturns<string>
  //         Return value from `withTempSource`.
  //
  // Example:
  //     const result = withTempSource(source, suffix);
  // Compute tmp for the following logic.
  const tmp = join(
    repoRoot,
    `${suffix.replace(/\.sd$/, "")}-${process.pid}-${Date.now()}.sd`,
  );
  writeFileSync(tmp, source);
  const result = run(tmp);

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return result;
}

export function fmtViaCli(source: string): FormatResult {
  // Description:
  //     FmtViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: FormatResult
  //         Return value from `fmtViaCli`.
  //
  // Example:
  //     const result = fmtViaCli(source);
  // Description:
  //     FmtViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: FormatResult
  //         Return value from `fmtViaCli`.
  //
  // Example:
  //     const result = fmtViaCli(source);

  // const result = fmtViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, changed: false, formatted: source };
  }
  const result = withTempSource(source, ".spanda-fmt-tmp.sd", (file) =>
    spawnSync(bin, ["fmt", "--json", file], { encoding: "utf-8" }),
  );

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return { ok: false, changed: false, formatted: source };
  }
  return JSON.parse(result.stdout) as FormatResult;
}

export function lintViaCli(source: string): LintResult {
  // Description:
  //     LintViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: LintResult
  //         Return value from `lintViaCli`.
  //
  // Example:
  //     const result = lintViaCli(source);
  // Description:
  //     LintViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: LintResult
  //         Return value from `lintViaCli`.
  //
  // Example:
  //     const result = lintViaCli(source);

  // const result = lintViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      ok: false,
      issues: [{ rule: "cli", message: "Rust CLI not built", line: 1, column: 1, severity: "error" }],
    };
  }
  const result = withTempSource(source, ".spanda-lint-tmp.sd", (file) =>
    spawnSync(bin, ["lint", "--json", file], { encoding: "utf-8" }),
  );

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      issues: [{ rule: "cli", message: result.stderr || "lint failed", line: 1, column: 1, severity: "error" }],
    };
  }
  return JSON.parse(result.stdout) as LintResult;
}

export function docViaCli(source: string): DocResult {
  // Description:
  //     DocViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: DocResult
  //         Return value from `docViaCli`.
  //
  // Example:
  //     const result = docViaCli(source);
  // Description:
  //     DocViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: DocResult
  //         Return value from `docViaCli`.
  //
  // Example:
  //     const result = docViaCli(source);

  // const result = docViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, markdown: "" };
  }
  const result = withTempSource(source, ".spanda-doc-tmp.sd", (file) =>
    spawnSync(bin, ["doc", "--json", file], { encoding: "utf-8" }),
  );

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return { ok: false, markdown: "" };
  }
  const parsed = JSON.parse(result.stdout) as {
    ok: boolean;
    markdown?: string;
    content?: string;
  };
  return {
    ok: parsed.ok,
    markdown: parsed.markdown ?? parsed.content ?? "",
  };
}

export function codegenViaCli(source: string, target: CodegenTarget = "native"): string {
  // Description:
  //     CodegenViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     target: CodegenTarget = "native"
  //         Caller-supplied target.
  //
  // Outputs:
  //     result: string
  //         Return value from `codegenViaCli`.
  //
  // Example:
  //     const result = codegenViaCli(source, target);
  // Description:
  //     CodegenViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     target: CodegenTarget = "native"
  //         Caller-supplied target.
  //
  // Outputs:
  //     result: string
  //         Return value from `codegenViaCli`.
  //
  // Example:
  //     const result = codegenViaCli(source, target);

  // const result = codegenViaCli(source, target);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const result = withTempSource(source, ".spanda-codegen-tmp.sd", (file) =>
    spawnSync(bin, ["codegen", "--target", target, file], { encoding: "utf-8" }),
  );

  // continue when status differs from 0.
  if (result.status !== 0) {
    throw new Error(result.stderr || "codegen failed");
  }
  return result.stdout ?? "";
}

export function deployViaCli(source: string): string {
  // Description:
  //     DeployViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: string
  //         Return value from `deployViaCli`.
  //
  // Example:
  //     const result = deployViaCli(source);
  // Description:
  //     DeployViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: string
  //         Return value from `deployViaCli`.
  //
  // Example:
  //     const result = deployViaCli(source);

  // const result = deployViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const result = withTempSource(source, ".spanda-deploy-tmp.sd", (file) =>
    spawnSync(bin, ["deploy", "--target", "wasm", file], { encoding: "utf-8" }),
  );

  // continue when status differs from 0.
  if (result.status !== 0) {
    throw new Error(result.stderr || "deploy failed");
  }
  return result.stdout ?? "";
}

export function debugViaCli(source: string, breakpoints: number[] = []): DebugResult {
  // Description:
  //     DebugViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     breakpoints: number[] = []
  //         Caller-supplied breakpoints.
  //
  // Outputs:
  //     result: DebugResult
  //         Return value from `debugViaCli`.
  //
  // Example:
  //     const result = debugViaCli(source, breakpoints);
  // Description:
  //     DebugViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     breakpoints: number[] = []
  //         Caller-supplied breakpoints.
  //
  // Outputs:
  //     result: DebugResult
  //         Return value from `debugViaCli`.
  //
  // Example:
  //     const result = debugViaCli(source, breakpoints);

  // const result = debugViaCli(source, breakpoints);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, pauses: [] };
  }
  const args = ["debug", ...breakpoints.flatMap((line) => ["--break", String(line)])];
  const result = withTempSource(source, ".spanda-debug-tmp.sd", (file) =>
    spawnSync(bin, [...args, file], { encoding: "utf-8" }),
  );

  // continue when status differs from 0.
  if (result.status !== 0) {
    return { ok: false, pauses: [] };
  }
  const pauses: DebugPause[] = [];

  // Handle each input line.
  for (const line of (result.stdout ?? "").split("\n")) {
    const m = line.match(/^\s*line (\d+) — (.+)$/);

    // continue when m.
    if (m) {
      pauses.push({ line: Number(m[1]), reason: m[2]! });
    }
  }
  return { ok: true, pauses };
}

export function runNativeCli(args: string[]): SpawnSyncReturns<string> {
  // Description:
  //     RunNativeCli.
  //
  // Inputs:
  //     args: string[]
  //         Caller-supplied args.
  //
  // Outputs:
  //     result: SpawnSyncReturns<string>
  //         Return value from `runNativeCli`.
  //
  // Example:
  //     const result = runNativeCli(args);
  // Description:
  //     RunNativeCli.
  //
  // Inputs:
  //     args: string[]
  //         Caller-supplied args.
  //
  // Outputs:
  //     result: SpawnSyncReturns<string>
  //         Return value from `runNativeCli`.
  //
  // Example:
  //     const result = runNativeCli(args);

  // const result = runNativeCli(args);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      status: 1,
      signal: null,
      output: ["", "Rust CLI not built (run: npm run build:rust)", ""],
      stdout: "",
      stderr: "Rust CLI not built (run: npm run build:rust)",
      pid: 0,
      error: new Error("Rust CLI not built"),
    } as SpawnSyncReturns<string>;
  }
  return spawnSync(bin, args, { encoding: "utf-8" });
}

export function verifyFileViaCli(filePath: string, extraArgs: string[] = []): VerifyResult {
  // Description:
  //     VerifyFileViaCli.
  //
  // Inputs:
  //     filePath: string
  //         Caller-supplied filePath.
  //     extraArgs: string[] = []
  //         Caller-supplied extraArgs.
  //
  // Outputs:
  //     result: VerifyResult
  //         Return value from `verifyFileViaCli`.
  //
  // Example:
  //     const result = verifyFileViaCli(filePath, extraArgs);
  // Description:
  //     VerifyFileViaCli.
  //
  // Inputs:
  //     filePath: string
  //         Caller-supplied filePath.
  //     extraArgs: string[] = []
  //         Caller-supplied extraArgs.
  //
  // Outputs:
  //     result: VerifyResult
  //         Return value from `verifyFileViaCli`.
  //
  // Example:
  //     const result = verifyFileViaCli(filePath, extraArgs);

  // const result = verifyFileViaCli(filePath, extraArgs);
  const source = readFileSync(filePath, "utf-8");
  return verifyViaCli(source, extraArgs);
}

export type SecurityCliReport = {
  findings: Array<{
    severity: string;
    message: string;
    line: number;
    column: number;
  }>;
};

function securityViaCli(source: string, mode: "check" | "audit"): SecurityCliReport {
  // Description:
  //     SecurityViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //     mode: "check" | "audit"
  //         Caller-supplied mode.
  //
  // Outputs:
  //     result: SecurityCliReport
  //         Return value from `securityViaCli`.
  //
  // Example:
  //     const result = securityViaCli(source, mode);

  // Run native spanda security check or audit against source text.
  const bin = cliPath();
  if (!bin) {
    return {
      findings: [
        {
          severity: "error",
          message: "Rust CLI not built (run: npm run build:rust)",
          line: 1,
          column: 1,
        },
      ],
    };
  }
  const result = withTempSource(source, `.spanda-security-${mode}-tmp.sd`, (file) =>
    spawnSync(bin, ["security", mode, "--json", file], { encoding: "utf-8" }),
  );
  if (!result.stdout?.trim()) {
    return {
      findings: [
        {
          severity: "error",
          message: result.stderr || `security ${mode} failed`,
          line: 1,
          column: 1,
        },
      ],
    };
  }
  return JSON.parse(result.stdout) as SecurityCliReport;
}

export function securityCheckViaCli(source: string): SecurityCliReport {
  // Description:
  //     SecurityCheckViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: SecurityCliReport
  //         Return value from `securityCheckViaCli`.
  //
  // Example:

  //     const result = securityCheckViaCli(source);

  return securityViaCli(source, "check");
}

export function securityAuditViaCli(source: string): SecurityCliReport {
  // Description:
  //     SecurityAuditViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: SecurityCliReport
  //         Return value from `securityAuditViaCli`.
  //
  // Example:

  //     const result = securityAuditViaCli(source);

  return securityViaCli(source, "audit");
}
