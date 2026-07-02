/**
 * Mission assurance declaration parsing helpers for the TypeScript parser.
 * @module
 */

import type {
  AnomalyDetectorDecl,
  AnomalyHandlerDecl,
  AssuranceCaseDecl,
  ExpectedBehavior,
  KnowledgeComponent,
  KnowledgeDependency,
  KnowledgeModelDecl,
  MitigationBranch,
  MitigationDecl,
  MissionConstraintDecl,
  MissionPlanDecl,
  MissionStepDecl,
  OperatingModeDecl,
  PrognosticRule,
  PrognosticsDecl,
  ResiliencePolicyDecl,
  RecoveryPolicyDecl,
  RecoveryPolicyBranch,
  ContinuityPolicyDecl,
  ContinuityPolicyBranch,
  DecisionTreeDecl,
  DecisionTreeBranch,
  DecisionTreeNestedBranch,
  OfflinePolicyDecl,
  StateEstimatorDecl,
} from "../ast/assurance-decls.js";
import type { Span } from "../ast/nodes.js";
import type { Token } from "../lexer/index.js";

export class ParseError extends Error {
  constructor(
    message: string,
    readonly line: number,
    readonly column: number,
  ) {
    super(message);
    this.name = "ParseError";
  }
}

export type AssuranceParseCtx = {
  advance(): Token;
  peek(): Token;
  previous(): Token;
  check(type: string): boolean;
  match(...types: string[]): boolean;
  expect(type: string, message: string): Token;
  parseLabel(message: string): string;
  parseDottedName(message: string): string;
  spanFrom(start: Token, end: Token): Span;
};

function parseComparisonOp(ctx: AssuranceParseCtx): string {
  // Description:
  //     ParseComparisonOp.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: string
  //         Return value from `parseComparisonOp`.
  //
  // Example:

  //     const result = parseComparisonOp(ctx);

  if (ctx.check("GTE")) {
    ctx.advance();
    return ">=";
  }
  if (ctx.check("GT")) {
    ctx.advance();
    return ">";
  }
  if (ctx.check("LTE")) {
    ctx.advance();
    return "<=";
  }
  if (ctx.check("LT")) {
    ctx.advance();
    return "<";
  }
  if (ctx.check("EQ")) {
    ctx.advance();
    return "==";
  }
  const t = ctx.peek();
  throw new ParseError("Expected comparison operator", t.line, t.column);
}

function parseThresholdValue(ctx: AssuranceParseCtx): string {
  // Description:
  //     ParseThresholdValue.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: string
  //         Return value from `parseThresholdValue`.
  //
  // Example:

  //     const result = parseThresholdValue(ctx);

  let threshold: string;
  if (ctx.check("TRUE")) {
    ctx.advance();
    threshold = "true";
  } else if (ctx.check("FALSE")) {
    ctx.advance();
    threshold = "false";
  } else if (ctx.check("NUMBER") || ctx.check("UNIT_LITERAL") || ctx.check("IDENT")) {
    threshold = ctx.advance().lexeme;
  } else {
    const t = ctx.peek();
    throw new ParseError("Expected threshold value", t.line, t.column);
  }
  if (ctx.check("IDENT")) {
    threshold += ` ${ctx.advance().lexeme}`;
    if (ctx.check("SLASH")) {
      threshold += ctx.advance().lexeme;
      if (ctx.check("IDENT")) threshold += ctx.advance().lexeme;
    }
  } else if (ctx.check("PERCENT")) {
    threshold += ctx.advance().lexeme;
  }
  return threshold;
}

function parseBracketNameList(ctx: AssuranceParseCtx): string[] {
  // Description:
  //     ParseBracketNameList.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: string[]
  //         Return value from `parseBracketNameList`.
  //
  // Example:

  //     const result = parseBracketNameList(ctx);

  ctx.expect("LBRACKET", "Expected '['");
  const items: string[] = [];
  while (!ctx.check("RBRACKET") && !ctx.check("EOF")) {
    items.push(ctx.parseDottedName("Expected name in list"));
    if (ctx.check("COMMA")) ctx.advance();
    else break;
  }
  ctx.expect("RBRACKET", "Expected ']'");
  return items;
}

