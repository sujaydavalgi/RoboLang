/**
 * TypeScript mission continuity analysis (native CLI fallback).
 * @module
 */

import type { Program } from "./ast/nodes.js";
import { evaluateReadinessTs, type ReadinessOptions } from "./readiness.js";

export type ContinuityTrigger =
  | "robot_failed"
  | "robot_degraded"
  | "device_disconnected"
  | "fleet_member_offline"
  | "swarm_member_lost"
  | "communication_interrupted"
  | "battery_critical"
  | "hardware_capability_lost";

export type TakeoverMode =
  | "resume"
  | "restart"
  | "partial_restart"
  | "shadow_takeover"
  | "hot_takeover"
  | "cold_takeover"
  | "human_takeover";

export type ContinuationDecision =
  | "continue"
  | "restart"
  | "partial_restart"
  | "abort"
  | "human_approval_required";

export type SuccessionScope =
  | "robot"
  | "device"
  | "fleet"
  | "swarm"
  | "group"
  | "crowd"
  | "mission_cluster";

export type ContinuityContext = {
  mission: string;
  failed_entity: string;
  trigger: ContinuityTrigger;
  progress_percent: number;
  scope: SuccessionScope;
  current_step?: string;
  checkpoints?: string[];
};

export type MissionCheckpoint = {
  name: string;
  progress_percent: number;
  mission_state: { plan: string; current_step: string | null; status: string };
  robot_state: string;
  health_state: string;
  safety_state: string;
  capability_state: string;
};

export type MissionStateSnapshot = {
  mission: string;
  completed_steps: string[];
  current_goal: string | null;
  progress_percent: number;
  checkpoints: MissionCheckpoint[];
};

export type MissionStateTransfer = {
  from_entity: string;
  to_entity: string;
  snapshot: MissionStateSnapshot;
  transferable: boolean;
  transfer_notes: string[];
};

export type SuccessorCandidate = {
  entity: string;
  scope: SuccessionScope;
  capability_match_percent: number;
  health: string;
  readiness_score: number;
  location_distance: number;
  battery_percent: number;
  connectivity: string;
  trust_score: number;
  eligible: boolean;
  blockers: string[];
};

export type SuccessorRanking = {
  candidate: SuccessorCandidate;
  composite_score: number;
  rank: number;
};

export type ContinuityEvidence = {
  takeover_evidence: string[];
  delegation_evidence: string[];
  continuity_evidence: string[];
  diagnosis?: string;
};

export type MissionContinuityReport = {
  mission: string;
  failed_entity: string;
  trigger: ContinuityTrigger;
  can_continue: boolean;
  decision: ContinuationDecision;
  takeover_mode: TakeoverMode;
  selected_successor: string | null;
  checkpoint: MissionCheckpoint | null;
  evidence: ContinuityEvidence;
  passed: boolean;
};

export type TakeoverReport = {
  mission: string;
  failed_entity: string;
  successor: string;
  mode: TakeoverMode;
  decision: ContinuationDecision;
  succeeded: boolean;
  diagnosis: string;
};

export type DelegationReport = {
  mission: string;
  from_entity: string;
  to_entity: string;
  scope: SuccessionScope;
  task_redistribution: string[];
  passed: boolean;
};

export type SuccessionReport = {
  mission: string;
  failed_entity: string;
  candidates: SuccessorCandidate[];
  rankings: SuccessorRanking[];
  selected: string | null;
  passed: boolean;
};

function fleetMembers(program: Program, failed: string): string[] {
  const names: string[] = [];
  for (const fleet of program.fleets ?? []) {
    if (fleet.kind === "FleetDecl") {
      for (const m of fleet.members) {
        if (m !== failed) names.push(m);
      }
    }
  }
  if (names.length === 0) {
    for (const robot of program.robots ?? []) {
      if (robot.kind === "RobotDecl" && robot.name !== failed) {
        names.push(robot.name);
      }
    }
  }
  return names;
}

function evaluateCandidate(
  program: Program,
  entity: string,
  scope: SuccessionScope,
): SuccessorCandidate {
  const readiness = evaluateReadinessTs(program, {} as ReadinessOptions);
  const score = readiness.score?.total ?? 0;
  const status = readiness.status ?? "NotReady";
  const blockers: string[] = [];
  if (status === "NotReady") blockers.push("Successor not mission-ready");

  return {
    entity,
    scope,
    capability_match_percent: 100,
    health: status === "Ready" ? "Healthy" : status === "Degraded" ? "Degraded" : "Unhealthy",
    readiness_score: score,
    location_distance: entity.length * 2,
    battery_percent: 85,
    connectivity: "Connected",
    trust_score: 92,
    eligible: blockers.length === 0,
    blockers,
  };
}

