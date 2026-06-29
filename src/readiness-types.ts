/**
 * Readiness report types for compile-time span lookup and platform evaluation.
 * @module
 */

export type ReadinessSeverity = "Critical" | "High" | "Medium" | "Low" | "Info";
export type ReadinessStatus = "Ready" | "Degraded" | "NotReady" | "Unknown";

export type ReadinessIssue = {
  factor: string;
  severity: ReadinessSeverity;
  message: string;
  suggested_action?: string;
};

export type ReadinessFactorScore = {
  factor: string;
  score: number;
  weight: number;
  weighted: number;
};

export type ReadinessReport = {
  status: ReadinessStatus;
  mission_ready: boolean;
  score: { total: number; maximum: number; factors: ReadinessFactorScore[] };
  issues: ReadinessIssue[];
  target?: string;
  robots: string[];
};

export type ReadinessDashboard = {
  overall_score: number;
  mission_ready_count: number;
  degraded_count: number;
  not_ready_count: number;
  top_issues: string[];
  reports: ReadinessReport[];
};

export type ReadinessOptions = {
  target?: string;
  includeRuntime?: boolean;
  injectHealthFaults?: boolean;
  simulate?: boolean;
  strictCertify?: boolean;
};
