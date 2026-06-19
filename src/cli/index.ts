#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { compileFile, run } from "../compile.js";
import { createDefaultSimulator } from "../simulator/index.js";
import { LexerError } from "../lexer/index.js";
import { ParseError } from "../parser/index.js";
import { TypeCheckError } from "../types/index.js";
import { RuntimeError } from "../runtime/index.js";

const USAGE = `Spanda Programming Language — the pulse of autonomous intelligence

Usage:
  spanda run <file.sd>   Run program with interpreter (simulated backend)
  spanda sim <file.sd>   Run program in simulation mode with visual output
  spanda check <file.sd> Type-check a Spanda source file

Examples:
  spanda run examples/rover.sd
  spanda sim examples/rover.sd
  spanda check examples/rover.sd
`;

function main(): void {
  const args = process.argv.slice(2);
  const command = args[0];
  const filePath = args[1];

  if (!command || command === "--help" || command === "-h") {
    console.log(USAGE);
    process.exit(0);
  }

  if (!filePath) {
    console.error("Error: missing file path");
    console.log(USAGE);
    process.exit(1);
  }

  const absPath = resolve(filePath);

  try {
    switch (command) {
      case "check":
        compileFile(absPath);
        console.log(`✓ ${filePath} — no type errors`);
        break;

      case "run":
      case "sim":
        runSimulation(absPath, command === "sim");
        break;

      default:
        console.error(`Unknown command: ${command}`);
        console.log(USAGE);
        process.exit(1);
    }
  } catch (err) {
    printError(err);
    process.exit(1);
  }
}

function runSimulation(filePath: string, verbose: boolean): void {
  const { program } = compileFile(filePath);
  const robot = program.robots[0];
  if (!robot) {
    console.error("No robot defined in program");
    process.exit(1);
  }

  const sim = createDefaultSimulator();
  const logs: string[] = [];

  console.log(`\n🤖 Running robot "${robot.name}" from ${filePath}\n`);

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
