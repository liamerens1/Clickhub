import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Tests run in Node by default. Component tests opt into jsdom with a
// `// @vitest-environment jsdom` docblock at the top of the file.
export default defineConfig({
  plugins: [react()],
  test: {
    root: import.meta.dirname,
    include: ["tests/**/*.test.{ts,tsx}"],
    environment: "node",
  },
});
