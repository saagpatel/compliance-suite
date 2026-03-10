import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  cacheDir: process.env.VITE_CACHE_DIR || "node_modules/.vite",
  clearScreen: false,
  server: {
    strictPort: true,
    port: 5174
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@packages/types": path.resolve(__dirname, "../../packages/types/src")
    }
  }
});
