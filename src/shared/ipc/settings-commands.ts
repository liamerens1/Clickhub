import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { z } from "zod";

import { type Config, configSchema } from "../schema";

/**
 * Typed wrappers over the Rust commands in `src-tauri/src/ipc/` for the settings window. Every
 * reply is validated before use, and failures come back as values, never as exceptions.
 */

export const configSnapshotSchema = z.object({
  config: configSchema,
  path: z.string(),
  problem: z
    .object({
      error: z.string(),
      backup: z.string().nullable(),
      fallback: z.enum(["lastGood", "defaults"]),
    })
    .nullable(),
});

export const appStatusSchema = z.object({
  paused: z.boolean(),
  problem: z.string().nullable(),
});

export const appInfoSchema = z.object({
  version: z.string(),
  /** Installed with the installer (so it can start with Windows, update and uninstall). */
  installed: z.boolean(),
  configPath: z.string(),
  logDir: z.string().nullable(),
});

export const autostartSchema = z.object({ available: z.boolean(), enabled: z.boolean() });

/** Mirrors `UpdateState` in `src-tauri/src/lifecycle/updates.rs`. */
export const updateStateSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("idle") }),
  z.object({ kind: z.literal("checking") }),
  z.object({ kind: z.literal("upToDate"), checkedAtMs: z.number() }),
  z.object({ kind: z.literal("available"), version: z.string(), notes: z.string().nullable() }),
  z.object({
    kind: z.literal("downloading"),
    downloaded: z.number(),
    total: z.number().nullable(),
  }),
  z.object({ kind: z.literal("installing") }),
  z.object({ kind: z.literal("failed"), error: z.string() }),
]);

export type Autostart = z.infer<typeof autostartSchema>;
export type UpdateState = z.infer<typeof updateStateSchema>;
export type ConfigSnapshot = z.infer<typeof configSnapshotSchema>;
export type AppStatus = z.infer<typeof appStatusSchema>;
export type AppInfo = z.infer<typeof appInfoSchema>;
export type Folder = "config" | "logs";
export type IpcResult<T> = { ok: true; value: T } | { ok: false; error: string };

async function call<T>(
  command: string,
  schema: z.ZodType<T>,
  args?: Record<string, unknown>,
): Promise<IpcResult<T>> {
  let reply: unknown;
  try {
    reply = await invoke<unknown>(command, args);
  } catch (error) {
    return { ok: false, error: typeof error === "string" ? error : String(error) };
  }
  const result = schema.safeParse(reply);
  if (!result.success) {
    return { ok: false, error: `unexpected reply from ${command}: ${result.error.message}` };
  }
  return { ok: true, value: result.data };
}

export const getConfig = () => call("get_config", configSnapshotSchema);
export const saveConfig = (config: Config) => call("save_config", configSnapshotSchema, { config });
export const reloadConfig = () => call("reload_config", configSnapshotSchema);
export const getStatus = () => call("get_status", appStatusSchema);
export const setPaused = (paused: boolean) => call("set_paused", appStatusSchema, { paused });
/** Switches the trigger off while the window waits for "press the button you want". */
export const setTriggerCapture = (capturing: boolean) =>
  call("set_trigger_capture", z.null(), { capturing });
/** Opens the Windows file dialog; `null` when the user cancelled. */
export const pickFile = () => call("pick_file", z.string().nullable());
export const openFolder = (folder: Folder) => call("open_folder", z.null(), { folder });
export const getAppInfo = () => call("get_app_info", appInfoSchema);
export const getAutostart = () => call("get_autostart", autostartSchema);
export const setAutostart = (enabled: boolean) =>
  call("set_autostart", autostartSchema, { enabled });
export const getUpdateState = () => call("get_update_state", updateStateSchema);
export const checkForUpdates = () => call("check_for_updates", updateStateSchema);
/** On success ClickHub exits and the installer restarts it, so this may never resolve. */
export const installUpdate = () => call("install_update", z.null());
/** Starts the uninstaller and quits ClickHub. Ask the user first. */
export const uninstall = () => call("uninstall", z.null());

/** The running config changed (saved here, or reloaded from the tray). */
export function onConfigChanged(handler: () => void): Promise<UnlistenFn> {
  return getCurrentWebviewWindow().listen<unknown>("config:changed", () => {
    handler();
  });
}

/** Update checking or installing moved on. */
export function onUpdateChanged(handler: (state: UpdateState) => void): Promise<UnlistenFn> {
  return getCurrentWebviewWindow().listen<unknown>("update:changed", ({ payload }) => {
    const result = updateStateSchema.safeParse(payload);
    if (result.success) {
      handler(result.data);
    } else {
      console.error("ClickHub settings: ignoring malformed update:changed", result.error.issues);
    }
  });
}

/** Pause state or the tray problem changed. */
export function onStatusChanged(handler: (status: AppStatus) => void): Promise<UnlistenFn> {
  return getCurrentWebviewWindow().listen<unknown>("status:changed", ({ payload }) => {
    const result = appStatusSchema.safeParse(payload);
    if (result.success) {
      handler(result.data);
    } else {
      console.error("ClickHub settings: ignoring malformed status:changed", result.error.issues);
    }
  });
}
