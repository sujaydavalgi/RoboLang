export type TokenType =
  | "IMPORT"
  | "HAL"
  | "SOC"
  | "FROM"
  | "I2C"
  | "SPI"
  | "UART"
  | "GPIO"
  | "PWM"
  | "ADC"
  | "OUT"
  | "IN"
  | "BAUD"
  | "FREQUENCY"
  | "PIN"
  | "ROBOT"
  | "NODE"
  | "TOPIC"
  | "SERVICE"
  | "ACTION"
  | "SENSOR"
  | "ACTUATOR"
  | "SAFETY"
  | "AI"
  | "MODEL"
  | "FILE"
  | "INFER"
  | "OUTPUT"
  | "INPUT"
  | "BEHAVIOR"
  | "LOOP"
  | "EVERY"
  | "LET"
  | "IF"
  | "ELSE"
  | "STOP_IF"
  | "PUBLISH"
  | "CALL"
  | "SEND_GOAL"
  | "WITH"
  | "ZONE"
  | "CIRCLE"
  | "RECT"
  | "AT"
  | "RADIUS"
  | "SIZE"
  | "EMERGENCY_STOP"
  | "RESET_EMERGENCY_STOP"
  | "ON"
  | "TRUE"
  | "FALSE"
  | "AND"
  | "OR"
  | "NOT"
  | "IDENT"
  | "NUMBER"
  | "STRING"
  | "UNIT_LITERAL"
  | "LBRACE"
  | "RBRACE"
  | "LPAREN"
  | "RPAREN"
  | "SEMICOLON"
  | "COLON"
  | "COMMA"
  | "DOT"
  | "ASSIGN"
  | "PLUS"
  | "MINUS"
  | "STAR"
  | "SLASH"
  | "LT"
  | "LTE"
  | "GT"
  | "GTE"
  | "EQ"
  | "NEQ"
  | "EOF";

export type UnitLexeme = "m" | "s" | "ms" | "rad" | "m/s" | "rad/s" | "deg" | "Hz";

export type Token = {
  type: TokenType;
  lexeme: string;
  value: string | number | boolean | null;
  unit?: UnitLexeme;
  line: number;
  column: number;
  offset: number;
};

export class LexerError extends Error {
  constructor(
    message: string,
    public line: number,
    public column: number,
  ) {
    super(message);
    this.name = "LexerError";
  }
}

const KEYWORDS: Record<string, TokenType> = {
  import: "IMPORT",
  hal: "HAL",
  soc: "SOC",
  from: "FROM",
  i2c: "I2C",
  spi: "SPI",
  uart: "UART",
  gpio: "GPIO",
  pwm: "PWM",
  adc: "ADC",
  out: "OUT",
  in: "IN",
  baud: "BAUD",
  frequency: "FREQUENCY",
  pin: "PIN",
  robot: "ROBOT",
  node: "NODE",
  topic: "TOPIC",
  service: "SERVICE",
  action: "ACTION",
  sensor: "SENSOR",
  actuator: "ACTUATOR",
  safety: "SAFETY",
  ai: "AI",
  model: "MODEL",
  file: "FILE",
  infer: "INFER",
  output: "OUTPUT",
  input: "INPUT",
  behavior: "BEHAVIOR",
  loop: "LOOP",
  every: "EVERY",
  let: "LET",
  if: "IF",
  else: "ELSE",
  stop_if: "STOP_IF",
  publish: "PUBLISH",
  call: "CALL",
  send_goal: "SEND_GOAL",
  with: "WITH",
  zone: "ZONE",
  circle: "CIRCLE",
  rect: "RECT",
  at: "AT",
  radius: "RADIUS",
  size: "SIZE",
  emergency_stop: "EMERGENCY_STOP",
  reset_emergency_stop: "RESET_EMERGENCY_STOP",
  on: "ON",
  true: "TRUE",
  false: "FALSE",
  and: "AND",
  or: "OR",
  not: "NOT",
};

const UNIT_SUFFIXES: UnitLexeme[] = ["m/s", "rad/s", "ms", "deg", "rad", "m", "s", "Hz"];

