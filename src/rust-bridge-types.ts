/**
 * Shared Rust CLI verify result types (language runtime layer).
 * @module
 */

export type CompatSeverity = "pass" | "warning" | "error";

export type CompatItem = {
  category: string;
  message: string;
  severity: CompatSeverity;
  line: number;
  column: number;
};

export type MatrixCell = {
  robot: string;
  target: string;
  compatible: boolean;
};

export type VerifyResult = {
  ok: boolean;
  compatible?: boolean;
  target?: string;
  items: CompatItem[];
  matrix?: { cells: MatrixCell[] };
};
