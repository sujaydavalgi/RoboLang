/**
 * Span lookup for readiness diagnostics (TypeScript mirror of spanda-readiness spans).
 * @module
 */

import type { Program } from "./ast/nodes.js";
import type { ReadinessIssue } from "./readiness-types.js";

type Spanned = { span: { start: { line: number; column: number } } };

function atSpan(node?: Spanned): {
  // Description:
  //     AtSpan.
  //
  // Inputs:
  //     node?: Spanned
  //         Caller-supplied node?.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = atSpan(node?);
 line: number; column: number } {
  return node ? { line: node.span.start.line, column: node.span.start.column } : { line: 1, column: 1 };
}

function deploySpan(program: Program): {
  // Description:
  //     DeploySpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = deploySpan(program);
 line: number; column: number } | undefined {
  const deploy = program.deployments?.[0];
  if (deploy && deploy.kind === "DeployDecl") {
    return atSpan(deploy);
  }
  return undefined;
}

function firstRobotSpan(program: Program): {
  // Description:
  //     FirstRobotSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = firstRobotSpan(program);
 line: number; column: number } {
  return atSpan(program.robots?.[0]);
}

function missionSpan(program: Program): Spanned | undefined {
  // Description:
  //     MissionSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `missionSpan`.
  //
  // Example:

  //     const result = missionSpan(program);

  const mission = program.robots?.find((r) => r.mission)?.mission;
  return mission ?? undefined;
}

function firstRobotSafetySpan(program: Program): Spanned | undefined {
  // Description:
  //     FirstRobotSafetySpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstRobotSafetySpan`.
  //
  // Example:

  //     const result = firstRobotSafetySpan(program);

  for (const robot of program.robots ?? []) {
    if (robot.safety) return robot.safety;
  }
  return undefined;
}

function firstAssuranceCaseSpan(program: Program): Spanned | undefined {
  // Description:
  //     FirstAssuranceCaseSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstAssuranceCaseSpan`.
  //
  // Example:

  //     const result = firstAssuranceCaseSpan(program);

  return program.assuranceCases?.[0];
}

function firstAssuranceCaseWithoutEvidence(program: Program): Spanned | undefined {
  // Description:
  //     FirstAssuranceCaseWithoutEvidence.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstAssuranceCaseWithoutEvidence`.
  //
  // Example:

  //     const result = firstAssuranceCaseWithoutEvidence(program);

  return program.assuranceCases?.find((c) => c.evidence.length === 0);
}

function firstKnowledgeModelSpan(program: Program): Spanned | undefined {
  // Description:
  //     FirstKnowledgeModelSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstKnowledgeModelSpan`.
  //
  // Example:

  //     const result = firstKnowledgeModelSpan(program);

  return program.knowledgeModels?.[0];
}

function firstEmptyKnowledgeModel(program: Program): Spanned | undefined {
  // Description:
  //     FirstEmptyKnowledgeModel.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstEmptyKnowledgeModel`.
  //
  // Example:

  //     const result = firstEmptyKnowledgeModel(program);

  return program.knowledgeModels?.find((m) => m.components.length === 0);
}

function firstAnomalyDetectorSpan(program: Program): Spanned | undefined {
  // Description:
  //     FirstAnomalyDetectorSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstAnomalyDetectorSpan`.
  //
  // Example:

  //     const result = firstAnomalyDetectorSpan(program);

  return program.anomalyDetectors?.[0];
}

function anomalyDetectorSpan(program: Program, name: string): Spanned | undefined {
  // Description:
  //     AnomalyDetectorSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //     name: string
  //         Caller-supplied name.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `anomalyDetectorSpan`.
  //
  // Example:

  //     const result = anomalyDetectorSpan(program, name);

  return program.anomalyDetectors?.find((d) => d.name === name);
}

function firstMitigationSpan(program: Program): Spanned | undefined {
  // Description:
  //     FirstMitigationSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstMitigationSpan`.
  //
  // Example:

  //     const result = firstMitigationSpan(program);

  return program.mitigations?.[0];
}

function firstStateEstimatorSpan(program: Program): Spanned | undefined {
  // Description:
  //     FirstStateEstimatorSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstStateEstimatorSpan`.
  //
  // Example:

  //     const result = firstStateEstimatorSpan(program);

  return program.stateEstimators?.[0];
}

function firstEmptyStateEstimator(program: Program): Spanned | undefined {
  // Description:
  //     FirstEmptyStateEstimator.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `firstEmptyStateEstimator`.
  //
  // Example:

  //     const result = firstEmptyStateEstimator(program);

  return program.stateEstimators?.find((e) => e.inputs.length === 0);
}

