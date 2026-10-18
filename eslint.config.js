import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import reactHooks from "eslint-plugin-react-hooks";
import simpleImportSort from "eslint-plugin-simple-import-sort";
import tseslint from "typescript-eslint";

// Module boundaries (see docs/coding-guidelines.md). Each block lists what a
// folder may never import.
const nodeBuiltins = {
  group: ["node:*", "fs", "path", "child_process", "os"],
  message: "Frontend code runs in WebView2. Node APIs are only for tests and tooling.",
};
const rawInvoke = {
  name: "@tauri-apps/api/core",
  message: "Call the backend through the typed wrappers in src/shared/ipc.",
};

export default defineConfig([
  globalIgnores(["dist/", "coverage/", "src-tauri/", "docs/memory/"]),

  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ["**/*.js"],
    extends: [tseslint.configs.disableTypeChecked],
  },

  {
    plugins: { "simple-import-sort": simpleImportSort },
    rules: {
      "simple-import-sort/imports": [
        "error",
        {
          // 1. side effects (CSS)  2. Node built-ins  3. packages  4. src/shared  5. relative
          groups: [["^\u0000"], ["^node:"], ["^@?\\w"], ["^\\.\\./.*shared"], ["^\\."]],
        },
      ],
      "simple-import-sort/exports": "error",
      "@typescript-eslint/consistent-type-imports": "error",
      "@typescript-eslint/no-explicit-any": "error",
    },
  },

  {
    files: ["src/**/*.{ts,tsx}", "tests/**/*.tsx"],
    extends: [reactHooks.configs.flat["recommended-latest"]],
  },

  {
    files: ["src/settings/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [rawInvoke],
          patterns: [
            nodeBuiltins,
            { group: ["**/overlay/**"], message: "settings/ never imports from overlay/." },
          ],
        },
      ],
    },
  },
  {
    files: ["src/overlay/**/*.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [rawInvoke],
          patterns: [
            nodeBuiltins,
            { group: ["react", "react-dom", "react/*"], message: "The overlay is vanilla TS." },
            { group: ["**/settings/**"], message: "overlay/ never imports from settings/." },
          ],
        },
      ],
    },
  },
  {
    files: ["src/shared/**/*.ts"],
    ignores: ["src/shared/ipc/**"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            nodeBuiltins,
            { group: ["react", "react-dom", "react/*"], message: "shared/ is framework-free." },
            { group: ["@tauri-apps/*"], message: "Only shared/ipc talks to Tauri." },
            {
              group: ["**/overlay/**", "**/settings/**"],
              message: "shared/ never imports a window.",
            },
          ],
        },
      ],
    },
  },
]);
