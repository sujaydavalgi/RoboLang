/**
 * registry module (lib/registry.ts).
 * @module
 */

import type { HalBackend } from "../hal/index.js";
import type { RuntimeValue } from "../runtime/interpreter.js";

export type SensorInterface = "i2c" | "spi" | "uart" | "usb" | "ethernet" | "gpio";

export type SensorDriverDef = {
  sensorType: string;
  vendor: string;
  model: string;
  interfaces: SensorInterface[];
  defaultBus?: SensorInterface;
  methods: string[];
  read: (ctx: DriverContext) => RuntimeValue;
};

export type DriverContext = {
  hal: HalBackend | null;
  halBinding: string | null;
  topic: string | null;
  simState?: { pose: { x: number; y: number; theta: number; z?: number } };
};

export type LibModule = {
  id: string;
  vendor: string;
  name: string;
  version: string;
  description: string;
  sensors: Record<string, SensorDriverDef>;
};

function scanReading(ctx: DriverContext, range = 10): RuntimeValue {
  // ScanReading.
  //
  // Parameters:
  // - `ctx` — input value
  // - `range` — optional input
  //
  // Returns:
  // `RuntimeValue`.
  //
  // Options:
  // - `range` — optional parameter
  //
  // Example:

  // const result = scanReading(ctx, range);
  const x = ctx.simState?.pose.x ?? 0;
  const nearest = Math.max(0.05, range - Math.abs(x) * 0.3);
  return { kind: "scan", nearestDistance: nearest };
}

function imuReading(yaw = 0): RuntimeValue {
  // ImuReading.
  //
  // Parameters:
  // - `yaw` — optional input
  //
  // Returns:
  // `RuntimeValue`.
  //
  // Options:
  // - `yaw` — optional parameter
  //
  // Example:

  // const result = imuReading(yaw);
  return {
    kind: "object",
    typeName: "IMUReading",
    fields: {
      roll: { kind: "number", value: 0, unit: "rad" },
      pitch: { kind: "number", value: 0, unit: "rad" },
      yaw: { kind: "number", value: yaw, unit: "rad" },
    },
  };
}

