import type {
  BehaviorDecl,
  Expr,
  Program,
  RobotDecl,
  SafetyRule,
  SafetyZoneDecl,
  Stmt,
  UnitKind,
  SynapseType,
} from "../ast/nodes.js";
import { resolveImport } from "../lib/registry.js";
import { resolveAiImport } from "../ai/registry.js";
import { getSocProfile, validateHalAgainstSoc } from "../soc/index.js";
import { halMemberFromDecl } from "../hal/index.js";
import {
  ACTION_TYPES,
  AI_INPUT_TYPES,
  AI_OUTPUT_TYPES,
  ACTUATOR_TYPES,
  BUILTIN_FUNCTIONS,
  BUILTIN_METHODS,
  MESSAGE_TYPES,
  OBJECT_PROPERTIES,
  POSE_PROPERTIES,
  ROBOT_METHODS,
  SCAN_PROPERTIES,
  SENSOR_TYPES,
  SERVICE_TYPES,
  VELOCITY_PROPERTIES,
  TypeCheckError,
  resultUnitForBinary,
  unitsCompatible,
  type TypeError,
} from "./units.js";

type SymbolEntry = {
  name: string;
  roboType: SynapseType;
  kind: "sensor" | "actuator" | "variable" | "behavior" | "topic" | "service" | "action" | "robot" | "ai_model";
  sensorType?: string;
  actuatorType?: string;
  messageType?: string;
  serviceType?: string;
  actionType?: string;
};

export function typeCheck(program: Program): void {
  const checker = new TypeChecker();
  checker.checkProgram(program);
  if (checker.errors.length > 0) {
    throw new TypeCheckError(checker.errors);
  }
}

class TypeChecker {
  errors: TypeError[] = [];
  private symbols = new Map<string, SymbolEntry>();
  private currentRobot: RobotDecl | null = null;

  checkProgram(program: Program): void {
    const imported = new Set<string>();
    for (const imp of program.imports) {
      if (!resolveImport(imp.path) && !resolveAiImport(imp.path)) {
        this.error(`Unknown library '${imp.path}'`, imp.span.start.line, imp.span.start.column);
      } else {
        imported.add(imp.path);
      }
    }

    for (const robot of program.robots) {
      this.checkRobot(robot, imported);
    }
  }

