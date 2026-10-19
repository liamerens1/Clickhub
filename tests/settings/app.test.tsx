// @vitest-environment jsdom
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { type Config, configSchema } from "../../src/shared/schema";

import { App } from "../../src/settings/app";

/** An in-memory stand-in for the Rust commands; `saved` records every save_config call. */
function mockBackend() {
  let config: Config = configSchema.parse({ version: 1 });
  const saved: Config[] = [];
  mockWindows("settings");
  mockIPC(
    (command, payload) => {
      const args = (payload ?? {}) as Record<string, unknown>;
      switch (command) {
        case "get_config":
        case "reload_config":
          return { config, path: "config.json", problem: null };
        case "save_config":
          config = configSchema.parse(args["config"]);
          saved.push(config);
          return { config, path: "config.json", problem: null };
        case "get_status":
          return { paused: false, problem: null };
        case "get_app_info":
          return { version: "0.4.0", installed: false, configPath: "config.json", logDir: null };
        case "get_autostart":
          return { available: false, enabled: false };
        case "get_update_state":
          return { kind: "idle" };
        default:
          return null;
      }
    },
    { shouldMockEvents: true },
  );
  return { saved, current: () => config };
}

let backend: ReturnType<typeof mockBackend>;

beforeEach(() => {
  backend = mockBackend();
});

afterEach(async () => {
  cleanup();
  // Unmounting stops the event listeners asynchronously; let that finish before the mocks go.
  await new Promise((resolve) => setTimeout(resolve, 0));
  clearMocks();
});

const labels = () =>
  Array.from(document.querySelectorAll(".slot-label")).map((label) => label.textContent);

describe("settings App", () => {
  it("shows the sections and the ring's slots", async () => {
    render(<App />);
    expect(screen.getByRole("navigation", { name: "Settings sections" })).toBeTruthy();
    await waitFor(() => {
      expect(labels()).toEqual([
        "Copy",
        "Paste",
        "Screenshot",
        "File Explorer",
        "Browser",
        "Show desktop",
      ]);
    });
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Ring");
  });

  it("saves a label edit once typing pauses, and Undo puts it back", async () => {
    render(<App />);
    await waitFor(() => {
      expect(labels()).toHaveLength(6);
    });
    const firstLabel = document.querySelector(".slot-label");
    if (firstLabel === null) throw new Error("no slot rows");
    fireEvent.click(firstLabel);
    fireEvent.change(screen.getByLabelText("Label"), { target: { value: "Duplicate" } });
    expect(backend.saved).toHaveLength(0);

    await waitFor(() => {
      expect(backend.current().ring.slots[0]?.label).toBe("Duplicate");
    });
    fireEvent.click(screen.getByRole("button", { name: /Undo/ }));
    await waitFor(() => {
      expect(backend.current().ring.slots[0]?.label).toBe("Copy");
    });
    expect(labels()[0]).toBe("Copy");
  });

  it("moves a slot with Alt+Down and saves the new order", async () => {
    render(<App />);
    await waitFor(() => {
      expect(labels()).toHaveLength(6);
    });
    const firstHead = document.querySelector<HTMLElement>(".slot-head");
    if (firstHead === null) throw new Error("no slot rows");
    fireEvent.keyDown(firstHead, { key: "ArrowDown", altKey: true });
    await waitFor(() => {
      expect(
        backend
          .current()
          .ring.slots.map((slot) => slot.label)
          .slice(0, 2),
      ).toEqual(["Paste", "Copy"]);
    });
  });

  it("adds a slot from a preset", async () => {
    render(<App />);
    await waitFor(() => {
      expect(labels()).toHaveLength(6);
    });
    fireEvent.click(screen.getByRole("button", { name: /Add slot/ }));
    const menu = screen.getByRole("menu");
    fireEvent.click(within(menu).getByText("Task Manager"));
    await waitFor(() => {
      expect(backend.current().ring.slots).toHaveLength(7);
    });
    expect(backend.current().ring.slots[6]).toEqual({
      label: "Task Manager",
      action: { kind: "shortcut", keys: "Ctrl+Shift+Esc" },
    });
  });

  it("toggles a trigger modifier on the Trigger page", async () => {
    render(<App />);
    await waitFor(() => {
      expect(labels()).toHaveLength(6);
    });
    fireEvent.click(screen.getByRole("button", { name: "Trigger" }));
    fireEvent.click(screen.getByRole("button", { name: "Ctrl" }));
    await waitFor(() => {
      expect(backend.current().trigger).toEqual({
        kind: "mouse",
        button: "x2",
        modifiers: ["ctrl"],
        swallow: true,
      });
    });
  });
});
