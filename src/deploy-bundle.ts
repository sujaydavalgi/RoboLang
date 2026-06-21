/**
 * Signed OTA deploy artifact bundles.
 * @module
 */

import { createHash } from "node:crypto";
import type { DeployAssignment, DeployPlan } from "./deploy-service.js";

export type DeployArtifactBundle = {
  version: string;
  program: string;
  programHash?: string;
  assignments: DeployAssignment[];
  certifications: string[];
  signature?: string;
  publicKey?: string;
};

type BundleCanonicalBody = {
  version: string;
  program: string;
  program_hash?: string;
  assignments: Array<{ robot_name: string; hardware: string }>;
  certifications: string[];
};

function canonicalBody(bundle: DeployArtifactBundle): BundleCanonicalBody {
  return {
    version: bundle.version,
    program: bundle.program,
    program_hash: bundle.programHash,
    assignments: bundle.assignments.map((assignment) => ({
      robot_name: assignment.robotName,
      hardware: assignment.hardware,
    })),
    certifications: bundle.certifications,
  };
}

export function buildDeployBundle(plan: DeployPlan): DeployArtifactBundle {
  // Materialize the rollout manifest fields from a deploy plan.
  return {
    version: plan.version,
    program: plan.program,
    programHash: plan.programHash,
    assignments: plan.assignments,
    certifications: plan.certifications,
    signature: undefined,
    publicKey: undefined,
  };
}

export function bundleCanonicalJson(bundle: DeployArtifactBundle): string {
  return JSON.stringify(canonicalBody(bundle));
}

function seedBytes(material: string): Uint8Array {
  return createHash("sha256").update(material).digest();
}

export async function signDeployBundle(
  bundle: DeployArtifactBundle,
  keyMaterial: string,
): Promise<DeployArtifactBundle> {
  const { sign, publicKeyFromMaterial } = await import("./security/index.js");
  const canonical = bundleCanonicalJson(bundle);
  return {
    ...bundle,
    publicKey: publicKeyFromMaterial(keyMaterial),
    signature: sign(canonical, keyMaterial),
  };
}

export async function verifyDeployBundle(
  bundle: DeployArtifactBundle,
  keyMaterial: string,
): Promise<boolean> {
  if (!bundle.signature) return false;
  const { verifySignature } = await import("./security/index.js");
  return verifySignature(bundleCanonicalJson(bundle), bundle.signature, keyMaterial);
}