function stateEstimatorSpan(program: Program, name: string): Spanned | undefined {
  // Description:
  //     StateEstimatorSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //     name: string
  //         Caller-supplied name.
  //
  // Outputs:
  //     result: Spanned | undefined
  //         Return value from `stateEstimatorSpan`.
  //
  // Example:

  //     const result = stateEstimatorSpan(program, name);

  return program.stateEstimators?.find((e) => e.name === name);
}

function assuranceSpan(program: Program): {
  // Description:
  //     AssuranceSpan.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = assuranceSpan(program);
 line: number; column: number } | undefined {
  const node =
    firstAssuranceCaseSpan(program) ??
    firstKnowledgeModelSpan(program) ??
    firstStateEstimatorSpan(program) ??
    firstAnomalyDetectorSpan(program) ??
    firstMitigationSpan(program);
  return node ? atSpan(node) : undefined;
}

function extractQuotedName(message: string, prefix: string): string | undefined {
  // Description:
  //     ExtractQuotedName.
  //
  // Inputs:
  //     message: string
  //         Caller-supplied message.
  //     prefix: string
  //         Caller-supplied prefix.
  //
  // Outputs:
  //     result: string | undefined
  //         Return value from `extractQuotedName`.
  //
  // Example:

  //     const result = extractQuotedName(message, prefix);

  if (!message.startsWith(prefix)) return undefined;
  const rest = message.slice(prefix.length);
  const end = rest.indexOf("'");
  return end >= 0 ? rest.slice(0, end) : undefined;
}

/** Resolve a display line/column for a readiness issue factor. */
export function lineColumnForFactor(program: Program, factor: string): {
  // Description:
  //     LineColumnForFactor.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //     factor: string
  //         Caller-supplied factor.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = lineColumnForFactor(program, factor);
 line: number; column: number } {
  switch (factor) {
    case "Hardware":
    case "Battery":
    case "Connectivity":
    case "Storage":
    case "Compute":
    case "Packages":
    case "Providers":
      return deploySpan(program) ?? firstRobotSpan(program);
    case "Health": {
      const health = program.healthChecks?.[0];
      return health ? atSpan(health) : firstRobotSpan(program);
    }
    case "Capabilities":
    case "Mission Requirements": {
      const mission = missionSpan(program);
      return mission ? atSpan(mission) : firstRobotSpan(program);
    }
    case "Safety": {
      const safety = firstRobotSafetySpan(program);
      return safety ? atSpan(safety) : firstRobotSpan(program);
    }
    case "Fleet": {
      const fleet = program.fleets?.[0];
      return fleet ? atSpan(fleet) : { line: 1, column: 1 };
    }
    case "Assurance":
      return assuranceSpan(program) ?? { line: 1, column: 1 };
    default:
      return { line: 1, column: 1 };
  }
}

/** Resolve a precise line/column for a readiness issue using message context. */
export function lineColumnForIssue(
  program: Program,
  issue: ReadinessIssue,
): {
  // Description:
  //     LineColumnForIssue.
  //
  // Inputs:
  //     program: Program
  //         Caller-supplied program.
  //     issue: ReadinessIssue
  //         Caller-supplied issue.
  //
  // Outputs:
  //     None.
  //
  // Example:

 // const result = lineColumnForIssue(program, issue);
 line: number; column: number } {
  if (issue.factor === "Assurance") {
    const detectorName = extractQuotedName(issue.message, "Anomaly detector '");
    if (detectorName) {
      const detector = anomalyDetectorSpan(program, detectorName);
      if (detector) return atSpan(detector);
    }
    if (issue.message.includes("Assurance case")) {
      const decl = firstAssuranceCaseWithoutEvidence(program);
      if (decl) return atSpan(decl);
    }
    if (issue.message.includes("Knowledge model")) {
      const decl = firstEmptyKnowledgeModel(program);
      if (decl) return atSpan(decl);
    }
    const estimatorName = extractQuotedName(issue.message, "State estimator '");
    if (estimatorName) {
      const estimator = stateEstimatorSpan(program, estimatorName);
      if (estimator) return atSpan(estimator);
    }
    if (issue.message.includes("State estimator")) {
      const decl = firstEmptyStateEstimator(program);
      if (decl) return atSpan(decl);
    }
    const fallback = assuranceSpan(program);
    if (fallback) return fallback;
  }
  return lineColumnForFactor(program, issue.factor);
}
