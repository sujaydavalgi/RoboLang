/**
 * Mission continuity snapshot types for checkpoint persistence.
 * @module
 */

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
