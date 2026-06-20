/** Planned FFI bridge import paths (orchestration layer — no native linking yet). */
export const FFI_BRIDGE_IMPORTS = new Set([
  "python.torch",
  "python.opencv",
  "python.numpy",
  "python.ros2",
  "cpp.ros2",
  "cpp.pcl",
  "cpp.opencv",
  "cpp.cuda",
]);

export type FfiBridgeKind = "python" | "cpp";

export function ffiBridgeKind(path: string): FfiBridgeKind | null {
  if (path.startsWith("python.")) return "python";
  if (path.startsWith("cpp.")) return "cpp";
  return null;
}

/** Returns true when the import is a registered (planned) FFI bridge namespace. */
export function resolveFfiImport(path: string): boolean {
  if (FFI_BRIDGE_IMPORTS.has(path)) return true;
  const kind = ffiBridgeKind(path);
  if (!kind) return false;
  const suffix = path.slice(kind.length + 1);
  return suffix.length > 0 && /^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*$/.test(suffix);
}
