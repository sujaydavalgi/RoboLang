/**
 * Runtime certification gate before executing deploy-target programs.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import { verifyCertificationProof } from "./certify-verify.js";

export function enforceCertificationRuntime(program: Program, strict: boolean): void {
  // Block run/sim when certification proof checklist reports errors.
  if (!strict) return;
  const blocking = verifyCertificationProof(program, true).find((item) => item.severity === "error");
  if (blocking) {
    throw new Error(`certification runtime gate: ${blocking.message}`);
  }
}

export function certificationRuntimeEnabledFromEnv(): boolean {
  const value = process.env.SPANDA_ENFORCE_CERTIFY?.toLowerCase();
  return value === "1" || value === "true" || value === "yes";
}
