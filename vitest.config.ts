import path from "node:path";
import react from "@vitejs/plugin-react";
import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  test: {
    environment: "jsdom",
    exclude: [
      ...configDefaults.exclude,
      ".worktrees/**",
      "scripts/asr-eval.test.mjs",
    ],
    setupFiles: ["./src/test/setup.ts"],
    css: true,
    restoreMocks: true,
  },
});
