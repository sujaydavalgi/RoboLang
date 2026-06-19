export type { AIProvider, CompletionRequest, DetectionRequest, EmbedRequest } from "./AIProvider.js";
export { AIModel, createAIModel, type AiModelConfig } from "./AIModel.js";
export type { AgentRuntime, PlanExecutor } from "./Agent.js";
export {
  agentToolNames,
  agentUsesModels,
  createAgentRuntime,
  executeAgentPlan,
} from "./Agent.js";
export { MemoryStore, type MemoryKind } from "./MemoryStore.js";
export { MockAIProvider, mockAnalyzeFrame, mockCameraFrame, mockSummarize } from "./MockAIProvider.js";
export { buildPrompt } from "./PromptRuntime.js";
export { resolveAiImport, listAiLibraries, AI_LIB_REGISTRY } from "./registry.js";

import type { RuntimeValue } from "../runtime/interpreter.js";

export function runtimeSafeAction(linear: number, angular: number): RuntimeValue {
  return { kind: "safe_action", linear, angular, trusted: true };
}

export function runtimeActionProposal(
  linear: number,
  angular: number,
  source: string,
): RuntimeValue {
  return { kind: "action_proposal", linear, angular, source, trace: [], trusted: false };
}

export function isActionProposal(value: RuntimeValue): boolean {
  return value.kind === "action_proposal";
}

export function isSafeAction(
  value: RuntimeValue,
): value is Extract<RuntimeValue, { kind: "safe_action" }> {
  return value.kind === "safe_action";
}

export function proposalFromValue(value: RuntimeValue): { linear: number; angular: number; source: string } | null {
  if (value.kind === "action_proposal") {
    return { linear: value.linear, angular: value.angular, source: value.source };
  }
  if (value.kind === "object" && value.typeName === "ActionProposal") {
    const linear = value.fields.linear?.kind === "number" ? value.fields.linear.value : 0;
    const angular = value.fields.angular?.kind === "number" ? value.fields.angular.value : 0;
    return { linear, angular, source: "object" };
  }
  if (value.kind === "velocity") {
    return { linear: value.linear, angular: value.angular, source: "velocity" };
  }
  return null;
}

export function safeActionFromProposal(
  linear: number,
  angular: number,
): RuntimeValue {
  return runtimeSafeAction(linear, angular);
}

export function wrapCompletion(text: string, model: string): RuntimeValue {
  return { kind: "completion", text, model };
}

export function wrapDetection(
  label: string,
  confidence: number,
  nearestDistance: number,
): RuntimeValue {
  return {
    kind: "object",
    typeName: "Detection",
    fields: {
      label: { kind: "string", value: label },
      confidence: { kind: "number", value: confidence, unit: "none" },
      nearest_distance: { kind: "number", value: nearestDistance, unit: "m" },
    },
  };
}

export type { RuntimeValue };