  private checkRobot(robot: RobotDecl, imported: Set<string>): void {
    this.currentRobot = robot;
    this.symbols.clear();

    if (robot.soc) {
      if (!getSocProfile(robot.soc.profile)) {
        this.error(`Unknown SoC profile '${robot.soc.profile}'`, robot.soc.span.start.line, robot.soc.span.start.column);
      }
    }

    if (robot.hal && robot.soc) {
      const profile = getSocProfile(robot.soc.profile);
      if (profile) {
        const members = robot.hal.members.map(halMemberFromDecl);
        for (const err of validateHalAgainstSoc(profile, members)) {
          this.error(err.message, robot.hal.span.start.line, robot.hal.span.start.column);
        }
      }
    }

    const halBusNames = new Set(robot.hal?.members.map((m) => m.name) ?? []);

    for (const node of robot.nodes) {
      if (!node.namespace) {
        this.error("Node should specify namespace with 'on \"/namespace\"'", node.span.start.line, node.span.start.column);
      }
    }

    for (const topic of robot.topics) {
      if (!MESSAGE_TYPES[topic.messageType]) {
        this.error(`Unknown message type '${topic.messageType}'`, topic.span.start.line, topic.span.start.column);
      }
      this.symbols.set(topic.name, {
        name: topic.name,
        roboType: MESSAGE_TYPES[topic.messageType] ?? { kind: "void" },
        kind: "topic",
        messageType: topic.messageType,
      });
    }

    for (const service of robot.services) {
      if (!SERVICE_TYPES[service.serviceType]) {
        this.error(`Unknown service type '${service.serviceType}'`, service.span.start.line, service.span.start.column);
      }
      this.symbols.set(service.name, {
        name: service.name,
        roboType: SERVICE_TYPES[service.serviceType] ?? { kind: "void" },
        kind: "service",
        serviceType: service.serviceType,
      });
    }

    for (const action of robot.actions) {
      if (!ACTION_TYPES[action.actionType]) {
        this.error(`Unknown action type '${action.actionType}'`, action.span.start.line, action.span.start.column);
      }
      this.symbols.set(action.name, {
        name: action.name,
        roboType: ACTION_TYPES[action.actionType] ?? { kind: "void" },
        kind: "action",
        actionType: action.actionType,
      });
    }

    for (const sensor of robot.sensors) {
      if (!SENSOR_TYPES[sensor.sensorType]) {
        this.error(`Unknown sensor type '${sensor.sensorType}'`, sensor.span.start.line, sensor.span.start.column);
      }
      if (sensor.library) {
        if (!imported.has(sensor.library)) {
          this.error(`Library '${sensor.library}' must be imported before use`, sensor.span.start.line, sensor.span.start.column);
        }
        const lib = resolveImport(sensor.library);
        if (lib && !lib.sensors[sensor.sensorType]) {
          this.error(`Sensor type '${sensor.sensorType}' not provided by library '${sensor.library}'`, sensor.span.start.line, sensor.span.start.column);
        }
      }
      if (sensor.binding?.kind === "hal" && !halBusNames.has(sensor.binding.busName)) {
        this.error(`Unknown HAL bus '${sensor.binding.busName}'`, sensor.span.start.line, sensor.span.start.column);
      }
      this.symbols.set(sensor.name, {
        name: sensor.name,
        roboType: SENSOR_TYPES[sensor.sensorType] ?? { kind: "named", name: sensor.sensorType },
        kind: "sensor",
        sensorType: sensor.sensorType,
      });
    }

    for (const actuator of robot.actuators) {
      if (!ACTUATOR_TYPES[actuator.actuatorType]) {
        this.error(`Unknown actuator type '${actuator.actuatorType}'`, actuator.span.start.line, actuator.span.start.column);
      }
      this.symbols.set(actuator.name, {
        name: actuator.name,
        roboType: ACTUATOR_TYPES[actuator.actuatorType] ?? { kind: "named", name: actuator.actuatorType },
        kind: "actuator",
        actuatorType: actuator.actuatorType,
      });
    }

    if (robot.safety) {
      const saved = new Map(this.symbols);
      for (const rule of robot.safety.rules) {
        this.checkSafetyRule(rule);
      }
      for (const zone of robot.safety.zones) {
        this.checkSafetyZone(zone);
      }
      this.symbols = saved;
    }

    if (robot.ai) {
      for (const model of robot.ai.models) {
        this.checkAiModel(model, imported);
      }
    }

    for (const behavior of robot.behaviors) {
      this.symbols.set(behavior.name, {
        name: behavior.name,
        roboType: { kind: "void" },
        kind: "behavior",
      });
      this.checkBehavior(behavior);
    }
  }

  private checkSafetyRule(rule: SafetyRule): void {
    if (rule.kind === "MaxSpeedRule") {
      const t = this.checkExpr(rule.value);
      if (t.kind !== "number" || !unitsCompatible(t.unit, rule.unit)) {
        this.error(
          `Expected value with unit '${rule.unit}' for ${rule.name}`,
          rule.span.start.line,
          rule.span.start.column,
        );
      }
    } else {
      const t = this.checkExpr(rule.condition);
      if (t.kind !== "bool") {
        this.error("stop_if condition must be boolean", rule.span.start.line, rule.span.start.column);
      }
    }
  }

  private checkSafetyZone(zone: SafetyZoneDecl): void {
    const x = this.checkExpr(zone.x);
    const y = this.checkExpr(zone.y);
    if (x.kind !== "number" || y.kind !== "number") {
      this.error("Zone coordinates must be numeric", zone.span.start.line, zone.span.start.column);
    }
    if (zone.shape === "circle" && zone.radius) {
      const r = this.checkExpr(zone.radius);
      if (r.kind !== "number") {
        this.error("Zone radius must be numeric", zone.span.start.line, zone.span.start.column);
      }
    }
    if (zone.shape === "rect" && zone.width && zone.height) {
      const w = this.checkExpr(zone.width);
      const h = this.checkExpr(zone.height);
      if (w.kind !== "number" || h.kind !== "number") {
        this.error("Zone size must be numeric", zone.span.start.line, zone.span.start.column);
      }
    }
  }

