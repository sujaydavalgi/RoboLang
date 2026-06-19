import type { RuntimeValue } from "../runtime/interpreter.js";

export type MemoryKind = "short_term" | "long_term";

export class MemoryStore {
  private entries: Array<{ key: string; value: RuntimeValue; at: number }> = [];
  private readonly limit: number;

  constructor(
    public readonly kind: MemoryKind,
    limit = kind === "short_term" ? 32 : 256,
  ) {
    this.limit = limit;
  }

  remember(key: string, value: RuntimeValue): void {
    this.entries.push({ key, value, at: Date.now() });
    if (this.entries.length > this.limit) {
      this.entries.shift();
    }
  }

  recall(key: string): RuntimeValue | undefined {
    for (let i = this.entries.length - 1; i >= 0; i--) {
      if (this.entries[i]!.key === key) return this.entries[i]!.value;
    }
    return undefined;
  }

  recent(count = 5): RuntimeValue[] {
    return this.entries.slice(-count).map((e) => e.value);
  }

  clear(): void {
    this.entries = [];
  }

  summaryForPrompt(): string | undefined {
    if (this.entries.length === 0) return undefined;
    const keys = this.entries
      .slice(-5)
      .map((e) => e.key)
      .reverse();
    return `Agent memory (${this.kind}): ${keys.join(", ")}`;
  }
}
