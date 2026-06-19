import type { AiModelDecl } from "../ast/nodes.js";
import type { RuntimeValue } from "../runtime/interpreter.js";
import { MockAIProvider, mockSummarize } from "./MockAIProvider.js";
import type { AIProvider } from "./AIProvider.js";
import { buildPrompt } from "./PromptRuntime.js";

export type AiModelConfig = {
  provider: string;
  model: string;
  temperature: number;
  maxTokens: number;
};

export class AIModel {
  readonly name: string;
  readonly modelType: string;
  readonly config: AiModelConfig;
  private provider: AIProvider;

  constructor(decl: AiModelDecl, provider: AIProvider = new MockAIProvider()) {
    this.name = decl.name;
    this.modelType = decl.modelType;
    this.config = parseConfig(decl);
    this.provider = provider;
  }

  reason(prompt: string, input?: RuntimeValue, goal?: string): RuntimeValue {
    if (this.modelType !== "LLM") {
      throw new Error(`Model '${this.name}' is ${this.modelType}, not LLM`);
    }
    return this.provider.complete({
      prompt: buildPrompt(prompt, input, goal),
      input,
      model: this.config.model,
      provider: this.config.provider,
      temperature: this.config.temperature,
      maxTokens: this.config.maxTokens,
    }) as RuntimeValue;
  }

  summarize(input?: RuntimeValue): RuntimeValue {
    if (this.modelType !== "LLM") {
      throw new Error(`Model '${this.name}' is ${this.modelType}, not LLM`);
    }
    return mockSummarize(input, this.config.model);
  }

  detect(frame: RuntimeValue): RuntimeValue {
    if (this.modelType !== "VisionModel") {
      throw new Error(`Model '${this.name} is ${this.modelType}, not VisionModel`);
    }
    if (!this.provider.detect) {
      throw new Error(`Provider '${this.config.provider} has no detect()`);
    }
    return this.provider.detect({
      model: this.config.model,
      provider: this.config.provider,
      frame,
    }) as RuntimeValue;
  }

  toRuntimeValue(): RuntimeValue {
    return {
      kind: "ai_model",
      name: this.name,
      modelType: this.modelType,
      provider: this.config.provider,
    };
  }
}

function parseConfig(decl: AiModelDecl): AiModelConfig {
  const map = new Map(decl.config.map((e) => [e.key, e.value]));
  return {
    provider: String(map.get("provider") ?? "mock"),
    model: String(map.get("model") ?? decl.name),
    temperature: Number(map.get("temperature") ?? 0.2),
    maxTokens: Number(map.get("max_tokens") ?? 512),
  };
}

export function createAIModel(decl: AiModelDecl): AIModel {
  return new AIModel(decl);
}
