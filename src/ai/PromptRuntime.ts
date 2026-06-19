import type { RuntimeValue } from "../runtime/interpreter.js";

export function buildPrompt(base: string, input?: RuntimeValue, goal?: string): string {
  const parts: string[] = [];
  if (goal?.trim()) {
    parts.push(`Goal: ${goal.trim()}`);
  }
  if (base.trim()) {
    parts.push(base.trim());
  }
  const inputSummary = summarizeInput(input);
  const header = parts.join("\n\n");
  return header ? `${header}\n\nContext:\n${inputSummary}` : `Context:\n${inputSummary}`;
}

function summarizeInput(input?: RuntimeValue): string {
  if (!input || input.kind === "void") return "(no input)";

  switch (input.kind) {
    case "scan":
      return `LiDAR scan — nearest obstacle ${input.nearestDistance.toFixed(2)} m`;
    case "string":
      return input.value;
    case "object":
      if (input.typeName === "Detection") {
        const label = input.fields.label?.kind === "string" ? input.fields.label.value : "object";
        const conf = input.fields.confidence?.kind === "number" ? input.fields.confidence.value : 0;
        return `Vision scene — ${label} (${conf.toFixed(2)} confidence)`;
      }
      if (input.typeName === "Detections") {
        const count = input.fields.count?.kind === "number" ? input.fields.count.value : 0;
        return `Detections — ${count} object(s) in view`;
      }
      return `${input.typeName} object`;
    case "completion":
      return input.text;
    case "goal":
      return `Goal — ${input.text}`;
    default:
      return `(${input.kind} value)`;
  }
}