export const LIB_REGISTRY: Record<string, LibModule> = {
  "velodyne.vlp16": {
    id: "velodyne.vlp16",
    vendor: "Velodyne",
    name: "vlp16",
    version: "1.0.0",
    description: "Velodyne VLP-16 3D LiDAR puck",
    sensors: {
      VelodyneVLP16: {
        sensorType: "VelodyneVLP16",
        vendor: "Velodyne",
        model: "VLP-16",
        interfaces: ["ethernet", "usb"],
        methods: ["read", "calibrate"],
        read: (ctx) => scanReading(ctx, 100),
      },
    },
  },
  "velodyne.vlp32": {
    id: "velodyne.vlp32",
    vendor: "Velodyne",
    name: "vlp32",
    version: "1.0.0",
    description: "Velodyne VLP-32C ultra puck",
    sensors: {
      VelodyneVLP32: {
        sensorType: "VelodyneVLP32",
        vendor: "Velodyne",
        model: "VLP-32C",
        interfaces: ["ethernet"],
        methods: ["read"],
        read: (ctx) => scanReading(ctx, 200),
      },
    },
  },
  "hokuyo.ust10": {
    id: "hokuyo.ust10",
    vendor: "Hokuyo",
    name: "ust10",
    version: "1.0.0",
    description: "Hokuyo UST-10LX 2D LiDAR",
    sensors: {
      HokuyoUST10: {
        sensorType: "HokuyoUST10",
        vendor: "Hokuyo",
        model: "UST-10LX",
        interfaces: ["ethernet", "uart"],
        methods: ["read"],
        read: (ctx) => scanReading(ctx, 10),
      },
    },
  },
  "hokuyo.utm30": {
    id: "hokuyo.utm30",
    vendor: "Hokuyo",
    name: "utm30",
    version: "1.0.0",
    description: "Hokuyo UTM-30LX-EW outdoor LiDAR",
    sensors: {
      HokuyoUTM30: {
        sensorType: "HokuyoUTM30",
        vendor: "Hokuyo",
        model: "UTM-30LX-EW",
        interfaces: ["ethernet"],
        methods: ["read"],
        read: (ctx) => scanReading(ctx, 30),
      },
    },
  },
  "bosch.bno055": {
    id: "bosch.bno055",
    vendor: "Bosch",
    name: "bno055",
    version: "1.0.0",
    description: "Bosch BNO055 9-DOF absolute orientation IMU",
    sensors: {
      BoschBNO055: {
        sensorType: "BoschBNO055",
        vendor: "Bosch",
        model: "BNO055",
        interfaces: ["i2c", "uart"],
        defaultBus: "i2c",
        methods: ["read", "calibrate"],
        read: (ctx) => {
          const yaw = ctx.simState?.pose.theta ?? 0;
          if (ctx.hal && ctx.halBinding) {
            const data = ctx.hal.readI2c(ctx.halBinding, 0x1a, 2);
            const raw = data[0] | (data[1] << 8);
            return imuReading(raw / 100);
          }
          return imuReading(yaw);
        },
      },
    },
  },
  "bosch.bmp388": {
    id: "bosch.bmp388",
    vendor: "Bosch",
    name: "bmp388",
    version: "1.0.0",
    description: "Bosch BMP388 barometric pressure sensor",
    sensors: {
      BoschBMP388: {
        sensorType: "BoschBMP388",
        vendor: "Bosch",
        model: "BMP388",
        interfaces: ["i2c", "spi"],
        defaultBus: "i2c",
        methods: ["read"],
        read: (ctx) => {
          const alt = ctx.simState?.pose.z ?? 0;
          return { kind: "number", value: alt, unit: "m" };
        },
      },
    },
  },
  "bosch.bme280": {
    id: "bosch.bme280",
    vendor: "Bosch",
    name: "bme280",
    version: "1.0.0",
    description: "Bosch BME280 environmental sensor (humidity, pressure, temperature)",
    sensors: {
      BoschBME280: {
        sensorType: "BoschBME280",
        vendor: "Bosch",
        model: "BME280",
        interfaces: ["i2c", "spi"],
        defaultBus: "i2c",
        methods: ["read", "calibrate"],
        read: (ctx) => {
          const x = ctx.simState?.pose.x ?? 0;
          const humidity = Math.max(30, Math.min(90, 55 - x * 2));
          return { kind: "number", value: humidity, unit: "rh" };
        },
      },
    },
  },
  "adafruit.bh1750": {
    id: "adafruit.bh1750",
    vendor: "Adafruit",
    name: "bh1750",
    version: "1.0.0",
    description: "Adafruit BH1750 digital light sensor",
    sensors: {
      AdafruitBH1750: {
        sensorType: "AdafruitBH1750",
        vendor: "Adafruit",
        model: "BH1750",
        interfaces: ["i2c"],
        defaultBus: "i2c",
        methods: ["read"],
        read: (ctx) => {
          const x = ctx.simState?.pose.x ?? 0;
          const lux = Math.max(0, Math.min(100_000, 400 - x * 20));
          return { kind: "number", value: lux, unit: "lux" };
        },
      },
    },
  },
  "intel.realsense": {
    id: "intel.realsense",
    vendor: "Intel",
    name: "realsense",
    version: "1.0.0",
    description: "Intel RealSense depth cameras",
    sensors: {
      IntelRealSenseD435: {
        sensorType: "IntelRealSenseD435",
        vendor: "Intel",
        model: "D435",
        interfaces: ["usb"],
        methods: ["read", "read_depth"],
        read: (ctx) => scanReading(ctx, 5),
      },
      IntelRealSenseD455: {
        sensorType: "IntelRealSenseD455",
        vendor: "Intel",
        model: "D455",
        interfaces: ["usb"],
        methods: ["read", "read_depth"],
        read: (ctx) => scanReading(ctx, 8),
      },
    },
  },
  "ydlidar.x4": {
    id: "ydlidar.x4",
    vendor: "YDLIDAR",
    name: "x4",
    version: "1.0.0",
    description: "YDLIDAR X4 2D LiDAR",
    sensors: {
      YdlidarX4: {
        sensorType: "YdlidarX4",
        vendor: "YDLIDAR",
        model: "X4",
        interfaces: ["uart", "usb"],
        defaultBus: "uart",
        methods: ["read"],
        read: (ctx) => scanReading(ctx, 6),
      },
    },
  },
  "ydlidar.g4": {
    id: "ydlidar.g4",
    vendor: "YDLIDAR",
    name: "g4",
    version: "1.0.0",
    description: "YDLIDAR G4 2D LiDAR",
    sensors: {
      YdlidarG4: {
        sensorType: "YdlidarG4",
        vendor: "YDLIDAR",
        model: "G4",
        interfaces: ["uart", "usb"],
        methods: ["read"],
        read: (ctx) => scanReading(ctx, 16),
      },
    },
  },
  "adafruit.vl53l0x": {
    id: "adafruit.vl53l0x",
    vendor: "Adafruit",
    name: "vl53l0x",
    version: "1.0.0",
    description: "Adafruit VL53L0X time-of-flight distance sensor",
    sensors: {
      AdafruitVL53L0X: {
        sensorType: "AdafruitVL53L0X",
        vendor: "Adafruit",
        model: "VL53L0X",
        interfaces: ["i2c"],
        defaultBus: "i2c",
        methods: ["read"],
        read: (ctx) => {
          const dist = Math.max(0.02, 2.0 - (ctx.simState?.pose.x ?? 0) * 0.1);
          return { kind: "number", value: dist, unit: "m" };
        },
      },
    },
  },
  "sparkfun.lsm9ds1": {
    id: "sparkfun.lsm9ds1",
    vendor: "SparkFun",
    name: "lsm9ds1",
    version: "1.0.0",
    description: "SparkFun LSM9DS1 9-DOF IMU breakout",
    sensors: {
      SparkfunLSM9DS1: {
        sensorType: "SparkfunLSM9DS1",
        vendor: "SparkFun",
        model: "LSM9DS1",
        interfaces: ["i2c", "spi"],
        defaultBus: "i2c",
        methods: ["read"],
        read: (ctx) => imuReading(ctx.simState?.pose.theta ?? 0),
      },
    },
  },
  "waveshare.uwmf": {
    id: "waveshare.uwmf",
    vendor: "Waveshare",
    name: "uwmf",
    version: "1.0.0",
    description: "Waveshare ultrasonic distance module",
    sensors: {
      WaveshareUWMF: {
        sensorType: "WaveshareUWMF",
        vendor: "Waveshare",
        model: "UWMF",
        interfaces: ["gpio", "uart"],
        methods: ["read"],
        read: (ctx) => {
          const dist = Math.max(0.02, 4.0 - (ctx.simState?.pose.x ?? 0) * 0.2);
          return { kind: "number", value: dist, unit: "m" };
        },
      },
    },
  },
  "ouster.os1": {
    id: "ouster.os1",
    vendor: "Ouster",
    name: "os1",
    version: "1.0.0",
    description: "Ouster OS1 digital LiDAR sensor",
    sensors: {
      OusterOS1: {
        sensorType: "OusterOS1",
        vendor: "Ouster",
        model: "OS1",
        interfaces: ["ethernet"],
        defaultBus: "ethernet",
        methods: ["read", "calibrate"],
        read: (ctx) => scanReading(ctx, 120),
      },
    },
  },
};

