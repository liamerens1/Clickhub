import { resolve } from "node:path";

import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const repoRoot = import.meta.dirname;
const srcRoot = resolve(repoRoot, "src");

// One HTML entry per Tauri window.
const entries = {
  settings: resolve(srcRoot, "settings/index.html"),
  overlay: resolve(srcRoot, "overlay/index.html"),
};

export default defineConfig({
  root: srcRoot,
  plugins: [react()],
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_ENV_"],
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: resolve(repoRoot, "dist"),
    emptyOutDir: true,
    // WebView2 is evergreen Chromium; no need to transpile for older engines.
    target: "chrome130",
    rollupOptions: {
      input: entries,
    },
  },
});
