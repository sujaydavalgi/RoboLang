#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { compileFile, run } from "../compile.js";
import { createDefaultSimulator } from "../simulator/index.js";
import { LexerError } from "../lexer/index.js";
import { ParseError } from "../parser/index.js";
import { TypeCheckError } from "../types/index.js";
import { RuntimeError } from "../runtime/index.js";
import {
  isCliAvailable,
  runNativeCli,
  fmtViaCli,
  lintViaCli,
  docViaCli,
  codegenViaCli,
  deployViaCli,
  debugViaCli,
  type VerifyResult,
} from "../rust-bridge.js";

const USAGE = `Spanda Programming Language — the pulse of autonomous intelligence

Usage:
  spanda check [--json] <file.sd>
  spanda verify [--json] [--target <Profile>] [--all-targets] [--simulate] <file.sd>
  spanda compatibility [flags] <file.sd>     Alias for verify
  spanda run [--json] [--verbose] <file.sd>
  spanda sim [--json] <file.sd>
  spanda fmt [--json] <file.sd>
  spanda lint [--json] <file.sd>
  spanda doc [--json] [--out <file.md>] <file.sd>
  spanda codegen [--target native|wasm|esp32] [--out <file>] <file.sd>
  spanda deploy --target wasm [--out <file.json>] <file.sd>
  spanda debug [--break <line>] <file.sd>
  spanda ir [--json] <file.sd>

Package commands (require native CLI: npm run build:rust):
  spanda init [name] [--description <text>]
  spanda build [--project <dir>]
  spanda test [--project <dir>]
  spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]
  spanda remove <package>
  spanda install [--project <dir>]
  spanda publish [--project <dir>]
  spanda registry search <query>

Examples:
  spanda check examples/rover.sd
  spanda verify examples/hardware/rover_deploy.sd --all-targets
  spanda run examples/rover.sd
  spanda fmt examples/rover.sd
`;

type ParsedArgs = {
  command: string;
  positional: string[];
  json: boolean;
  verbose: boolean;
  flags: Map<string, string | boolean>;
};

function parseArgs(argv: string[]): ParsedArgs {
  const positional: string[] = [];
  const flags = new Map<string, string | boolean>();
  let json = false;
  let verbose = false;

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]!;
    if (arg === "--json") {
      json = true;
    } else if (arg === "--verbose") {
      verbose = true;
    } else if (arg.startsWith("--")) {
      const key = arg.slice(2);
      const next = argv[i + 1];
      if (next && !next.startsWith("-")) {
        flags.set(key, next);
        i++;
      } else {
        flags.set(key, true);
      }
    } else if (arg.startsWith("-") && arg.length === 2) {
      flags.set(arg.slice(1), true);
    } else {
      positional.push(arg);
    }
  }

  const command = positional.shift() ?? "";
  return { command, positional, json, verbose, flags };
}

function requireNative(message: string): void {
  if (!isCliAvailable()) {
    console.error(`Error: ${message}`);
    console.error("Build the native CLI: npm run build:rust");
    process.exit(1);
  }
}

function flagStr(flags: Map<string, string | boolean>, key: string): string | undefined {
  const v = flags.get(key);
  return typeof v === "string" ? v : undefined;
}

function flagBool(flags: Map<string, string | boolean>, key: string): boolean {
  return flags.get(key) === true;
}

function main(): void {
  const parsed = parseArgs(process.argv.slice(2));
  const { command, positional, json, verbose, flags } = parsed;

  if (!command || command === "help" || command === "--help" || command === "-h") {
    console.log(USAGE);
    process.exit(0);
  }

  try {
    switch (command) {
      case "check":
        handleCheck(positional[0], json);
        break;
      case "verify":
      case "compatibility":
        handleVerify(positional[0], json, flags);
        break;
      case "run":
      case "sim":
        handleRun(positional[0], command === "sim", json, verbose);
        break;
      case "fmt":
        handleFmt(positional[0], json);
        break;
      case "lint":
        handleLint(positional[0], json);
        break;
      case "doc":
        handleDoc(positional[0], json, flagStr(flags, "out"));
        break;
      case "codegen":
        handleCodegen(positional[0], flagStr(flags, "target"), flagStr(flags, "out"));
        break;
      case "deploy":
        handleDeploy(positional[0], flagStr(flags, "out"));
        break;
      case "debug":
        handleDebug(positional[0], flags);
        break;
      case "ir":
        handleIr(positional[0], json);
        break;
      case "init":
      case "build":
      case "test":
      case "add":
      case "remove":
      case "install":
      case "publish":
        handlePackage(command, positional, flags, json);
        break;
      case "registry":
        handleRegistry(positional, json);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        console.log(USAGE);
        process.exit(1);
    }
  } catch (err) {
    if (json) {
      console.log(JSON.stringify({ ok: false, error: String(err) }));
    } else {
      printError(err);
    }
    process.exit(1);
  }
}

function absPath(filePath: string | undefined): string {
  if (!filePath) {
    console.error("Error: missing file path");
    console.log(USAGE);
    process.exit(1);
  }
  return resolve(filePath);
}

