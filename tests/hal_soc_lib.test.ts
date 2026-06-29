import { describe, it, expect } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { compile } from "../src/compile.js";
import { run } from "../src/cli/run-program.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { createSimHal } from "../src/hal/index.js";
import { getSocProfile, validateHalAgainstSoc, listSocProfiles } from "../src/soc/index.js";
import { resolveImport, listLibraries, listLibrariesByVendor } from "../src/lib/index.js";
import { TypeCheckError } from "../src/types/index.js";

describe("HAL", () => {
  it("parses hal block with buses and pins", () => {
    const source = `
      robot R {
        hal {
          i2c imu_bus at 0x68;
          gpio led out pin 13;
          pwm servo on pin 25 frequency 50 Hz;
          uart gps on "/dev/ttyUSB0" baud 9600;
        }
      }
    `;
    const ast = parse(tokenize(source));
    expect(ast.robots[0].hal!.members).toHaveLength(4);
  });

  it("simulates I2C read/write", () => {
    const hal = createSimHal();
    hal.configure([{ kind: "i2c", name: "bus", address: 0x68 }]);
    hal.writeI2c("bus", 0x10, [0xab, 0xcd]);
    expect(hal.readI2c("bus", 0x10, 2)).toEqual([0xab, 0xcd]);
  });
});

describe("SoC", () => {
  it("lists supported SoC profiles", () => {
    const profiles = listSocProfiles();
    expect(profiles.length).toBeGreaterThanOrEqual(6);
    expect(profiles.some((p) => p.name === "ESP32")).toBe(true);
    expect(profiles.some((p) => p.name === "STM32H7")).toBe(true);
    expect(profiles.some((p) => p.name === "RaspberryPi4")).toBe(true);
  });

  it("validates HAL against SoC limits", () => {
    const profile = getSocProfile("ArduinoUno")!;
    const errors = validateHalAgainstSoc(profile, [
      { kind: "i2c", name: "b1", address: 0x68 },
      { kind: "i2c", name: "b2", address: 0x69 },
    ]);
    expect(errors.length).toBeGreaterThan(0);
  });

  it("parses soc declaration", () => {
    const ast = parse(tokenize("robot R { soc ESP32; }"));
    expect(ast.robots[0].soc?.profile).toBe("ESP32");
  });
});

describe("sensor libraries", () => {
  it("resolves vendor libraries", () => {
    expect(resolveImport("bosch.bno055")?.vendor).toBe("Bosch");
    expect(resolveImport("velodyne.vlp16")?.sensors.VelodyneVLP16).toBeDefined();
    expect(resolveImport("intel.realsense")?.sensors.IntelRealSenseD435).toBeDefined();
    expect(resolveImport("ouster.os1")?.sensors.OusterOS1).toBeDefined();
  });

  it("lists libraries by vendor", () => {
    expect(listLibrariesByVendor("Hokuyo").length).toBe(2);
    expect(listLibraries().length).toBeGreaterThanOrEqual(10);
  });

  it("type-checks import and from clauses", () => {
    expect(() =>
      compile(`
        import bosch.bno055;
        robot R {
          soc ESP32;
          hal { i2c imu at 0x68; }
          sensor imu: BoschBNO055 from bosch.bno055 on imu;
        }
      `),
    ).not.toThrow();
  });

  it("rejects unimported library", () => {
    expect(() =>
      compile(`
        robot R {
          sensor imu: BoschBNO055 from bosch.bno055 on imu;
        }
      `),
    ).toThrow(TypeCheckError);
  });

  it("runs ESP32 multi-sensor example", () => {
    const { program } = compile(`
      import bosch.bmp388;
      import adafruit.vl53l0x;
      robot R {
        soc ESP32;
        hal { i2c bus at 0x76; }
        actuator wheels: DifferentialDrive;
        sensor baro: BoschBMP388 from bosch.bmp388 on bus;
        sensor tof: AdafruitVL53L0X from adafruit.vl53l0x on bus;
        behavior go() {
          loop every 50ms {
            let d = tof.read();
            if d < 0.3 m { wheels.stop(); } else { wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s); }
          }
        }
      }
    `);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 5 });
    expect(sim.getEventLog().some((e) => e.includes("drive") || e.includes("stop"))).toBe(true);
  });
});
