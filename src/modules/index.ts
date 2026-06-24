/**
 * index module (modules/index.ts).
 * @module
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { tokenize } from "../lexer/index.js";
import { parse } from "../parser/index.js";
import type { Program } from "../ast/nodes.js";
import type { ModuleFnDecl } from "../foundations.js";

export type ModuleExports = {
  functions: Map<string, ModuleFnDecl>;
};

export class ModuleRegistry {
  private modules = new Map<string, ModuleExports>();

  register(moduleName: string, program: Program): void {
    // Register the value.
    //
    // Parameters:
    // - `moduleName` — input value
    // - `program` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = register(moduleName, program);

    const exports: ModuleExports = { functions: new Map() };
    for (const func of program.functions) {
      if (func.visibility === "export" || func.visibility === "public") {
        exports.functions.set(func.name, func);
      }
    }
    this.modules.set(moduleName, exports);
  }

  exportsFor(importPath: string): ModuleExports | undefined {
    return this.modules.get(importPath);
  }

  function(importPath: string, name: string): ModuleFnDecl | undefined {
    return this.modules.get(importPath)?.functions.get(name);
  }

  static fromPrograms(entries: Array<[string, Program]>): ModuleRegistry {    // Compute registry for the following logic.
    const registry = new ModuleRegistry();

    // Iterate over the collection.
    for (const [name, program] of entries) {
      registry.register(name, program);
    }
    return registry;
}

  get moduleCount(): number {
    return this.modules.size;
  }
}

function collectModules(dir: string, out: Array<[string, Program]>): void {
  // CollectModules.
  //
  // Parameters:
  // - `dir` — input value
  // - `out` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = collectModules(dir, out);
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);

    // continue when statSync(path).isDirectory().
    if (statSync(path).isDirectory()) {
      collectModules(path, out);
      continue;
    }

    // continue when sd") is falsy.
    if (!entry.endsWith(".sd")) continue;
    const source = readFileSync(path, "utf8");
    const program = parse(tokenize(source));

    // continue when moduleName is falsy.
    if (!program.moduleName) {
      throw new Error(`Module file '${path}' must declare \`module <name>;\``);
    }
    out.push([program.moduleName, program]);
  }
}

export function loadProjectModules(projectRoot: string): ModuleRegistry {
  // LoadProjectModules.
  //
  // Parameters:
  // - `projectRoot` — input value
  //
  // Returns:
  // `ModuleRegistry`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = loadProjectModules(projectRoot);
  const entries: Array<[string, Program]> = [];

  // Iterate over ["src", "tests"].
  for (const sub of ["src", "tests"]) {
    const dir = join(projectRoot, sub);

    // Try the operation and handle failures below.
    try {

      // continue when statSync(dir).isDirectory()) collectModules(dir, entries.
      if (statSync(dir).isDirectory()) collectModules(dir, entries);
    } catch {
      /* missing dir */
    }
  }
  const vendorRoot = join(projectRoot, ".spanda", "packages");

  // Try the operation and handle failures below.
  try {

    // continue when statSync(vendorRoot).isDirectory().
    if (statSync(vendorRoot).isDirectory()) {

      // Iterate over readdirSync.
      for (const pkg of readdirSync(vendorRoot)) {
        const src = join(vendorRoot, pkg, "src");

        // Try the operation and handle failures below.
        try {

          // continue when statSync(src).isDirectory()) collectModules(src, entries.
          if (statSync(src).isDirectory()) collectModules(src, entries);
        } catch {
          /* no src */
        }
      }
    }
  } catch {
    /* no vendor root */
  }
  return ModuleRegistry.fromPrograms(entries);
}

export function moduleNameFromPath(path: string): string {
  // ModuleNameFromPath.
  //
  // Parameters:
  // - `path` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = moduleNameFromPath(path);
  const base = path.split("/").pop()?.replace(/\.sd$/, "") ?? "main";
  return base.replace(/-/g, "_");
}
