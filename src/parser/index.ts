import type { Token } from "../lexer/index.js";
import { unitFromLexeme } from "../lexer/index.js";
import type {
  ActuatorDecl,
  ActionDecl,
  BehaviorDecl,
  BinaryOp,
  Expr,
  HalBlock,
  HalMemberDecl,
  ImportDecl,
  NamedArg,
  NodeDecl,
  Program,
  RobotDecl,
  SafetyBlock,
  SafetyRule,
  SafetyZoneDecl,
  AiBlock,
  AiModelDecl,
  SensorBinding,
  SensorDecl,
  ServiceDecl,
  SocDecl,
  Span,
  Stmt,
  TopicDecl,
  UnitKind,
} from "../ast/nodes.js";

export class ParseError extends Error {
  constructor(
    message: string,
    public line: number,
    public column: number,
  ) {
    super(message);
    this.name = "ParseError";
  }
}

export function parse(tokens: Token[]): Program {
  const parser = new Parser(tokens);
  return parser.parseProgram();
}

class Parser {
  private pos = 0;

  constructor(private tokens: Token[]) {}

  private peek(): Token {
    return this.tokens[this.pos];
  }

  private previous(): Token {
    return this.tokens[this.pos - 1];
  }

  private advance(): Token {
    if (this.peek().type !== "EOF") this.pos++;
    return this.previous();
  }

  private check(type: Token["type"]): boolean {
    return this.peek().type === type;
  }

  private match(...types: Token["type"][]): boolean {
    for (const t of types) {
      if (this.check(t)) {
        this.advance();
        return true;
      }
    }
    return false;
  }

  private expect(type: Token["type"], message: string): Token {
    if (this.check(type)) return this.advance();
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
  }

  private spanFrom(start: Token, end: Token): Span {
    return {
      start: { line: start.line, column: start.column, offset: start.offset },
      end: { line: end.line, column: end.column, offset: end.offset },
    };
  }

  parseProgram(): Program {
    const start = this.peek();
    const imports: ImportDecl[] = [];
    const robots: RobotDecl[] = [];

    while (this.check("IMPORT")) {
      imports.push(this.parseImport());
    }

    while (!this.check("EOF")) {
      robots.push(this.parseRobot());
    }

    const end = this.previous();
    return {
      kind: "Program",
      imports,
      robots,
      span: this.spanFrom(start, end),
    };
  }

  private parseImport(): ImportDecl {
    const start = this.advance();
    const vendor = this.expect("IDENT", "Expected library vendor name");
    this.expect("DOT", "Expected '.' in import path");
    const module = this.expect("IDENT", "Expected library module name");
    this.expect("SEMICOLON", "Expected ';' after import");
    const end = this.previous();
    return {
      kind: "ImportDecl",
      path: `${vendor.lexeme}.${module.lexeme}`,
      span: this.spanFrom(start, end),
    };
  }