function handleCheck(filePath: string | undefined, json: boolean): void {
  const abs = absPath(filePath);
  if (isCliAvailable()) {
    const result = runNativeCli(json ? ["check", "--json", abs] : ["check", abs]);
    if (json) {
      console.log(result.stdout ?? "");
    } else {
      process.stdout.write(result.stdout ?? "");
      process.stderr.write(result.stderr ?? "");
    }
    process.exit(result.status === 0 ? 0 : 1);
  }
  compileFile(abs);
  if (json) {
    console.log(JSON.stringify({ ok: true, diagnostics: [] }));
  } else {
    console.log(`✓ ${filePath} — no type errors`);
  }
}

function handleVerify(filePath: string | undefined, json: boolean, flags: Map<string, string | boolean>): void {
  requireNative("Hardware verification requires the native Rust CLI.");
  const abs = absPath(filePath);
  const extra: string[] = [];
  const target = flagStr(flags, "target");
  if (target) extra.push("--target", target);
  if (flagBool(flags, "all-targets")) extra.push("--all-targets");
  if (flagBool(flags, "simulate")) extra.push("--simulate");
  if (json) extra.push("--json");

  const result = runNativeCli(["verify", abs, ...extra]);
  if (json) {
    console.log(result.stdout ?? "");
  } else {
    printVerifyHuman(JSON.parse(result.stdout || "{}") as VerifyResult, filePath!);
  }
  process.exit(result.status === 0 ? 0 : 1);
}

function printVerifyHuman(result: VerifyResult, filePath: string): void {
  const compatible = result.compatible ?? result.ok;
  console.log(`\nHardware compatibility: ${filePath}`);
  if (result.target) console.log(`  Target: ${result.target}`);
  console.log(`  Status: ${compatible ? "COMPATIBLE" : "INCOMPATIBLE"}\n`);
  for (const item of result.items) {
    const icon = item.severity === "pass" ? "✓" : item.severity === "warning" ? "⚠" : "✗";
    console.log(`  ${icon} [${item.category}] ${item.message}`);
  }
  if (result.matrix?.cells.length) {
    console.log("\n  Compatibility matrix:");
    for (const cell of result.matrix.cells) {
      console.log(`    ${cell.robot} × ${cell.target}: ${cell.compatible ? "ok" : "fail"}`);
    }
  }
  console.log();
}

function handleRun(filePath: string | undefined, verbose: boolean, json: boolean, extraVerbose: boolean): void {
  const abs = absPath(filePath);
  const showVerbose = verbose || extraVerbose;

  if (isCliAvailable() && json) {
    const args = ["run", "--json", abs];
    if (showVerbose) args.push("--verbose");
    const result = runNativeCli(args);
    console.log(result.stdout ?? "");
    process.exit(result.status === 0 ? 0 : 1);
  }

  runSimulation(abs, filePath!, showVerbose);
}

function runSimulation(absPath: string, displayPath: string, verbose: boolean): void {
  const { program } = compileFile(absPath);
  const robot = program.robots[0];
  if (!robot) {
    console.error("No robot defined in program");
    process.exit(1);
  }

  const sim = createDefaultSimulator();
  const logs: string[] = [];

  console.log(`\n🤖 Running robot "${robot.name}" from ${displayPath}\n`);

  const state = run(program, {
    backend: sim,
    maxLoopIterations: verbose ? 20 : 10,
    onLog: (msg) => logs.push(msg),
    onMotionBlocked: (reason) => logs.push(`⚠ BLOCKED: ${reason}`),
  });

  console.log("── Final State ──");
  console.log(`  Pose:     x=${state.pose.x.toFixed(3)} m, y=${state.pose.y.toFixed(3)} m, θ=${state.pose.theta.toFixed(3)} rad`);
  if (state.pose.z !== undefined) {
    console.log(`  Altitude: z=${state.pose.z.toFixed(3)} m`);
  }
  console.log(`  Velocity: linear=${state.velocity.linear.toFixed(3)} m/s, angular=${state.velocity.angular.toFixed(3)} rad/s`);
  console.log(`  E-stop:   ${state.emergencyStop ? "ACTIVE" : "off"}`);

  if (verbose) {
    console.log("\n── Simulation Log ──");
    for (const event of sim.getEventLog()) {
      console.log(`  ${event}`);
    }
    if (logs.length > 0) {
      console.log("\n── Runtime Log ──");
      for (const log of logs) {
        console.log(`  ${log}`);
      }
    }
  }

  console.log("\n✓ Simulation complete\n");
}