export function tokenize(source: string): Token[] {
  const tokens: Token[] = [];
  let line = 1;
  let column = 1;
  let i = 0;

  const loc = () => ({ line, column, offset: i });

  while (i < source.length) {
    const ch = source[i];

    if (ch === " " || ch === "\t" || ch === "\r") {
      i++;
      column++;
      continue;
    }

    if (ch === "\n") {
      i++;
      line++;
      column = 1;
      continue;
    }

    if (ch === "/" && source[i + 1] === "/") {
      while (i < source.length && source[i] !== "\n") {
        i++;
      }
      continue;
    }

    const start = loc();

    if (ch === "{") {
      tokens.push({ type: "LBRACE", lexeme: "{", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "}") {
      tokens.push({ type: "RBRACE", lexeme: "}", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "(") {
      tokens.push({ type: "LPAREN", lexeme: "(", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ")") {
      tokens.push({ type: "RPAREN", lexeme: ")", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ";") {
      tokens.push({ type: "SEMICOLON", lexeme: ";", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ":") {
      tokens.push({ type: "COLON", lexeme: ":", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ",") {
      tokens.push({ type: "COMMA", lexeme: ",", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ".") {
      tokens.push({ type: "DOT", lexeme: ".", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "+") {
      tokens.push({ type: "PLUS", lexeme: "+", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "-") {
      tokens.push({ type: "MINUS", lexeme: "-", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "*") {
      tokens.push({ type: "STAR", lexeme: "*", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "/") {
      tokens.push({ type: "SLASH", lexeme: "/", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "<" && source[i + 1] === "=") {
      tokens.push({ type: "LTE", lexeme: "<=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "<") {
      tokens.push({ type: "LT", lexeme: "<", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ">" && source[i + 1] === "=") {
      tokens.push({ type: "GTE", lexeme: ">=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === ">") {
      tokens.push({ type: "GT", lexeme: ">", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "=" && source[i + 1] === "=") {
      tokens.push({ type: "EQ", lexeme: "==", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "!" && source[i + 1] === "=") {
      tokens.push({ type: "NEQ", lexeme: "!=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "=") {
      tokens.push({ type: "ASSIGN", lexeme: "=", value: null, ...start });
      i++;
      column++;
      continue;
    }

    if (ch === '"') {
      i++;
      column++;
      let value = "";
      while (i < source.length && source[i] !== '"') {
        if (source[i] === "\\" && i + 1 < source.length) {
          value += source[i + 1];
          i += 2;
          column += 2;
        } else {
          value += source[i];
          i++;
          column++;
        }
      }
      if (i >= source.length) {
        throw new LexerError("Unterminated string", line, column);
      }
      i++;
      column++;
      tokens.push({ type: "STRING", lexeme: value, value, ...start });
      continue;
    }

    if (ch === "0" && (source[i + 1] === "x" || source[i + 1] === "X")) {
      i += 2;
      column += 2;
      let hexStr = "";
      while (i < source.length && isHexDigit(source[i])) {
        hexStr += source[i];
        i++;
        column++;
      }
      const num = parseInt(hexStr, 16);
      tokens.push({ type: "NUMBER", lexeme: `0x${hexStr}`, value: num, ...start });
      continue;
    }

    if (isDigit(ch) || (ch === "." && isDigit(source[i + 1]))) {
      let numStr = "";
      while (i < source.length && (isDigit(source[i]) || source[i] === ".")) {
        numStr += source[i];
        i++;
        column++;
      }
      const num = parseFloat(numStr);

      let matchedUnit: UnitLexeme | undefined;
      while (i < source.length && (source[i] === " " || source[i] === "\t")) {
        i++;
        column++;
      }
      for (const suffix of UNIT_SUFFIXES) {
        if (source.slice(i, i + suffix.length) === suffix) {
          const next = source[i + suffix.length];
          if (next && (isIdentChar(next) || next === "/")) continue;
          matchedUnit = suffix;
          i += suffix.length;
          column += suffix.length;
          break;
        }
      }

      if (matchedUnit) {
        tokens.push({
          type: "UNIT_LITERAL",
          lexeme: `${numStr}${matchedUnit}`,
          value: num,
          unit: matchedUnit,
          ...start,
        });
      } else {
        tokens.push({ type: "NUMBER", lexeme: numStr, value: num, ...start });
      }
      continue;
    }

    if (isIdentStart(ch)) {
      let ident = "";
      while (i < source.length && isIdentChar(source[i])) {
        ident += source[i];
        i++;
        column++;
      }
      const kw = KEYWORDS[ident];
      tokens.push({
        type: kw ?? "IDENT",
        lexeme: ident,
        value: ident,
        ...start,
      });
      continue;
    }

    throw new LexerError(`Unexpected character '${ch}'`, line, column);
  }

  tokens.push({
    type: "EOF",
    lexeme: "",
    value: null,
    line,
    column,
    offset: i,
  });
  return tokens;
}

function isHexDigit(ch: string): boolean {
  return isDigit(ch) || (ch >= "a" && ch <= "f") || (ch >= "A" && ch <= "F");
}

function isDigit(ch: string): boolean {
  return ch >= "0" && ch <= "9";
}

function isIdentStart(ch: string): boolean {
  return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || ch === "_";
}

function isIdentChar(ch: string): boolean {
  return isIdentStart(ch) || isDigit(ch);
}

export function unitFromLexeme(lexeme: UnitLexeme): import("../ast/nodes.js").UnitKind {
  return lexeme;
}
