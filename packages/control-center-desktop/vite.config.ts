/**
 * Vite config for Control Center desktop shell.
 * @module
 */

import react from "@vitejs/plugin-react";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const packageRoot = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  server: { port: 5174, strictPort: true },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      "@davalgi-spanda/web": path.resolve(packageRoot, "../web/src"),
    },
  },
});