  private checkAiModel(
    model: import("../ast/nodes.js").AiModelDecl,
    imported: Set<string>,
  ): void {
    if (!AI_OUTPUT_TYPES[model.outputType]) {
      this.error(
        `Unknown AI output type '${model.outputType}'`,
        model.span.start.line,
        model.span.start.column,
      );
    }
    if (model.library) {
      if (!imported.has(model.library)) {
        this.error(
          `AI library '${model.library}' must be imported before use`,
          model.span.start.line,
          model.span.start.column,
        );
      } else if (!resolveAiImport(model.library)) {
        this.error(
          `Unknown AI library '${model.library}'`,
          model.span.start.line,
          model.span.start.column,
        );
      }
    }
    for (const inputType of model.inputs) {
      if (!AI_INPUT_TYPES[inputType] && !SENSOR_TYPES[inputType]) {
        this.error(
          `Unknown AI input type '${inputType}'`,
          model.span.start.line,
          model.span.start.column,
        );
      }
    }
    if (this.symbols.has(model.name)) {
      this.error(
        `Duplicate ai model name '${model.name}'`,
        model.span.start.line,
        model.span.start.column,
      );
    }
    this.symbols.set(model.name, {
      name: model.name,
      roboType: AI_OUTPUT_TYPES[model.outputType] ?? { kind: "void" },
      kind: "ai_model",
    });
  }

  private checkBehavior(behavior: BehaviorDecl): void {
    const parentScope = new Map(this.symbols);
    this.symbols = new Map(parentScope);
    this.symbols.set("robot", {
      name: "robot",
      roboType: { kind: "named", name: "Robot" },
      kind: "robot",
    });
    for (const stmt of behavior.body) {
      this.checkStmt(stmt);
    }
    this.symbols = parentScope;
  }

  private checkStmt(stmt: Stmt): void {
    switch (stmt.kind) {
      case "VarDecl": {
        const t = this.checkExpr(stmt.init);
        this.symbols.set(stmt.name, {
          name: stmt.name,
          roboType: t,
          kind: "variable",
        });
        break;
      }
      case "IfStmt": {
        const cond = this.checkExpr(stmt.condition);
        if (cond.kind !== "bool") {
          this.error("if condition must be boolean", stmt.span.start.line, stmt.span.start.column);
        }
        for (const s of stmt.thenBranch) this.checkStmt(s);
        if (stmt.elseBranch) for (const s of stmt.elseBranch) this.checkStmt(s);
        break;
      }
      case "LoopStmt": {
        for (const s of stmt.body) this.checkStmt(s);
        break;
      }
      case "PublishStmt": {
        const topic = this.symbols.get(stmt.topicName);
        if (!topic || topic.kind !== "topic") {
          this.error(`Unknown topic '${stmt.topicName}'`, stmt.span.start.line, stmt.span.start.column);
        } else {
          const val = this.checkExpr(stmt.value);
          this.assertCompatible(topic.roboType, val, stmt.span.start.line, stmt.span.start.column);
        }
        break;
      }
      case "ServiceCallStmt": {
        const service = this.symbols.get(stmt.serviceName);
        if (!service || service.kind !== "service") {
          this.error(`Unknown service '${stmt.serviceName}'`, stmt.span.start.line, stmt.span.start.column);
        }
        break;
      }
      case "ActionSendStmt": {
        const action = this.symbols.get(stmt.actionName);
        if (!action || action.kind !== "action") {
          this.error(`Unknown action '${stmt.actionName}'`, stmt.span.start.line, stmt.span.start.column);
        } else {
          const goal = this.checkExpr(stmt.goal);
          if (goal.kind !== "pose" && goal.kind !== "trajectory") {
            this.error("Action goal must be pose or trajectory", stmt.span.start.line, stmt.span.start.column);
          }
        }
        break;
      }
      case "EmergencyStopStmt":
      case "ResetEmergencyStopStmt":
        break;
      case "ExprStmt":
        this.checkExpr(stmt.expr);
        break;
      case "ReturnStmt":
        if (stmt.value) this.checkExpr(stmt.value);
        break;
    }
  }