function handleFmt(filePath: string | undefined, json: boolean): void {
  requireNative("Formatting requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const result = fmtViaCli(source);
  if (json) {
    console.log(JSON.stringify(result));
  } else if (result.changed) {
    writeFileSync(abs, result.formatted);
    console.log(`✓ Formatted ${filePath}`);
  } else {
    console.log(`✓ ${filePath} — already formatted`);
  }
  process.exit(result.ok ? 0 : 1);
}

function handleLint(filePath: string | undefined, json: boolean): void {
  requireNative("Linting requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const result = lintViaCli(source);
  if (json) {
    console.log(JSON.stringify(result));
  } else {
    if (result.ok) {
      console.log(`✓ ${filePath} — no lint issues`);
    } else {
      console.error(`Lint issues in ${filePath}:`);
      for (const issue of result.issues) {
        console.error(`  [${issue.line}:${issue.column}] ${issue.severity}: ${issue.message} (${issue.rule})`);
      }
    }
  }
  process.exit(result.ok ? 0 : 1);
}

function handleDoc(filePath: string | undefined, json: boolean, outPath: string | undefined): void {
  requireNative("Documentation generation requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const result = docViaCli(source);
  if (json) {
    console.log(JSON.stringify(result));
  } else if (outPath) {
    writeFileSync(resolve(outPath), result.markdown);
    console.log(`✓ Wrote documentation to ${outPath}`);
  } else {
    console.log(result.markdown);
  }
  process.exit(result.ok ? 0 : 1);
}

function handleCodegen(filePath: string | undefined, target: string | undefined, outPath: string | undefined): void {
  requireNative("Codegen requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const t = (target ?? "native") as "native" | "wasm" | "esp32";
  const output = codegenViaCli(source, t);
  if (outPath) {
    writeFileSync(resolve(outPath), output);
    console.log(`✓ Wrote codegen output to ${outPath}`);
  } else {
    console.log(output);
  }
}

function handleDeploy(filePath: string | undefined, outPath: string | undefined): void {
  requireNative("Deploy requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const manifest = deployViaCli(source);
  if (outPath) {
    writeFileSync(resolve(outPath), manifest);
    console.log(`✓ Wrote WASM deploy manifest to ${outPath}`);
  } else {
    console.log(manifest);
  }
}

function handleIr(filePath: string | undefined, json: boolean): void {
  requireNative("Spanda IR lowering requires the native Rust CLI.");
  const abs = absPath(filePath);
  const args = ["ir", abs];
  if (json) args.push("--json");
  const result = runNativeCli(args);
  if (json) {
    console.log(result.stdout ?? "");
  } else {
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
  }
  process.exit(result.status === 0 ? 0 : 1);
}

function handleDebug(filePath: string | undefined, flags: Map<string, string | boolean>): void {
  requireNative("Debug requires the native Rust CLI.");
  const abs = absPath(filePath);
  const source = readFileSync(abs, "utf-8");
  const breakpoints: number[] = [];
  const br = flags.get("break");
  if (typeof br === "string") breakpoints.push(Number(br));
  const result = debugViaCli(source, breakpoints);
  if (result.pauses.length === 0) {
    console.log("✓ Debug session completed (no breakpoints hit)");
  } else {
    console.log("Debug pauses:");
    for (const p of result.pauses) {
      console.log(`  line ${p.line} — ${p.reason}`);
    }
  }
  process.exit(result.ok ? 0 : 1);
}

function handlePackage(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
  json: boolean,
): void {
  requireNative("Package commands require the native Rust CLI.");
  const args = [command];
  if (json) args.push("--json");
  const project = flagStr(flags, "project");
  if (project) args.push("--project", project);
  const description = flagStr(flags, "description");
  if (description) args.push("--description", description);
  const version = flagStr(flags, "version");
  if (version) args.push("--version", version);
  const pathFlag = flagStr(flags, "path");
  if (pathFlag) args.push("--path", pathFlag);
  const git = flagStr(flags, "git");
  if (git) args.push("--git", git);
  args.push(...positional);

  const result = runNativeCli(args);
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  process.exit(result.status === 0 ? 0 : 1);
}

function handleRegistry(positional: string[], json: boolean): void {
  requireNative("Registry commands require the native Rust CLI.");
  const sub = positional[0];
  if (sub === "search") {
    const query = positional[1];
    if (!query) {
      console.error("Error: missing search query");
      process.exit(1);
    }
    const args = ["registry", "search", query];
    if (json) args.push("--json");
    const result = runNativeCli(args);
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    process.exit(result.status === 0 ? 0 : 1);
  } else if (sub === "info") {
    const pkg = positional[1];
    if (!pkg) {
      console.error("Error: missing package name");
      process.exit(1);
    }
    const result = runNativeCli(["registry", "info", pkg]);
    console.log(result.stdout ?? "");
    process.exit(result.status === 0 ? 0 : 1);
  } else {
    console.error("Usage: spanda registry search <query> | spanda registry info <package>");
    process.exit(1);
  }
}

function printError(err: unknown): void {
  if (err instanceof LexerError) {
    console.error(`Lexer error [${err.line}:${err.column}]: ${err.message}`);
  } else if (err instanceof ParseError) {
    console.error(`Parse error [${err.line}:${err.column}]: ${err.message}`);
  } else if (err instanceof TypeCheckError) {
    console.error("Type errors:");
    for (const e of err.errors) {
      console.error(`  [${e.line}:${e.column}] ${e.message}`);
    }
  } else if (err instanceof RuntimeError) {
    console.error(`Runtime error [line ${err.line}]: ${err.message}`);
  } else if (err instanceof Error) {
    console.error(`Error: ${err.message}`);
  } else {
    console.error(String(err));
  }
}

main();
