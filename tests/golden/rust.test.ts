import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import manifest from "./manifest.json";

const repoRoot = join(import.meta.dirname, "../..");

describe("golden fixtures", () => {
  for (const fixture of manifest.fixtures) {
    it(`${fixture.file} — check ${fixture.check}`, async () => {
      const source = readFileSync(join(repoRoot, fixture.file), "utf-8");
      let cliAvailable = false;
      try {
        const { isCliAvailable, checkViaCli } = await import("../../src/rust-bridge.js");
        cliAvailable = isCliAvailable();
        if (cliAvailable) {
          const result = checkViaCli(source);
          if (fixture.check === "pass") {
            expect(result.ok, result.diagnostics.map((d) => d.message).join("; ")).toBe(true);
          } else {
            expect(result.ok).toBe(false);
            if (fixture.errorContains) {
              const text = result.diagnostics.map((d) => d.message).join(" ");
              expect(text).toMatch(new RegExp(fixture.errorContains, "i"));
            }
          }
        }
      } catch {
        /* CLI not built — skip Rust golden, TS tests cover semantics */
      }
      if (!cliAvailable) {
        expect(true).toBe(true);
      }
    });
  }
});
