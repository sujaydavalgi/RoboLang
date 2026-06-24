/**
 * Span-aware continuity policy diagnostics for IDE and check JSON fallbacks.
 * @module
 */

import type { Program } from "./ast/nodes.js";

export type ContinuityDiagnostic = {
  message: string;
  line: number;
  column: number;
  severity: string;
  category: string;
  suggested_fix?: string;
};

function normalizeAction(action: string): string {
  // Description:
  //     NormalizeAction.
  //
  // Inputs:
  //     action: string
  //         Continuity policy action text.
  //
  // Outputs:
  //     result: string
  //         Lowercased action with whitespace removed.
  //
  // Example:
  //     const key = normalizeAction("hot takeover");

  return action.toLowerCase().replace(/\s+/g, "");
}

function continuityActionIsHighRisk(action: string): boolean {
  // Description:
  //     ContinuityActionIsHighRisk.
  //
  // Inputs:
  //     action: string
  //         Continuity policy action text.
  //
  // Outputs:
  //     result: boolean
  //         True when the action implies hot, cold, or human takeover.
  //
  // Example:
  //     const risky = continuityActionIsHighRisk("hot takeover");

  const lower = normalizeAction(action);
  return (
    lower.includes("hottakeover") ||
    lower.includes("coldtakeover") ||
    lower.includes("humantakeover") ||
    lower.includes("operatortakeover")
  );
}

function robotHasApprovalTopic(program: Program): boolean {
  // Description:
  //     RobotHasApprovalTopic.
  //
  // Inputs:
  //     program: Program
  //         Parsed Spanda program.
  //
  // Outputs:
  //     result: boolean
  //         True when any robot exposes an Approval topic.
  //
  // Example:
  //     const approved = robotHasApprovalTopic(program);

  for (const robot of program.robots ?? []) {
    for (const topic of robot.topics ?? []) {
      if (topic.messageType === "Approval") {
        return true;
      }
    }
  }
  return false;
}

function recoveryHasHandoffAction(program: Program): boolean {
  // Description:
  //     RecoveryHasHandoffAction.
  //
  // Inputs:
  //     program: Program
  //         Parsed Spanda program.
  //
  // Outputs:
  //     result: boolean
  //         True when recovery reassigns or promotes fleet work.
  //
  // Example:
  //     const handoff = recoveryHasHandoffAction(program);

  for (const policy of program.recoveryPolicies ?? []) {
    for (const branch of policy.branches) {
      for (const action of branch.actions) {
        const lower = normalizeAction(action);
        if (
          lower.includes("reassign") ||
          lower.includes("promote") ||
          lower.includes("replace") ||
          lower.includes("redistribute")
        ) {
          return true;
        }
      }
    }
  }
  return false;
}

function fleetMemberCount(program: Program): number {
  // Description:
  //     FleetMemberCount.
  //
  // Inputs:
  //     program: Program
  //         Parsed Spanda program.
  //
  // Outputs:
  //     result: number
  //         Total fleet member references.
  //
  // Example:
  //     const members = fleetMemberCount(program);

  return (program.fleets ?? []).reduce((sum, fleet) => sum + fleet.members.length, 0);
}

/** Collect continuity-policy diagnostics mirroring the Rust assurance crate. */
export function collectContinuityDiagnostics(program: Program): ContinuityDiagnostic[] {
  // Description:
  //     CollectContinuityDiagnostics.
  //
  // Inputs:
  //     program: Program
  //         Parsed Spanda program.
  //
  // Outputs:
  //     result: ContinuityDiagnostic[]
  //         Span-aware continuity policy diagnostics.
  //
  // Example:
  //     const diags = collectContinuityDiagnostics(program);

  const diags: ContinuityDiagnostic[] = [];
  const continuityPolicies = program.continuityPolicies ?? [];
  const recoveryPolicies = program.recoveryPolicies ?? [];
  const fleets = program.fleets ?? [];
  const approvalPath = robotHasApprovalTopic(program);
  const hasContinuity = continuityPolicies.length > 0;
  const multiMemberFleet = fleetMemberCount(program) >= 2;

  if (multiMemberFleet && !hasContinuity) {
    const fleet = fleets[0];
    diags.push({
      message: "Fleet declared without continuity_policy for takeover and succession",
      line: fleet?.span?.start.line ?? 1,
      column: fleet?.span?.start.column ?? 1,
      severity: "warning",
      category: "continuity:policy",
      suggested_fix:
        "Add continuity_policy with on robot.failed { resume from checkpoint; reassign mission; }",
    });
  }

  if (recoveryHasHandoffAction(program) && multiMemberFleet && !hasContinuity) {
    const policy = recoveryPolicies[0];
    diags.push({
      message: "Recovery reassigns mission but no continuity_policy defines takeover mode",
      line: policy?.span?.start.line ?? 1,
      column: policy?.span?.start.column ?? 1,
      severity: "info",
      category: "continuity:handoff",
      suggested_fix:
        "Pair recovery reassign with continuity_policy for checkpoint resume and successor selection",
    });
  }

  for (const policy of continuityPolicies) {
    if (policy.branches.length === 0) {
      diags.push({
        message: `continuity_policy '${policy.name}' has no on branches`,
        line: policy.span.start.line,
        column: policy.span.start.column,
        severity: "warning",
        category: "continuity:policy",
        suggested_fix: "Add on robot.failed { resume from checkpoint; } branches",
      });
      continue;
    }
    for (const branch of policy.branches) {
      const triggerLower = branch.condition.toLowerCase();
      if (
        (triggerLower.includes("fleet") || triggerLower.includes("swarm")) &&
        fleets.length === 0
      ) {
        diags.push({
          message: `continuity_policy '${policy.name}' references fleet failures but no fleet is declared`,
          line: branch.span.start.line,
          column: branch.span.start.column,
          severity: "error",
          category: "continuity:fleet",
          suggested_fix: "Declare fleet <Name> { members; } or adjust trigger",
        });
      }
      for (const action of branch.actions) {
        if (continuityActionIsHighRisk(action) && !approvalPath) {
          diags.push({
            message: `High-risk continuity action '${action}' should have an Approval topic or operator path`,
            line: branch.span.start.line,
            column: branch.span.start.column,
            severity: "warning",
            category: "continuity:approval",
            suggested_fix:
              'Add topic approval: Approval subscribe on "/ops/approval"; or mission requires approval Operator',
          });
        }
      }
    }
  }

  return diags;
}
