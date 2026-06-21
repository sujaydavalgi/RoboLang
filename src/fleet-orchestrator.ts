/**
 * Fleet orchestration beyond in-process fleet run.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import {
  createMissionRuntime,
  missionAdvance,
  missionStart,
  type MissionRuntime,
} from "./robotics-platform.js";

export type FleetMemberState = {
  robotName: string;
  missionName: string | null;
  missionState: string;
  currentStep: string;
  hasPeerLink: boolean;
  peerHandoffs?: string[];
};

export type PeerDelivery = {
  fromRobot: string;
  toRobot: string;
  topic: string;
  step: string;
  delivered: boolean;
};

export type FleetOrchestrationReport = {
  fleetName: string;
  members: FleetMemberState[];
  coordinationMode: string;
  stepsAdvanced: number;
  peerMessages?: string[];
  peerDeliveries?: PeerDelivery[];
  remoteRelayed?: number;
  remoteFailed?: number;
};

export type FleetOrchestrationResult = {
  program: string;
  fleets: FleetOrchestrationReport[];
  success: boolean;
};

function missionForRobot(robot: Program["robots"][number]): MissionRuntime | null {
  if (!robot.mission) return null;
  return createMissionRuntime(
    robot.mission.name,
    [...robot.mission.steps],
    robot.mission.durationHours,
  );
}

export function orchestrateFleets(program: Program, programPath: string): FleetOrchestrationResult {
  // Coordinate declared fleet groups using each member robot's mission controller.
  const reports: FleetOrchestrationReport[] = [];

  for (const fleet of program.fleets) {
    const members: FleetMemberState[] = [];
    let stepsAdvanced = 0;
    const peerMessages: string[] = [];
    const peerDeliveries: PeerDelivery[] = [];

    for (const memberName of fleet.members) {
      const robot = program.robots.find((r) => r.name === memberName);
      if (!robot) {
        members.push({
          robotName: memberName,
          missionName: null,
          missionState: "MissingRobot",
          currentStep: "",
          hasPeerLink: false,
        });
        continue;
      }

      const runtime = missionForRobot(robot);
      if (runtime) {
        missionStart(runtime);
        const step = missionAdvance(runtime);
        if (step) stepsAdvanced += 1;
        const peerHandoffs = (robot.peerRobots ?? []).flatMap((peer) =>
          step ? [`${memberName}->${peer.name}:step=${step}`] : [],
        );
        peerMessages.push(...peerHandoffs);
        for (const peer of robot.peerRobots ?? []) {
          if (!step) continue;
          peerDeliveries.push({
            fromRobot: memberName,
            toRobot: peer.name,
            topic: "mission_step",
            step,
            delivered: true,
          });
        }
        members.push({
          robotName: memberName,
          missionName: runtime.name,
          missionState: runtime.state,
          currentStep: step,
          hasPeerLink: (robot.peerRobots?.length ?? 0) > 0,
          peerHandoffs,
        });
      } else {
        members.push({
          robotName: memberName,
          missionName: null,
          missionState: "NoMission",
          currentStep: "",
          hasPeerLink: (robot.peerRobots?.length ?? 0) > 0,
        });
      }
    }

    const hasPeerLink = members.some((m) => m.hasPeerLink);
    reports.push({
      fleetName: fleet.name,
      members,
      coordinationMode: peerDeliveries.length > 0
        ? "peer_mesh_mission"
        : hasPeerLink
          ? "peer_round_robin_mission"
          : "round_robin_mission",
      stepsAdvanced,
      peerMessages,
      peerDeliveries,
    });
  }

  const success = reports.every((r) =>
    r.members.every((m) => m.missionState !== "MissingRobot"),
  );

  return { program: programPath, fleets: reports, success };
}

export async function orchestrateFleetsMesh(
  program: Program,
  programPath: string,
  meshUrl: string,
  token?: string,
): Promise<FleetOrchestrationResult> {
  const { relayDeliveriesViaMesh } = await import("./fleet-mesh.js");
  const result = orchestrateFleets(program, programPath);
  let success = result.success;
  for (const fleet of result.fleets) {
    if (!fleet.peerDeliveries?.length) continue;
    try {
      const resp = await relayDeliveriesViaMesh(meshUrl, fleet.peerDeliveries, token);
      fleet.remoteRelayed = resp.relayed;
      fleet.remoteFailed = resp.failed;
      if (resp.relayed > 0) fleet.coordinationMode = "distributed_peer_mesh";
      if (resp.failed > 0) success = false;
    } catch {
      fleet.remoteFailed = fleet.peerDeliveries.length;
      success = false;
    }
  }
  return { ...result, success };
}

export async function orchestrateFleetsRemote(
  program: Program,
  programPath: string,
  registry: import("./fleet-remote.js").FleetAgentRegistry,
): Promise<FleetOrchestrationResult> {
  // Coordinate locally, then push peer mission steps to remote fleet agents.
  const { relayPeerDeliveries } = await import("./fleet-remote.js");
  const result = orchestrateFleets(program, programPath);
  let success = result.success;
  for (const fleet of result.fleets) {
    const { relayed, failed } = await relayPeerDeliveries(fleet.peerDeliveries ?? [], registry);
    fleet.remoteRelayed = relayed;
    fleet.remoteFailed = failed;
    if (relayed > 0) {
      fleet.coordinationMode = "distributed_peer_mesh";
    }
    if (failed > 0) {
      success = false;
    }
  }
  return { ...result, success };
}