function parseActionStatement(ctx: AssuranceParseCtx): string {
  // Description:
  //     ParseActionStatement.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: string
  //         Return value from `parseActionStatement`.
  //
  // Example:

  //     const result = parseActionStatement(ctx);

  const parts: string[] = [];
  while (!ctx.check("SEMICOLON") && !ctx.check("RBRACE") && !ctx.check("EOF")) {
    parts.push(ctx.advance().lexeme);
    if (ctx.check("DOT")) parts.push(ctx.advance().lexeme);
    if (ctx.check("LPAREN")) {
      parts.push(ctx.advance().lexeme);
      while (!ctx.check("RPAREN") && !ctx.check("EOF")) parts.push(ctx.advance().lexeme);
      if (ctx.check("RPAREN")) parts.push(ctx.advance().lexeme);
    }
  }
  if (ctx.check("SEMICOLON")) ctx.advance();
  return parts.join("");
}

export function parseKnowledgeModel(ctx: AssuranceParseCtx): KnowledgeModelDecl {
  // Description:
  //     ParseKnowledgeModel.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: KnowledgeModelDecl
  //         Return value from `parseKnowledgeModel`.
  //
  // Example:

  //     const result = parseKnowledgeModel(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected knowledge model name");
  ctx.expect("LBRACE", "Expected '{' after knowledge_model name");
  const components: KnowledgeComponent[] = [];
  const dependencies: KnowledgeDependency[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "component") {
      ctx.advance();
      const comp = ctx.parseLabel("Expected component name");
      ctx.expect("SEMICOLON", "Expected ';' after component");
      components.push({ name: comp, span: ctx.spanFrom(start, ctx.previous()) });
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "dependency") {
      ctx.advance();
      const capability = ctx.parseLabel("Expected dependency capability");
      ctx.expect("REQUIRES", "Expected 'requires' in dependency");
      const requires = parseBracketNameList(ctx);
      ctx.expect("SEMICOLON", "Expected ';' after dependency");
      dependencies.push({ capability, requires, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected component or dependency in knowledge_model", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close knowledge_model");
  return { kind: "KnowledgeModelDecl", name, components, dependencies, span: ctx.spanFrom(start, end) };
}

export function parseStateEstimator(ctx: AssuranceParseCtx): StateEstimatorDecl {
  // Description:
  //     ParseStateEstimator.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: StateEstimatorDecl
  //         Return value from `parseStateEstimator`.
  //
  // Example:

  //     const result = parseStateEstimator(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected state_estimator name");
  ctx.expect("LBRACE", "Expected '{' after state_estimator name");
  let inputs: string[] = [];
  let outputType = "";
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "inputs") {
      ctx.advance();
      inputs = parseBracketNameList(ctx);
      ctx.expect("SEMICOLON", "Expected ';' after inputs");
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "output") {
      ctx.advance();
      outputType = ctx.parseLabel("Expected output type");
      ctx.expect("SEMICOLON", "Expected ';' after output");
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected inputs or output in state_estimator", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close state_estimator");
  return { kind: "StateEstimatorDecl", name, inputs, outputType, span: ctx.spanFrom(start, end) };
}

export function parseAnomalyDetector(ctx: AssuranceParseCtx): AnomalyDetectorDecl {
  // Description:
  //     ParseAnomalyDetector.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: AnomalyDetectorDecl
  //         Return value from `parseAnomalyDetector`.
  //
  // Example:

  //     const result = parseAnomalyDetector(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected anomaly_detector name");
  ctx.expect("LBRACE", "Expected '{' after anomaly_detector name");
  const expected: ExpectedBehavior[] = [];
  let learnedBackend: string | null = null;
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "learned") {
      ctx.advance();
      if (ctx.check("IDENT") && ctx.peek().lexeme === "backend") {
        ctx.advance();
      }
      learnedBackend = ctx.parseDottedName("Expected learned backend path");
      ctx.expect("SEMICOLON", "Expected ';' after learned backend");
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "expected") {
      ctx.advance();
      const metric = ctx.parseDottedName("Expected metric path");
      const operator = parseComparisonOp(ctx);
      const threshold = parseThresholdValue(ctx);
      ctx.expect("SEMICOLON", "Expected ';' after expected rule");
      expected.push({ metric, operator, threshold, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'learned' or 'expected' rule in anomaly_detector", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close anomaly_detector");
  return { kind: "AnomalyDetectorDecl", name, learnedBackend, expected, span: ctx.spanFrom(start, end) };
}

export function parseAnomalyHandler(ctx: AssuranceParseCtx): AnomalyHandlerDecl {
  // Description:
  //     ParseAnomalyHandler.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: AnomalyHandlerDecl
  //         Return value from `parseAnomalyHandler`.
  //
  // Example:

  //     const result = parseAnomalyHandler(ctx);

  const start = ctx.advance();
  ctx.expect("IDENT", "Expected 'anomaly' after on");
  const detector = ctx.parseLabel("Expected anomaly detector name");
  ctx.expect("IDENT", "Expected 'severity'");
  const severity = ctx.parseLabel("Expected severity level");
  ctx.expect("LBRACE", "Expected '{' after anomaly handler");
  const actions: string[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    actions.push(parseActionStatement(ctx));
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close anomaly handler");
  return { kind: "AnomalyHandlerDecl", detector, severity, actions, span: ctx.spanFrom(start, end) };
}

export function parsePrognostics(ctx: AssuranceParseCtx): PrognosticsDecl {
  // Description:
  //     ParsePrognostics.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: PrognosticsDecl
  //         Return value from `parsePrognostics`.
  //
  // Example:

  //     const result = parsePrognostics(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected prognostics name");
  ctx.expect("LBRACE", "Expected '{' after prognostics name");
  const rules: PrognosticRule[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    const kind = ctx.parseLabel("Expected prognostic rule kind");
    const target =
      kind === "predict" || kind === "warn_if"
        ? ctx.parseDottedName("Expected prognostic target")
        : ctx.parseLabel("Expected prognostic target");
    let threshold: string | null = null;
    if (kind === "warn_if" && (ctx.check("LT") || ctx.check("LTE"))) {
      parseComparisonOp(ctx);
      threshold = parseThresholdValue(ctx);
    }
    ctx.expect("SEMICOLON", "Expected ';' after prognostic rule");
    rules.push({ kind, target, threshold, span: ctx.spanFrom(start, ctx.previous()) });
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close prognostics");
  return { kind: "PrognosticsDecl", name, rules, span: ctx.spanFrom(start, end) };
}

export function parseMitigation(ctx: AssuranceParseCtx): MitigationDecl {
  // Description:
  //     ParseMitigation.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: MitigationDecl
  //         Return value from `parseMitigation`.
  //
  // Example:

  //     const result = parseMitigation(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected mitigation name");
  ctx.expect("LBRACE", "Expected '{' after mitigation name");
  const branches: MitigationBranch[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IF")) {
      ctx.advance();
      const condition = ctx.parseDottedName("Expected mitigation condition");
      ctx.expect("LBRACE", "Expected '{' after if condition");
      const actions: string[] = [];
      while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
        actions.push(parseActionStatement(ctx));
      }
      ctx.expect("RBRACE", "Expected '}' to close if branch");
      branches.push({ condition, actions, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'if' branch in mitigation", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close mitigation");
  return { kind: "MitigationDecl", name, branches, span: ctx.spanFrom(start, end) };
}

export function parseRecoveryPolicy(ctx: AssuranceParseCtx): RecoveryPolicyDecl {
  // Description:
  //     ParseRecoveryPolicy.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: RecoveryPolicyDecl
  //         Return value from `parseRecoveryPolicy`.
  //
  // Example:

  //     const result = parseRecoveryPolicy(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected recovery_policy name");
  ctx.expect("LBRACE", "Expected '{' after recovery_policy name");
  const branches: RecoveryPolicyBranch[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("ON")) {
      ctx.advance();
      const condition = ctx.parseDottedName("Expected recovery policy condition");
      ctx.expect("LBRACE", "Expected '{' after on condition");
      const actions: string[] = [];
      while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
        actions.push(parseActionStatement(ctx));
      }
      ctx.expect("RBRACE", "Expected '}' to close on branch");
      branches.push({ condition, actions, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'on' branch in recovery_policy", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close recovery_policy");
  return { kind: "RecoveryPolicyDecl", name, branches, span: ctx.spanFrom(start, end) };
}

export function parseContinuityPolicy(ctx: AssuranceParseCtx): ContinuityPolicyDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected continuity_policy name");
  ctx.expect("LBRACE", "Expected '{' after continuity_policy name");
  const branches: ContinuityPolicyBranch[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("ON")) {
      ctx.advance();
      const condition = ctx.parseDottedName("Expected continuity policy condition");
      ctx.expect("LBRACE", "Expected '{' after on condition");
      const actions: string[] = [];
      while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
        actions.push(parseActionStatement(ctx));
      }
      ctx.expect("RBRACE", "Expected '}' to close on branch");
      branches.push({ condition, actions, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'on' branch in continuity_policy", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close continuity_policy");
  return { kind: "ContinuityPolicyDecl", name, branches, span: ctx.spanFrom(start, end) };
}

export function parseAssuranceCase(ctx: AssuranceParseCtx): AssuranceCaseDecl {
  // Description:
  //     ParseAssuranceCase.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: AssuranceCaseDecl
  //         Return value from `parseAssuranceCase`.
  //
  // Example:

  //     const result = parseAssuranceCase(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected assurance_case name");
  ctx.expect("LBRACE", "Expected '{' after assurance_case name");
  const evidence: string[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "evidence") {
      ctx.advance();
      evidence.push(ctx.parseDottedName("Expected evidence source"));
      ctx.expect("SEMICOLON", "Expected ';' after evidence");
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected evidence in assurance_case", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close assurance_case");
  return { kind: "AssuranceCaseDecl", name, evidence, span: ctx.spanFrom(start, end) };
}

export function parseResiliencePolicy(ctx: AssuranceParseCtx): ResiliencePolicyDecl {
  // Description:
  //     ParseResiliencePolicy.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: ResiliencePolicyDecl
  //         Return value from `parseResiliencePolicy`.
  //
  // Example:

  //     const result = parseResiliencePolicy(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected resilience_policy name");
  ctx.expect("LBRACE", "Expected '{' after resilience_policy name");
  const strategies: string[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "strategy") {
      ctx.advance();
      strategies.push(ctx.parseLabel("Expected strategy name"));
      ctx.expect("SEMICOLON", "Expected ';' after strategy");
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected strategy in resilience_policy", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close resilience_policy");
  return { kind: "ResiliencePolicyDecl", name, strategies, span: ctx.spanFrom(start, end) };
}

export function parseMissionPlan(ctx: AssuranceParseCtx): MissionPlanDecl {
  // Description:
  //     ParseMissionPlan.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: MissionPlanDecl
  //         Return value from `parseMissionPlan`.
  //
  // Example:

  //     const result = parseMissionPlan(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected mission_plan name");
  ctx.expect("LBRACE", "Expected '{' after mission_plan name");
  const steps: MissionStepDecl[] = [];
  const constraints: MissionConstraintDecl[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "step") {
      ctx.advance();
      const stepName = ctx.parseLabel("Expected step name");
      ctx.expect("SEMICOLON", "Expected ';' after step");
      steps.push({ name: stepName, span: ctx.spanFrom(start, ctx.previous()) });
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "constraint") {
      ctx.advance();
      const parts: string[] = [];
      while (!ctx.check("SEMICOLON") && !ctx.check("EOF")) parts.push(ctx.advance().lexeme);
      ctx.expect("SEMICOLON", "Expected ';' after constraint");
      constraints.push({ constraint: parts.join(" "), span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected step or constraint in mission_plan", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close mission_plan");
  return { kind: "MissionPlanDecl", name, steps, constraints, span: ctx.spanFrom(start, end) };
}

export function parseOperatingMode(ctx: AssuranceParseCtx): OperatingModeDecl {
  // Description:
  //     ParseOperatingMode.
  //
  // Inputs:
  //     ctx: AssuranceParseCtx
  //         Caller-supplied ctx.
  //
  // Outputs:
  //     result: OperatingModeDecl
  //         Return value from `parseOperatingMode`.
  //
  // Example:

  //     const result = parseOperatingMode(ctx);

  const start = ctx.advance();
  const name = ctx.parseLabel("Expected operating_mode name");
  ctx.expect("LBRACE", "Expected '{' after operating_mode name");
  let modeKind = "normal";
  if (ctx.check("IDENT")) {
    modeKind = ctx.advance().lexeme;
    ctx.expect("SEMICOLON", "Expected ';' after mode kind");
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close operating_mode");
  return { kind: "OperatingModeDecl", name, modeKind, span: ctx.spanFrom(start, end) };
}

function parseDecisionTreeCondition(ctx: AssuranceParseCtx): string {
  const parts: string[] = [];
  while (!ctx.check("LBRACE") && !ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") || ctx.check("NUMBER")) {
      parts.push(ctx.advance().lexeme);
    } else if (ctx.check("DOT")) {
      ctx.advance();
      parts.push(".");
    } else if (ctx.check("EQ")) {
      ctx.advance();
      parts.push("==");
    } else if (ctx.check("ASSIGN")) {
      ctx.advance();
      parts.push("=");
    } else if (ctx.check("NEQ")) {
      ctx.advance();
      parts.push("!=");
    } else {
      break;
    }
  }
  if (parts.length === 0) {
    const t = ctx.peek();
    throw new ParseError("Expected decision tree condition", t.line, t.column);
  }
  return parts.join(" ").replace(/ \. /g, ".");
}

export function parseDecisionTree(ctx: AssuranceParseCtx): DecisionTreeDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected decision_tree name");
  const scope = ctx.parseLabel("Expected decision_tree scope (local, reflex, fleet)");
  const layer = scope;
  ctx.expect("LBRACE", "Expected '{' after decision_tree scope");
  let version: string | null = null;
  const branches: DecisionTreeBranch[] = [];
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "version") {
      ctx.advance();
      ctx.expect("EQ", "Expected '=' after version");
      version = ctx.parseLabel("Expected version string");
    } else if (ctx.check("WHEN")) {
      ctx.advance();
      const condition = parseDecisionTreeCondition(ctx);
      ctx.expect("LBRACE", "Expected '{' after when condition");
      const actions: string[] = [];
      const nested: DecisionTreeNestedBranch[] = [];
      while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
        if (ctx.check("IF")) {
          ctx.advance();
          const nestedCond = parseDecisionTreeCondition(ctx);
          ctx.expect("LBRACE", "Expected '{' after if condition");
          const nestedActions: string[] = [];
          while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
            nestedActions.push(parseActionStatement(ctx));
          }
          ctx.expect("RBRACE", "Expected '}' to close if branch");
          nested.push({
            condition: nestedCond,
            actions: nestedActions,
            span: ctx.spanFrom(start, ctx.previous()),
          });
        } else if (ctx.check("ELSE")) {
          ctx.advance();
          if (ctx.check("IF")) {
            ctx.advance();
            const nestedCond = parseDecisionTreeCondition(ctx);
            ctx.expect("LBRACE", "Expected '{' after else if");
            const nestedActions: string[] = [];
            while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
              nestedActions.push(parseActionStatement(ctx));
            }
            ctx.expect("RBRACE", "Expected '}' to close else if");
            nested.push({
              condition: nestedCond,
              actions: nestedActions,
              span: ctx.spanFrom(start, ctx.previous()),
            });
          } else {
            ctx.expect("LBRACE", "Expected '{' after else");
            const nestedActions: string[] = [];
            while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
              nestedActions.push(parseActionStatement(ctx));
            }
            ctx.expect("RBRACE", "Expected '}' to close else");
            nested.push({
              condition: "else",
              actions: nestedActions,
              span: ctx.spanFrom(start, ctx.previous()),
            });
          }
        } else {
          actions.push(parseActionStatement(ctx));
        }
      }
      ctx.expect("RBRACE", "Expected '}' to close when branch");
      branches.push({ condition, actions, nested, span: ctx.spanFrom(start, ctx.previous()) });
    } else {
      const t = ctx.peek();
      throw new ParseError("Expected 'when' branch or 'version' in decision_tree", t.line, t.column);
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close decision_tree");
  return {
    kind: "DecisionTreeDecl",
    name,
    scope,
    layer,
    version,
    branches,
    span: ctx.spanFrom(start, end),
  };
}

export function parseOfflinePolicy(ctx: AssuranceParseCtx): OfflinePolicyDecl {
  const start = ctx.advance();
  const name = ctx.parseLabel("Expected offline_policy name");
  ctx.expect("LBRACE", "Expected '{' after offline_policy name");
  let maxDurationMinutes = 30;
  let allowedActions: string[] = [];
  let forbiddenActions: string[] = [];
  let policyVersion: string | null = null;
  let signature: string | null = null;
  let expiresAtMs: number | null = null;
  while (!ctx.check("RBRACE") && !ctx.check("EOF")) {
    if (ctx.check("IDENT") && ctx.peek().lexeme === "max_duration") {
      ctx.advance();
      ctx.expect("ASSIGN", "Expected '=' after max_duration");
      const numTok = ctx.advance();
      const digits = numTok.lexeme.replace(/\D/g, "");
      maxDurationMinutes = Number.parseInt(digits, 10) || 30;
      if (ctx.check("IDENT") && ctx.peek().lexeme === "min") {
        ctx.advance();
      }
    } else if (ctx.check("SEMICOLON")) {
      ctx.advance();
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "allowed_actions") {
      ctx.advance();
      allowedActions = parseBracketNameList(ctx);
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "forbidden_actions") {
      ctx.advance();
      forbiddenActions = parseBracketNameList(ctx);
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "policy_version") {
      ctx.advance();
      ctx.expect("ASSIGN", "Expected '=' after policy_version");
      policyVersion = ctx.parseLabel("Expected policy_version string");
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "signature") {
      ctx.advance();
      ctx.expect("ASSIGN", "Expected '=' after signature");
      signature = ctx.parseLabel("Expected signature hex");
    } else if (ctx.check("IDENT") && ctx.peek().lexeme === "expires_at") {
      ctx.advance();
      ctx.expect("ASSIGN", "Expected '=' after expires_at");
      const numTok = ctx.advance();
      expiresAtMs = Number.parseFloat(numTok.lexeme) || null;
    } else {
      const t = ctx.peek();
      throw new ParseError(
        "Expected max_duration, allowed_actions, forbidden_actions, policy_version, signature, or expires_at in offline_policy",
        t.line,
        t.column,
      );
    }
  }
  const end = ctx.expect("RBRACE", "Expected '}' to close offline_policy");
  return {
    kind: "OfflinePolicyDecl",
    name,
    maxDurationMinutes,
    allowedActions,
    forbiddenActions,
    policyVersion,
    signature,
    expiresAtMs,
    span: ctx.spanFrom(start, end),
  };
}