  private checkExpr(expr: Expr): SynapseType {
    switch (expr.kind) {
      case "LiteralExpr":
        if (typeof expr.value === "boolean") return { kind: "bool" };
        if (typeof expr.value === "number") return { kind: "number", unit: "none" };
        if (typeof expr.value === "string") return { kind: "string" };
        return { kind: "void" };

      case "UnitLiteralExpr":
        return { kind: "number", unit: expr.unit };

      case "IdentExpr": {
        const sym = this.symbols.get(expr.name);
        if (!sym) {
          this.error(`Undefined identifier '${expr.name}'`, expr.span.start.line, expr.span.start.column);
          return { kind: "void" };
        }
        return sym.roboType;
      }

      case "BinaryExpr": {
        const left = this.checkExpr(expr.left);
        const right = this.checkExpr(expr.right);
        const result = resultUnitForBinary(expr.op, left, right);
        if (!result) {
          this.error(
            `Invalid operation '${expr.op}' for types`,
            expr.span.start.line,
            expr.span.start.column,
          );
          return { kind: "void" };
        }
        return result;
      }

      case "UnaryExpr": {
        const operand = this.checkExpr(expr.operand);
        if (expr.op === "not" && operand.kind !== "bool") {
          this.error("Operand of 'not' must be boolean", expr.span.start.line, expr.span.start.column);
        }
        if (expr.op === "-" && operand.kind !== "number") {
          this.error("Operand of '-' must be numeric", expr.span.start.line, expr.span.start.column);
        }
        return expr.op === "not" ? { kind: "bool" } : operand;
      }

      case "MemberExpr":
        return this.checkMember(expr);

      case "CallExpr":
        return this.checkCall(expr);

      case "InferExpr":
        return this.checkInfer(expr);

      default:
        return { kind: "void" };
    }
  }

