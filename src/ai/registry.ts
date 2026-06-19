export type AiLibModule = {
  id: string;
  vendor: string;
  name: string;
  version: string;
  description: string;
  runtime: "onnx" | "tflite" | "tensorrt";
};

export const AI_LIB_REGISTRY: Record<string, AiLibModule> = {
  "onnx.runtime": {
    id: "onnx.runtime",
    vendor: "ONNX",
    name: "runtime",
    version: "1.0.0",
    description: "ONNX Runtime inference backend",
    runtime: "onIPPAPI",
  },
  "tflite.runtime": {
    id: "tflite.runtime",
    vendor: "TensorFlow",
    name: "runtime",
    version: "1.0.0",
    description: "TensorFlow Lite inference backend",
    runtime: "tflite",
  },
  "tensorrt.runtime": {
    id: "tensorrt.runtime",
    vendor: "NVIDIA",
    name: "runtime",
    version: "1.0.0",
    description: "TensorRT inference backend for Jetson",
    runtime: "tensorrt",
  },
};

export function resolveAiImport(path: string): AiLibModule | undefined {
  return AI_LIB_REGISTRY[path];
}

export function listAiLibraries(): AiLibModule[] {
  return Object.values(AI_LIB_REGISTRY);
}