export function resolveImport(path: string): LibModule | undefined {
  // ResolveImport.
  //
  // Parameters:
  // - `path` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resolveImport(path);
  return LIB_REGISTRY[path];
}

export function getSensorDriver(libraryId: string, sensorType: string): SensorDriverDef | undefined {
  // GetSensorDriver.
  //
  // Parameters:
  // - `libraryId` — input value
  // - `sensorType` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = getSensorDriver(libraryId, sensorType);
  const lib = LIB_REGISTRY[libraryId];

  // continue when lib is falsy.
  if (!lib) return undefined;
  return lib.sensors[sensorType];
}

export function getSensorTypeFromLib(libraryId: string, sensorType: string): boolean {
  // GetSensorTypeFromLib.
  //
  // Parameters:
  // - `libraryId` — input value
  // - `sensorType` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = getSensorTypeFromLib(libraryId, sensorType);
  return getSensorDriver(libraryId, sensorType) !== undefined;
}

export function allLibrarySensorTypes(): Record<string, {
  // AllLibrarySensorTypes.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // `Record<string,`.
  //
  // Options:
  // None.
  //
  // Example:

 // const result = allLibrarySensorTypes();
 roboType: { kind: "named"; name: string }; library: string }> {
  const result: Record<string, { roboType: { kind: "named"; name: string }; library: string }> = {};
  for (const [libId, mod] of Object.entries(LIB_REGISTRY)) {
    for (const typeName of Object.keys(mod.sensors)) {
      result[typeName] = { roboType: { kind: "named", name: typeName }, library: libId };
    }
  }
  return result;
}

export function listLibraries(): LibModule[] {
  // ListLibraries.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // `LibModule[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = listLibraries();
  return Object.values(LIB_REGISTRY);
}

export function listLibrariesByVendor(vendor: string): LibModule[] {
  // ListLibrariesByVendor.
  //
  // Parameters:
  // - `vendor` — input value
  //
  // Returns:
  // `LibModule[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = listLibrariesByVendor(vendor);
  return listLibraries().filter((l) => l.vendor.toLowerCase() === vendor.toLowerCase());
}

export function readWithDriver(
  driver: SensorDriverDef,
  ctx: DriverContext,
): RuntimeValue {
  // ReadWithDriver.
  //
  // Parameters:
  // - `driver` — input value
  // - `ctx` — input value
  //
  // Returns:
  // `RuntimeValue`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = readWithDriver(driver, ctx);
  return driver.read(ctx);
}