  private parseRobot(): RobotDecl {
    const start = this.expect("ROBOT", "Expected 'robot'");
    const nameTok = this.expect("IDENT", "Expected robot name");
    this.expect("LBRACE", "Expected '{' after robot name");

    let soc: SocDecl | null = null;
    let hal: HalBlock | null = null;
    const nodes: NodeDecl[] = [];
    const topics: TopicDecl[] = [];
    const services: ServiceDecl[] = [];
    const actions: ActionDecl[] = [];
    const sensors: SensorDecl[] = [];
    const actuators: ActuatorDecl[] = [];
    let safety: SafetyBlock | null = null;
    let ai: AiBlock | null = null;
    const behaviors: BehaviorDecl[] = [];

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("SOC")) {
        soc = this.parseSoc();
      } else if (this.check("HAL")) {
        hal = this.parseHal();
      } else if (this.check("NODE")) {
        nodes.push(this.parseNode());
      } else if (this.check("TOPIC")) {
        topics.push(this.parseTopic());
      } else if (this.check("SERVICE")) {
        services.push(this.parseService());
      } else if (this.check("ACTION")) {
        actions.push(this.parseAction());
      } else if (this.check("SENSOR")) {
        sensors.push(this.parseSensor());
      } else if (this.check("ACTUATOR")) {
        actuators.push(this.parseActuator());
      } else if (this.check("SAFETY")) {
        safety = this.parseSafety();
      } else if (this.check("AI")) {
        ai = this.parseAi();
      } else if (this.check("BEHAVIOR")) {
        behaviors.push(this.parseBehavior());
      } else {
        const t = this.peek();
        throw new ParseError("Expected robot member declaration", t.line, t.column);
      }
    }

    const end = this.expect("RBRACE", "Expected '}' to close robot block");
    return {
      kind: "RobotDecl",
      name: nameTok.lexeme,
      soc,
      hal,
      nodes,
      topics,
      services,
      actions,
      sensors,
      actuators,
      safety,
      ai,
      behaviors,
      span: this.spanFrom(start, end),
    };
  }

  private parseSoc(): SocDecl {
    const start = this.advance();
    const profile = this.expect("IDENT", "Expected SoC profile name");
    this.expect("SEMICOLON", "Expected ';' after soc declaration");
    const end = this.previous();
    return { kind: "SocDecl", profile: profile.lexeme, span: this.spanFrom(start, end) };
  }

  private parseHal(): HalBlock {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after hal");
    const members: HalMemberDecl[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      members.push(this.parseHalMember());
    }
    const end = this.expect("RBRACE", "Expected '}' to close hal block");
    return { kind: "HalBlock", members, span: this.spanFrom(start, end) };
  }

  private parseHalMember(): HalMemberDecl {
    const start = this.peek();

    if (this.match("I2C")) {
      const name = this.expect("IDENT", "Expected I2C bus name");
      this.expect("AT", "Expected 'at' after I2C bus name");
      const addrTok = this.expect("NUMBER", "Expected I2C address");
      this.expect("SEMICOLON", "Expected ';' after I2C declaration");
      return {
        kind: "HalI2cDecl",
        name: name.lexeme,
        address: addrTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.match("SPI")) {
      const name = this.expect("IDENT", "Expected SPI bus name");
      this.expect("AT", "Expected 'at' after SPI bus name");
      const busTok = this.expect("NUMBER", "Expected SPI bus number");
      let csPin: number | null = null;
      if (this.match("PIN")) {
        const cs = this.expect("NUMBER", "Expected CS pin number");
        csPin = cs.value as number;
      }
      this.expect("SEMICOLON", "Expected ';' after SPI declaration");
      return {
        kind: "HalSpiDecl",
        name: name.lexeme,
        bus: busTok.value as number,
        csPin,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.match("GPIO")) {
      const name = this.expect("IDENT", "Expected GPIO name");
      let direction: "in" | "out" = "out";
      if (this.match("OUT")) direction = "out";
      else if (this.match("IN")) direction = "in";
      this.expect("PIN", "Expected 'pin' keyword");
      const pinTok = this.expect("NUMBER", "Expected GPIO pin number");
      this.expect("SEMICOLON", "Expected ';' after GPIO declaration");
      return {
        kind: "HalGpioDecl",
        name: name.lexeme,
        direction,
        pin: pinTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.match("PWM")) {
      const name = this.expect("IDENT", "Expected PWM name");
      this.expect("ON", "Expected 'on' after PWM name");
      this.expect("PIN", "Expected 'pin' after on");
      const pinTok = this.expect("NUMBER", "Expected PWM pin");
      this.expect("FREQUENCY", "Expected 'frequency' after PWM pin");
      const freq = this.parseFrequencyHz();
      this.expect("SEMICOLON", "Expected ';' after PWM declaration");
      return {
        kind: "HalPwmDecl",
        name: name.lexeme,
        pin: pinTok.value as number,
        frequencyHz: freq,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.match("UART")) {
      const name = this.expect("IDENT", "Expected UART name");
      this.expect("ON", "Expected 'on' after UART name");
      const device = this.expect("STRING", "Expected UART device path");
      this.expect("BAUD", "Expected 'baud' after UART device");
      const baudTok = this.expect("NUMBER", "Expected baud rate");
      this.expect("SEMICOLON", "Expected ';' after UART declaration");
      return {
        kind: "HalUartDecl",
        name: name.lexeme,
        device: device.value as string,
        baud: baudTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.match("ADC")) {
      const name = this.expect("IDENT", "Expected ADC name");
      this.expect("ON", "Expected 'on' after ADC name");
      this.expect("IDENT", "Expected 'channel' keyword"); // channel as ident
      const chTok = this.expect("NUMBER", "Expected ADC channel number");
      this.expect("SEMICOLON", "Expected ';' after ADC declaration");
      return {
        kind: "HalAdcDecl",
        name: name.lexeme,
        channel: chTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    const t = this.peek();
    throw new ParseError("Expected HAL member (i2c, spi, gpio, pwm, uart, adc)", t.line, t.column);
  }

  private parseFrequencyHz(): number {
    const tok = this.peek();
    if (tok.type === "UNIT_LITERAL" && tok.unit === "Hz") {
      this.advance();
      return tok.value as number;
    }
    if (tok.type === "NUMBER") {
      this.advance();
      if (this.check("IDENT") && this.peek().lexeme === "Hz") {
        this.advance();
      }
      return tok.value as number;
    }
    throw new ParseError("Expected frequency like 50 Hz", tok.line, tok.column);
  }

  private parseNode(): NodeDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected node name");
    let namespace: string | null = null;
    if (this.match("ON")) {
      const ns = this.expect("STRING", "Expected namespace string after 'on'");
      namespace = ns.value as string;
    }
    this.expect("SEMICOLON", "Expected ';' after node declaration");
    const end = this.previous();
    return { kind: "NodeDecl", name: name.lexeme, namespace, span: this.spanFrom(start, end) };
  }

  private parseTopic(): TopicDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected topic name");
    this.expect("COLON", "Expected ':' after topic name");
    const messageType = this.expect("IDENT", "Expected message type");
    this.expect("PUBLISH", "Expected 'publish' after message type");
    this.expect("ON", "Expected 'on' after publish");
    const topicTok = this.expect("STRING", "Expected topic string");
    this.expect("SEMICOLON", "Expected ';' after topic declaration");
    const end = this.previous();
    return {
      kind: "TopicDecl",
      name: name.lexeme,
      messageType: messageType.lexeme,
      topic: topicTok.value as string,
      span: this.spanFrom(start, end),
    };
  }

  private parseService(): ServiceDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected service name");
    this.expect("COLON", "Expected ':' after service name");
    const serviceType = this.expect("IDENT", "Expected service type");
    this.expect("SEMICOLON", "Expected ';' after service declaration");
    const end = this.previous();
    return {
      kind: "ServiceDecl",
      name: name.lexeme,
      serviceType: serviceType.lexeme,
      span: this.spanFrom(start, end),
    };
  }

  private parseAction(): ActionDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected action name");
    this.expect("COLON", "Expected ':' after action name");
    const actionType = this.expect("IDENT", "Expected action type");
    this.expect("SEMICOLON", "Expected ';' after action declaration");
    const end = this.previous();
    return {
      kind: "ActionDecl",
      name: name.lexeme,
      actionType: actionType.lexeme,
      span: this.spanFrom(start, end),
    };
  }

  private parseSensor(): SensorDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected sensor name");
    this.expect("COLON", "Expected ':' after sensor name");
    const sensorType = this.expect("IDENT", "Expected sensor type");

    let library: string | null = null;
    if (this.match("FROM")) {
      const vendor = this.expect("IDENT", "Expected library vendor in from clause");
      this.expect("DOT", "Expected '.' in library path");
      const module = this.expect("IDENT", "Expected library module in from clause");
      library = `${vendor.lexeme}.${module.lexeme}`;
    }

    let binding: SensorBinding | null = null;
    if (this.match("ON")) {
      if (this.check("STRING")) {
        const topicTok = this.advance();
        binding = { kind: "topic", path: topicTok.value as string };
      } else {
        const busName = this.expect("IDENT", "Expected HAL bus name or topic string after 'on'");
        binding = { kind: "hal", busName: busName.lexeme };
      }
    }

    this.expect("SEMICOLON", "Expected ';' after sensor declaration");
    const end = this.previous();
    return {
      kind: "SensorDecl",
      name: name.lexeme,
      sensorType: sensorType.lexeme,
      library,
      binding,
      span: this.spanFrom(start, end),
    };
  }

  private parseActuator(): ActuatorDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected actuator name");
    this.expect("COLON", "Expected ':' after actuator name");
    const actuatorType = this.expect("IDENT", "Expected actuator type");
    this.expect("SEMICOLON", "Expected ';' after actuator declaration");
    const end = this.previous();
    return {
      kind: "ActuatorDecl",
      name: name.lexeme,
      actuatorType: actuatorType.lexeme,
      span: this.spanFrom(start, end),
    };
  }

  private parseSafety(): SafetyBlock {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after safety");
    const rules: SafetyRule[] = [];
    const zones: SafetyZoneDecl[] = [];

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("STOP_IF")) {
        rules.push(this.parseStopIfRule());
      } else if (this.check("ZONE")) {
        zones.push(this.parseSafetyZone());
      } else if (this.check("IDENT")) {
        rules.push(this.parseMaxSpeedRule());
      } else {
        const t = this.peek();
        throw new ParseError("Expected safety rule or zone", t.line, t.column);
      }
    }

    const end = this.expect("RBRACE", "Expected '}' to close safety block");
    return { kind: "SafetyBlock", rules, zones, span: this.spanFrom(start, end) };
  }

  private parseAi(): AiBlock {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after ai");
    const models: AiModelDecl[] = [];

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("MODEL")) {
        models.push(this.parseAiModel());
      } else {
        const t = this.peek();
        throw new ParseError("Expected ai model declaration", t.line, t.column);
      }
    }

    const end = this.expect("RBRACE", "Expected '}' to close ai block");
    return { kind: "AiBlock", models, span: this.spanFrom(start, end) };
  }

  private parseAiModel(): AiModelDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected model name");

    let library: string | null = null;
    if (this.match("FROM")) {
      const vendor = this.expect("IDENT", "Expected library vendor");
      this.expect("DOT", "Expected '.' in library path");
      const module = this.expect("IDENT", "Expected library module");
      library = `${vendor.lexeme}.${module.lexeme}`;
    }

    this.expect("OUTPUT", "Expected 'output' in ai model declaration");
    const outputType = this.expect("IDENT", "Expected output type");

    this.expect("FILE", "Expected 'file' in ai model declaration");
    const pathTok = this.expect("STRING", "Expected model file path");

    const inputs: string[] = [];
    while (this.match("INPUT")) {
      const inputType = this.expect("IDENT", "Expected input type after input");
      inputs.push(inputType.lexeme);
    }

    this.expect("SEMICOLON", "Expected ';' after ai model declaration");
    const end = this.previous();
    return {
      kind: "AiModelDecl",
      name: name.lexeme,
      outputType: outputType.lexeme,
      path: pathTok.value as string,
      library,
      inputs,
      span: this.spanFrom(start, end),
    };
  }

  private parseSafetyZone(): SafetyZoneDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected zone name");
    let shape: "circle" | "rect" = "circle";
    if (this.match("CIRCLE")) shape = "circle";
    else if (this.match("RECT")) shape = "rect";
    else throw new ParseError("Expected 'circle' or 'rect' after zone name", this.peek().line, this.peek().column);

    this.expect("AT", "Expected 'at' in zone declaration");
    this.expect("LPAREN", "Expected '(' after 'at'");
    const x = this.parseExpr();
    this.expect("COMMA", "Expected ',' between coordinates");
    const y = this.parseExpr();
    this.expect("RPAREN", "Expected ')' after coordinates");

    let radius: Expr | null = null;
    let width: Expr | null = null;
    let height: Expr | null = null;

    if (shape === "circle") {
      this.expect("RADIUS", "Expected 'radius' for circle zone");
      radius = this.parseExpr();
    } else {
      this.expect("SIZE", "Expected 'size' for rect zone");
      this.expect("LPAREN", "Expected '(' after 'size'");
      width = this.parseExpr();
      this.expect("COMMA", "Expected ',' between size dimensions");
      height = this.parseExpr();
      this.expect("RPAREN", "Expected ')' after size");
    }

    this.expect("SEMICOLON", "Expected ';' after zone declaration");
    const end = this.previous();
    return {
      kind: "SafetyZoneDecl",
      name: name.lexeme,
      shape,
      x,
      y,
      radius,
      width,
      height,
      span: this.spanFrom(start, end),
    };
  }

  private parseMaxSpeedRule(): SafetyRule {
    const start = this.peek();
    const name = this.advance();
    this.expect("ASSIGN", "Expected '=' in safety rule");
    const value = this.parseExpr();
    let unit: UnitKind;
    if (value.kind === "UnitLiteralExpr") {
      unit = value.unit;
    } else {
      unit = this.parseUnitSuffix();
    }
    this.expect("SEMICOLON", "Expected ';' after safety rule");
    const end = this.previous();
    return {
      kind: "MaxSpeedRule",
      name: name.lexeme,
      value,
      unit,
      span: this.spanFrom(start, end),
    };
  }

  private parseStopIfRule(): SafetyRule {
    const start = this.advance();
    const condition = this.parseExpr();
    this.expect("SEMICOLON", "Expected ';' after stop_if rule");
    const end = this.previous();
    return { kind: "StopIfRule", condition, span: this.spanFrom(start, end) };
  }

  private parseBehavior(): BehaviorDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected behavior name");
    this.expect("LPAREN", "Expected '(' after behavior name");
    this.expect("RPAREN", "Expected ')' after behavior parameters");
    this.expect("LBRACE", "Expected '{' after behavior signature");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close behavior");
    return {
      kind: "BehaviorDecl",
      name: name.lexeme,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseBlock(): Stmt[] {
    const stmts: Stmt[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      stmts.push(this.parseStmt());
    }
    return stmts;
  }

  private parseStmt(): Stmt {
    const start = this.peek();

    if (this.match("LET")) {
      const name = this.expect("IDENT", "Expected variable name");
      this.expect("ASSIGN", "Expected '=' in let declaration");
      const init = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after let declaration");
      const end = this.previous();
      return {
        kind: "VarDecl",
        name: name.lexeme,
        init,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("IF")) {
      const condition = this.parseExpr();
      this.expect("LBRACE", "Expected '{' after if condition");
      const thenBranch = this.parseBlock();
      this.expect("RBRACE", "Expected '}' after if block");

      let elseBranch: Stmt[] | null = null;
      if (this.match("ELSE")) {
        this.expect("LBRACE", "Expected '{' after else");
        elseBranch = this.parseBlock();
        this.expect("RBRACE", "Expected '}' after else block");
      }

      const end = this.previous();
      return {
        kind: "IfStmt",
        condition,
        thenBranch,
        elseBranch,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("LOOP")) {
      this.expect("EVERY", "Expected 'every' after loop");
      const interval = this.parseDuration();
      this.expect("LBRACE", "Expected '{' after loop interval");
      const body = this.parseBlock();
      const end = this.expect("RBRACE", "Expected '}' to close loop");
      return {
        kind: "LoopStmt",
        intervalMs: interval,
        body,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("PUBLISH")) {
      const topicName = this.expect("IDENT", "Expected topic name after publish");
      this.expect("WITH", "Expected 'with' after topic name");
      const value = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after publish statement");
      const end = this.previous();
      return {
        kind: "PublishStmt",
        topicName: topicName.lexeme,
        value,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("CALL")) {
      const serviceName = this.expect("IDENT", "Expected service name after call");
      this.expect("LPAREN", "Expected '(' after service name");
      this.expect("RPAREN", "Expected ')' after service arguments");
      this.expect("SEMICOLON", "Expected ';' after service call");
      const end = this.previous();
      return {
        kind: "ServiceCallStmt",
        serviceName: serviceName.lexeme,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("SEND_GOAL")) {
      const actionName = this.expect("IDENT", "Expected action name after send_goal");
      this.expect("WITH", "Expected 'with' after action name");
      const goal = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after send_goal statement");
      const end = this.previous();
      return {
        kind: "ActionSendStmt",
        actionName: actionName.lexeme,
        goal,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("EMERGENCY_STOP")) {
      this.expect("SEMICOLON", "Expected ';' after emergency_stop");
      const end = this.previous();
      return { kind: "EmergencyStopStmt", span: this.spanFrom(start, end) };
    }

    if (this.match("RESET_EMERGENCY_STOP")) {
      this.expect("SEMICOLON", "Expected ';' after reset_emergency_stop");
      const end = this.previous();
      return { kind: "ResetEmergencyStopStmt", span: this.spanFrom(start, end) };
    }

    const expr = this.parseExpr();
    this.expect("SEMICOLON", "Expected ';' after expression");
    const end = this.previous();
    return { kind: "ExprStmt", expr, span: this.spanFrom(start, end) };
  }

  private parseDuration(): number {
    const tok = this.peek();
    if (tok.type === "UNIT_LITERAL" && tok.unit === "ms") {
      this.advance();
      return tok.value as number;
    }
    if (tok.type === "UNIT_LITERAL" && tok.unit === "s") {
      this.advance();
      return (tok.value as number) * 1000;
    }
    if (tok.type === "NUMBER") {
      this.advance();
      if (this.check("IDENT") && this.peek().lexeme === "ms") {
        this.advance();
        return tok.value as number;
      }
    }
    throw new ParseError("Expected duration like 50ms", tok.line, tok.column);
  }

  private parseUnitSuffix(): UnitKind {
    const unit = this.tryParseUnitSuffix();
    if (!unit) {
      const t = this.peek();
      throw new ParseError("Expected unit suffix", t.line, t.column);
    }
    return unit;
  }

  private tryParseUnitSuffix(): UnitKind | null {
    if (this.check("UNIT_LITERAL")) {
      const t = this.advance();
      return unitFromLexeme(t.unit!);
    }

    if (this.check("IDENT") && this.peek().lexeme === "m" && this.tokens[this.pos + 1]?.type === "SLASH" && this.tokens[this.pos + 2]?.lexeme === "s") {
      this.advance();
      this.advance();
      this.advance();
      return "m/s";
    }

    if (this.check("IDENT") && this.peek().lexeme === "rad" && this.tokens[this.pos + 1]?.type === "SLASH" && this.tokens[this.pos + 2]?.lexeme === "s") {
      this.advance();
      this.advance();
      this.advance();
      return "rad/s";
    }

    if (this.check("IDENT")) {
      const lexeme = this.peek().lexeme;
      if (isUnitIdent(lexeme)) {
        this.advance();
        return unitFromLexeme(lexeme as import("../lexer/index.js").UnitLexeme);
      }
    }

    return null;
  }

  private parseExpr(): Expr {
    return this.parseOr();
  }

  private parseOr(): Expr {
    let left = this.parseAnd();
    while (this.match("OR")) {
      const opStart = this.previous();
      const right = this.parseAnd();
      left = {
        kind: "BinaryExpr",
        op: "or",
        left,
        right,
        span: this.spanFrom(
          { ...opStart, type: "OR" },
          this.previous(),
        ),
      };
    }
    return left;
  }

  private parseAnd(): Expr {
    let left = this.parseComparison();
    while (this.match("AND")) {
      const opStart = this.previous();
      const right = this.parseComparison();
      left = {
        kind: "BinaryExpr",
        op: "and",
        left,
        right,
        span: this.spanFrom(opStart, this.previous()),
      };
    }
    return left;
  }

  private parseComparison(): Expr {
    let left = this.parseAdditive();
    while (
      this.match("LT", "LTE", "GT", "GTE", "EQ", "NEQ")
    ) {
      const opTok = this.previous();
      const op = opTok.lexeme as BinaryOp;
      const right = this.parseAdditive();
      left = {
        kind: "BinaryExpr",
        op,
        left,
        right,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return left;
  }

  private parseAdditive(): Expr {
    let left = this.parseMultiplicative();
    while (this.match("PLUS", "MINUS")) {
      const opTok = this.previous();
      const op = opTok.lexeme as BinaryOp;
      const right = this.parseMultiplicative();
      left = {
        kind: "BinaryExpr",
        op,
        left,
        right,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return left;
  }

  private parseMultiplicative(): Expr {
    let left = this.parseUnary();
    while (this.match("STAR", "SLASH")) {
      const opTok = this.previous();
      const op = opTok.lexeme as BinaryOp;
      const right = this.parseUnary();
      left = {
        kind: "BinaryExpr",
        op,
        left,
        right,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return left;
  }

  private parseUnary(): Expr {
    if (this.match("MINUS", "NOT")) {
      const opTok = this.previous();
      const op = (opTok.type === "NOT" ? "not" : "-") as import("../ast/nodes.js").UnaryOp;
      const operand = this.parseUnary();
      return {
        kind: "UnaryExpr",
        op,
        operand,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return this.parsePostfix();
  }

  private parsePostfix(): Expr {
    let expr = this.parsePrimary();

    while (true) {
      if (this.match("DOT")) {
        const prop = this.expect("IDENT", "Expected property name after '.'");
        expr = {
          kind: "MemberExpr",
          object: expr,
          property: prop.lexeme,
          span: this.spanFrom(
            { ...prop, type: "DOT" },
            prop,
          ),
        };
      } else if (this.match("LPAREN")) {
        const args: Expr[] = [];
        const namedArgs: NamedArg[] = [];

        if (!this.check("RPAREN")) {
          do {
            if (this.isNamedArgStart()) {
              const name = this.parseNamedArgName();
              this.advance(); // colon
              const value = this.parseExpr();
              namedArgs.push({
                name,
                value,
                span: this.spanFrom(this.previous(), this.previous()),
              });
            } else {
              args.push(this.parseExpr());
            }
          } while (this.match("COMMA"));
        }

        const end = this.expect("RPAREN", "Expected ')' after arguments");
        expr = {
          kind: "CallExpr",
          callee: expr,
          args,
          namedArgs,
          span: this.spanFrom(
            { line: expr.span.start.line, column: expr.span.start.column, offset: 0, type: "LPAREN", lexeme: "(", value: null },
            end,
          ),
        };
      } else {
        break;
      }
    }

    return expr;
  }

  private parsePrimary(): Expr {
    const start = this.peek();

    if (this.match("INFER")) {
      const modelName = this.expect("IDENT", "Expected model name after infer");
      this.expect("WITH", "Expected 'with' after model name");
      const namedArgs: NamedArg[] = [];

      if (!this.check("SEMICOLON") && !this.check("RBRACE") && !this.check("RPAREN") && !this.check("EOF")) {
        do {
          const argStart = this.peek();
          const argName = this.parseNamedArgName();
          this.advance(); // colon
          const value = this.parseExpr();
          namedArgs.push({
            name: argName,
            value,
            span: this.spanFrom(argStart, this.previous()),
          });
        } while (this.match("COMMA"));
      }

      const end = this.previous();
      return {
        kind: "InferExpr",
        modelName: modelName.lexeme,
        namedArgs,
        span: this.spanFrom(start, end),
      };
    }
    if (this.match("TRUE")) {
      return {
        kind: "LiteralExpr",
        value: true,
        span: this.spanFrom(start, this.previous()),
      };
    }
    if (this.match("FALSE")) {
      return {
        kind: "LiteralExpr",
        value: false,
        span: this.spanFrom(start, this.previous()),
      };
    }
    if (this.match("ROBOT")) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: "robot",
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("NUMBER")) {
      const tok = this.previous();
      const unit = this.tryParseUnitSuffix();
      if (unit) {
        return {
          kind: "UnitLiteralExpr",
          value: tok.value as number,
          unit,
          span: this.spanFrom(start, this.previous()),
        };
      }
      return {
        kind: "LiteralExpr",
        value: tok.value as number,
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("UNIT_LITERAL")) {
      const tok = this.previous();
      return {
        kind: "UnitLiteralExpr",
        value: tok.value as number,
        unit: unitFromLexeme(tok.unit!),
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("STRING")) {
      return {
        kind: "LiteralExpr",
        value: this.previous().value as string,
        span: this.spanFrom(start, this.previous()),
      };
    }
    if (this.match("IDENT")) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: tok.lexeme,
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("LPAREN")) {
      const expr = this.parseExpr();
      const end = this.expect("RPAREN", "Expected ')' after expression");
      return { ...expr, span: this.spanFrom(start, end) };
    }

    const t = this.peek();
    throw new ParseError("Expected expression", t.line, t.column);
  }

  private isNamedArgStart(): boolean {
    const next = this.tokens[this.pos + 1];
    if (next?.type !== "COLON") return false;
    return this.check("IDENT") || this.check("FROM");
  }

  private parseNamedArgName(): string {
    if (this.match("FROM")) return "from";
    return this.advance().lexeme;
  }
}

export { parse as parseTokens };

function isUnitIdent(lexeme: string): boolean {
  return ["m", "s", "ms", "rad", "deg", "Hz"].includes(lexeme);
}