  private checkMember(expr: import("../ast/nodes.js").MemberExpr): SynapseType {
    const objType = this.checkExpr(expr.object);

    if (objType.kind === "scan") {
      const prop = SCAN_PROPERTIES[expr.property];
      if (!prop) {
        this.error(`Unknown scan property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    if (objType.kind === "pose") {
      const prop = POSE_PROPERTIES[expr.property];
      if (!prop) {
        this.error(`Unknown pose property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    if (objType.kind === "velocity") {
      const prop = VELOCITY_PROPERTIES[expr.property];
      if (!prop) {
        this.error(`Unknown velocity property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    if (objType.kind === "named") {
      const objProps = OBJECT_PROPERTIES[objType.name];
      if (objProps?.[expr.property]) return objProps[expr.property];

      const methods = BUILTIN_METHODS[objType.name];
      if (methods?.[expr.property]) return methods[expr.property].returns;
    }

    this.error(`Unknown member '${expr.property}'`, expr.span.start.line, expr.span.start.column);
    return { kind: "void" };
  }

  private checkCall(expr: import("../ast/nodes.js").CallExpr): SynapseType {
    if (expr.callee.kind === "IdentExpr") {
      const fn = BUILTIN_FUNCTIONS[expr.callee.name];
      if (!fn) {
        this.error(`Unknown function '${expr.callee.name}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      for (const arg of expr.namedArgs) {
        const expected = fn.namedParams[arg.name];
        if (!expected) {
          this.error(`Unknown named argument '${arg.name}'`, arg.span.start.line, arg.span.start.column);
          continue;
        }
        const actual = this.checkExpr(arg.value);
        this.assertCompatible(expected, actual, arg.span.start.line, arg.span.start.column);
      }
      return fn.returns;
    }

    if (expr.callee.kind !== "MemberExpr" || expr.callee.object.kind !== "IdentExpr") {
      this.error("Invalid call target", expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }

    const member = expr.callee;
    if (member.object.kind !== "IdentExpr") {
      this.error("Invalid call target", expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }
    const targetName = member.object.name;
    const sym = this.symbols.get(targetName);
    if (!sym) {
      this.error(`Undefined identifier '${targetName}'`, expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }

    if (sym.kind === "robot") {
      const method = ROBOT_METHODS[member.property];
      if (!method) {
        this.error(`Unknown robot method '${member.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      for (let i = 0; i < expr.args.length; i++) {
        const expected = method.params[i];
        if (expected) {
          const actual = this.checkExpr(expr.args[i]);
          this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
        }
      }
      return method.returns;
    }

    let typeName = "";
    if (sym.kind === "sensor" && sym.sensorType) typeName = sym.sensorType;
    else if (sym.kind === "actuator" && sym.actuatorType) typeName = sym.actuatorType;
    else if (sym.roboType.kind === "named") typeName = sym.roboType.name;
    else if (sym.roboType.kind === "scan") typeName = "Scan";

    const methods = BUILTIN_METHODS[typeName];
    const method = methods?.[member.property];
    if (!method) {
      this.error(
        `Unknown method '${member.property}' on ${typeName}`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    if (method.namedParams) {
      for (const arg of expr.namedArgs) {
        const expected = method.namedParams[arg.name];
        if (!expected) {
          this.error(`Unknown named argument '${arg.name}'`, arg.span.start.line, arg.span.start.column);
          continue;
        }
        const actual = this.checkExpr(arg.value);
        this.assertCompatible(expected, actual, arg.span.start.line, arg.span.start.column);
      }
    }

    for (const arg of expr.args) {
      this.checkExpr(arg);
    }

    if (member.property === "read" && typeName === "Lidar") {
      return { kind: "scan" };
    }

    return method.returns;
  }

  private checkInfer(expr: import("../ast/nodes.js").InferExpr): SynapseType {
    const model = this.symbols.get(expr.modelName);
    if (!model || model.kind !== "ai_model") {
      this.error(
        `Unknown AI model '${expr.modelName}'`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    const aiModel = this.currentRobot?.ai?.models.find((m) => m.name === expr.modelName);
    if (aiModel && aiModel.inputs.length > 0) {
      for (const arg of expr.namedArgs) {
        const matchesInput = aiModel.inputs.some((inputType) => {
          const expected = AI_INPUT_TYPES[inputType] ?? SENSOR_TYPES[inputType];
          if (!expected) return inputType.toLowerCase() === arg.name.toLowerCase();
          const actual = this.checkExpr(arg.value);
          return this.typesCompatible(expected, actual);
        });
        if (!matchesInput) {
          this.error(
            `Input '${arg.name}' does not match declared model inputs [${aiModel.inputs.join(", ")}]`,
            arg.span.start.line,
            arg.span.start.column,
          );
        } else {
          this.checkExpr(arg.value);
        }
      }
    } else {
      for (const arg of expr.namedArgs) {
        this.checkExpr(arg.value);
      }
    }

    return model.roboType;
  }

  private typesCompatible(expected: SynapseType, actual: SynapseType): boolean {
    if (expected.kind === actual.kind) {
      if (expected.kind === "number" && actual.kind === "number") {
        return unitsCompatible(expected.unit, actual.unit);
      }
      if (expected.kind === "named" && actual.kind === "named") {
        return expected.name === actual.name || actual.name.includes(expected.name);
      }
      return true;
    }
    if (expected.kind === "named" && actual.kind === "scan" && expected.name.includes("Lidar")) {
      return true;
    }
    return false;
  }

  private assertCompatible(expected: SynapseType, actual: SynapseType, line: number, column: number): void {
    if (expected.kind === "void" && actual.kind === "void") return;
    if (expected.kind === actual.kind) {
      if (expected.kind === "number" && actual.kind === "number") {
        if (!unitsCompatible(expected.unit, actual.unit)) {
          this.error(
            `Unit mismatch: expected '${expected.unit}', got '${actual.unit}'`,
            line,
            column,
          );
        }
      }
      return;
    }
    this.error(`Type mismatch: expected ${expected.kind}, got ${actual.kind}`, line, column);
  }

  private error(message: string, line: number, column: number): void {
    this.errors.push({ message, line, column });
  }
}

export { typeCheck as check };
