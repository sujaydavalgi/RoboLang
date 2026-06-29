import { describe, it, expect } from "vitest";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import {
  RoutingCommBus,
  TlsTransportSession,
  transportSecurityFromBusFields,
} from "../src/transport/index.js";
import { runtimeVelocity } from "../src/runtime/values.js";
import { SecurityContext } from "../src/security/index.js";
import { securityCheck, reportHasErrors } from "../src/security/validate.js";

describe("transport wire frames", () => {
  it("preserves source_id through encrypted mqtt poll", () => {
    const tls = new TlsTransportSession();
    const security = transportSecurityFromBusFields("required", "none", "none");
    security.certPath = "certs/test.pem";
    security.keySecret = "test_key";
    tls.connect(security);
    const bus = new RoutingCommBus();
    bus.configure({ security, tls });
    bus.subscribe("/motion", "motion_cmd");
    bus.reconnectTransport("mqtt");
    bus.publish("/motion", "Velocity", runtimeVelocity(0.5, 0), "mqtt", "Navigator");
    const inbound = bus.pollInbound("mqtt");
    expect(inbound).toHaveLength(1);
    expect(inbound[0]![1].sourceId).toBe("Navigator");
  });
});

describe("trusted source enforcement", () => {
  it("rejects untrusted agent publish at runtime", () => {
    const source = `
robot R {
  trust trusted;
  permissions [
    crypto.encrypt,
    identity.sign,
    identity.verify,
    secure_topic.publish
  ];
  identity RoverIdentity { id: "rover"; public_key: "k1"; }
  topic motion_cmd: Velocity publish on "/motion" secure {
    encryption required;
    signed required;
    trusted_sources [Navigator];
    reject_untrusted true;
  };
  agent BadAgent {
    uses planner;
    tools [];
    goal "bad";
    can [ publish(motion_cmd), plan ];
    plan { publish motion_cmd with velocity(linear: 0.0 m/s, angular: 0.0 rad/s); }
  }
  ai_model planner: LLM { provider: "mock"; model: "test"; temperature: 0.1; }
  behavior run() { BadAgent.plan(); }
}
`;
    expect(() =>
      run(compile(source).program, {
        backend: createDefaultSimulator(),
        maxLoopIterations: 1,
        onLog: () => {},
      }),
    ).toThrow(/untrusted/i);
  });

  it("enforces trusted source on inbound receive", () => {
    const ctx = new SecurityContext();
    ctx.capabilities.grant("secure_topic.subscribe");
    ctx.secureEndpoints.register("/motion", {
      signed: false,
      minTrust: null,
      requires: [],
      encryption: "none",
      authentication: "none",
      integrity: "none",
      trustedSources: ["Navigator"],
      rejectUntrusted: true,
    });
    expect(() => ctx.verifyInboundMessage("/motion", "{}", "BadAgent")).toThrow(/untrusted/i);
    expect(() => ctx.verifyInboundMessage("/motion", "{}", "Navigator")).not.toThrow();
  });
});

describe("security check", () => {
  it("rejects encrypted bus without secrets", () => {
    const source = `
robot R {
  bus mesh { transport: "mqtt"; encryption: required; }
  behavior run() {}
}
`;
    const report = securityCheck(source);
    expect(reportHasErrors(report)).toBe(true);
    expect(report.findings.some((f) => f.message.includes("encrypted bus"))).toBe(true);
  });
});

describe("runtime trust boundaries", () => {
  it("rejects unencrypted publish crossing robot_to_robot on mqtt", () => {
    const source = `
robot R {
  trust trusted;
  trust_boundary robot_to_robot;
  topic motion_cmd: Velocity publish on "/motion";
  bus mesh { transport: "mqtt"; };
  behavior run() {
    publish motion_cmd with velocity(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
}
`;
    expect(() =>
      run(compile(source).program, {
        backend: createDefaultSimulator(),
        maxLoopIterations: 1,
        onLog: () => {},
      }),
    ).toThrow(/encryption/i);
  });
});

describe("broker url", () => {
  it("parses bus url field", () => {
    const source = `
robot R {
  bus mesh {
    transport: "mqtt";
    url: "mqtts://broker.example.com:8883";
  };
  behavior run() {}
}
`;
    const program = compile(source).program;
    expect(program.robots[0]!.buses[0]!.brokerUrl).toBe("mqtts://broker.example.com:8883");
  });
});
