import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

import { type Config, configSchema } from "../schema";

/**
 * Development only: a stand-in for the Rust commands so the settings page can be opened in a
 * plain browser (`pnpm dev`, then http://localhost:1420/settings/). Never part of a build:
 * `settings/main.tsx` imports it dynamically behind `import.meta.env.DEV`.
 *
 * Query options: `?problem` shows the broken-config banner, `?fail` makes every save fail.
 */
export function installDevMock(): void {
  const params = new URLSearchParams(window.location.search);
  let config: Config = configSchema.parse({ version: 1 });
  let paused = false;
  const problem = params.has("problem")
    ? {
        error: "config is not valid: unknown field `slot`, expected `slots` at line 4 column 11",
        backup:
          "C:\\Users\\you\\AppData\\Roaming\\com.liamerens.clickhub\\config.invalid-1760000000.json",
        fallback: "lastGood",
      }
    : null;
  const path = "C:\\Users\\you\\AppData\\Roaming\\com.liamerens.clickhub\\config.json";

  mockWindows("settings");
  mockIPC(
    async (command, payload) => {
      await new Promise((resolve) => setTimeout(resolve, 25));
      const args = (payload ?? {}) as Record<string, unknown>;
      switch (command) {
        case "get_config":
        case "reload_config":
          return { config, path, problem };
        case "save_config": {
          if (params.has("fail")) throw new Error("the disk is full (dev mock)");
          const result = configSchema.safeParse(args["config"]);
          if (!result.success) throw new Error(result.error.message);
          config = result.data;
          return { config, path, problem: null };
        }
        case "get_status":
        case "set_paused":
          if (command === "set_paused") paused = args["paused"] === true;
          return { paused, problem: null };
        case "set_trigger_capture":
        case "open_folder":
          return null;
        case "pick_file":
          return "C:\\Windows\\System32\\notepad.exe";
        case "get_app_info":
          return {
            version: "0.3.0",
            configPath: path,
            logDir: "C:\\Users\\you\\AppData\\Local\\com.liamerens.clickhub\\logs",
          };
        default:
          throw new Error(`dev mock: unknown command ${command}`);
      }
    },
    { shouldMockEvents: true },
  );
}
