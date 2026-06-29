/**
 * Built-in library sensor type catalog for compile-time unit checking.
 * @module
 */

export type LibrarySensorEntry = {
  roboType: { kind: "named"; name: string };
  library: string;
};

const BUILTIN_LIBRARY_SENSOR_TYPES: Record<string, LibrarySensorEntry> = {
  VelodyneVLP16: { roboType: { kind: "named", name: "VelodyneVLP16" }, library: "velodyne.vlp16" },
  VelodyneVLP32: { roboType: { kind: "named", name: "VelodyneVLP32" }, library: "velodyne.vlp32" },
  HokuyoUST10: { roboType: { kind: "named", name: "HokuyoUST10" }, library: "hokuyo.ust10" },
  HokuyoUTM30: { roboType: { kind: "named", name: "HokuyoUTM30" }, library: "hokuyo.utm30" },
  BoschBNO055: { roboType: { kind: "named", name: "BoschBNO055" }, library: "bosch.bno055" },
  BoschBMP388: { roboType: { kind: "named", name: "BoschBMP388" }, library: "bosch.bmp388" },
  BoschBME280: { roboType: { kind: "named", name: "BoschBME280" }, library: "bosch.bme280" },
  AdafruitBH1750: { roboType: { kind: "named", name: "AdafruitBH1750" }, library: "adafruit.bh1750" },
  IntelRealSenseD435: { roboType: { kind: "named", name: "IntelRealSenseD435" }, library: "intel.realsense" },
  IntelRealSenseD455: { roboType: { kind: "named", name: "IntelRealSenseD455" }, library: "intel.realsense" },
  YdlidarX4: { roboType: { kind: "named", name: "YdlidarX4" }, library: "ydlidar.x4" },
  YdlidarG4: { roboType: { kind: "named", name: "YdlidarG4" }, library: "ydlidar.g4" },
  AdafruitVL53L0X: { roboType: { kind: "named", name: "AdafruitVL53L0X" }, library: "adafruit.vl53l0x" },
  SparkfunLSM9DS1: { roboType: { kind: "named", name: "SparkfunLSM9DS1" }, library: "sparkfun.lsm9ds1" },
  WaveshareUWMF: { roboType: { kind: "named", name: "WaveshareUWMF" }, library: "waveshare.uwmf" },
  OusterOS1: { roboType: { kind: "named", name: "OusterOS1" }, library: "ouster.os1" },
};

export function allLibrarySensorTypes(): Record<string, LibrarySensorEntry> {
  // Return the built-in sensor type catalog mirrored from lib/registry.
  return { ...BUILTIN_LIBRARY_SENSOR_TYPES };
}
