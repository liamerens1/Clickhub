import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

import { type Config, configSchema } from "../schema";

/**
 * Development only: a stand-in for the Rust commands so the settings page can be opened in a
 * plain browser (`pnpm dev`, then http://localhost:1420/settings/). Never part of a build:
 * `settings/main.tsx` imports it dynamically behind `import.meta.env.DEV`.
 *
 * Query options: `?problem` shows the broken-config banner, `?fail` makes every save fail,
 * `?installed` acts as an installed copy, `?update` makes Check now find version 9.9.9.
 */
export function installDevMock(): void {
  const params = new URLSearchParams(window.location.search);
  let config: Config = configSchema.parse({ version: 1 });
  let paused = false;
  let autostart = params.has("installed");
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
        case "install_update":
        case "uninstall":
          return null;
        case "get_autostart":
        case "set_autostart":
          if (command === "set_autostart") autostart = args["enabled"] === true;
          return { available: params.has("installed"), enabled: autostart };
        case "get_update_state":
          return { kind: "idle" };
        case "check_for_updates":
          return params.has("update")
            ? { kind: "available", version: "9.9.9", notes: null }
            : { kind: "upToDate", checkedAtMs: Date.now() };
        case "pick_file":
          return "C:\\Windows\\System32\\notepad.exe";
        case "get_app_info":
          return {
            version: "0.4.0",
            installed: params.has("installed"),
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