function rankCandidates(candidates: SuccessorCandidate[]): SuccessorRanking[] {
  const ranked = candidates
    .filter((c) => c.eligible)
    .map((c) => ({
      candidate: c,
      composite_score:
        c.capability_match_percent * 0.25 +
        c.readiness_score * 0.2 +
        c.battery_percent * 0.1 +
        c.trust_score * 0.15,
      rank: 0,
    }))
    .sort((a, b) => b.composite_score - a.composite_score);

  ranked.forEach((r, i) => {
    r.rank = i + 1;
  });
  return ranked;
}

function inferMode(context: ContinuityContext): TakeoverMode {
  if (context.trigger === "battery_critical") return "hot_takeover";
  return context.progress_percent > 0 ? "resume" : "restart";
}

function inferDecision(context: ContinuityContext, safe: boolean): ContinuationDecision {
  if (!safe) return "abort";
  if (context.progress_percent > 0) return "continue";
  return "restart";
}

/**
 * Evaluate mission continuity for a program and context.
 */
export function evaluateContinuityTs(
  program: Program,
  context: ContinuityContext,
): MissionContinuityReport {
  const candidates = fleetMembers(program, context.failed_entity).map((e) =>
    evaluateCandidate(program, e, context.scope),
  );
  const rankings = rankCandidates(candidates);
  const selected = rankings[0]?.candidate.entity ?? null;
  const mode = inferMode(context);
  const safe = selected !== null;
  const decision = inferDecision(context, safe);

  return {
    mission: context.mission,
    failed_entity: context.failed_entity,
    trigger: context.trigger,
    can_continue: safe && decision !== "abort",
    decision,
    takeover_mode: mode,
    selected_successor: selected,
    checkpoint:
      context.progress_percent > 0
        ? {
            name: `checkpoint_${context.progress_percent}pct`,
            progress_percent: context.progress_percent,
            mission_state: {
              plan: context.mission,
              current_step: context.current_step ?? null,
              status: "in_progress",
            },
            robot_state: "last_known",
            health_state: "unknown",
            safety_state: "last_validated",
            capability_state: "matched",
          }
        : null,
    evidence: {
      takeover_evidence: selected ? [`Successor: ${selected}`, `Mode: ${mode}`] : [],
      delegation_evidence: [],
      continuity_evidence: [`Evaluated ${candidates.length} candidates`],
      diagnosis: `Continuity evaluation for ${context.failed_entity}`,
    },
    passed: safe && decision !== "abort",
  };
}

/**
 * Plan takeover for a failed entity.
 */
export function planTakeoverTs(
  program: Program,
  context: ContinuityContext,
  successor?: string,
): TakeoverReport {
  const continuity = evaluateContinuityTs(program, context);
  const selected = successor ?? continuity.selected_successor ?? "NoSuccessor";
  return {
    mission: context.mission,
    failed_entity: context.failed_entity,
    successor: selected,
    mode: continuity.takeover_mode,
    decision: continuity.decision,
    succeeded: continuity.passed,
    diagnosis: `Takeover triggered by ${context.trigger} on ${context.failed_entity}`,
  };
}

/**
 * Plan mission delegation.
 */
export function planDelegationTs(
  program: Program,
  context: ContinuityContext,
  toEntity?: string,
): DelegationReport {
  const to = toEntity ?? "BackupRobot";
  return {
    mission: context.mission,
    from_entity: context.failed_entity,
    to_entity: to,
    scope: context.scope,
    task_redistribution: [
      `Transfer mission ownership to ${to}`,
      `Resume from ${context.progress_percent}% progress`,
    ],
    passed: true,
  };
}

/**
 * Plan successor succession.
 */
export function planSuccessionTs(
  program: Program,
  context: ContinuityContext,
): SuccessionReport {
  const candidates = fleetMembers(program, context.failed_entity).map((e) =>
    evaluateCandidate(program, e, context.scope),
  );
  const rankings = rankCandidates(candidates);
  return {
    mission: context.mission,
    failed_entity: context.failed_entity,
    candidates,
    rankings,
    selected: rankings[0]?.candidate.entity ?? null,
    passed: rankings.length > 0,
  };
}
