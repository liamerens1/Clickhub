// @vitest-environment jsdom
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";

import { getConfig, pickFile, saveConfig } from "../../../src/shared/ipc";
import { configSchema } from "../../../src/shared/schema";

afterEach(clearMocks);

const config = configSchema.parse({ version: 1 });

describe("settings commands", () => {
  it("returns a validated snapshot", async () => {
    mockIPC(() => ({ config, path: "config.json", problem: null }));
    const result = await getConfig();
    expect(result).toEqual({ ok: true, value: { config, path: "config.json", problem: null } });
  });

  it("turns a malformed reply into an error value", async () => {
    mockIPC(() => ({ config: { version: 2 }, path: 1 }));
    const result = await getConfig();
    expect(result.ok).toBe(false);
  });

  it("turns a rejected command into an error value with Rust's message", async () => {
    mockIPC(() => {
      throw new Error("the ring needs 2 to 12 slots, found 1");
    });
    const result = await saveConfig(config);
    expect(result).toEqual({
      ok: false,
      error: expect.stringContaining("the ring needs 2 to 12 slots") as unknown,
    });
  });

  it("sends the config under the argument name Rust expects", async () => {
    let sent: unknown;
    mockIPC((command, payload) => {
      sent = { command, payload };
      return { config, path: "config.json", problem: null };
    });
    await saveConfig(config);
    expect(sent).toEqual({ command: "save_config", payload: { config } });
  });

  it("reports a cancelled file dialog as null", async () => {
    mockIPC(() => null);
    expect(await pickFile()).toEqual({ ok: true, value: null });
  });
});
