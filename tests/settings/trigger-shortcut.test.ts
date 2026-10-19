import { describe, expect, it } from "vitest";

import {
  fromTriggerShortcut,
  toTriggerShortcut,
  triggerShortcutProblem,
} from "../../src/settings/trigger-shortcut";

describe("trigger shortcut format", () => {
  it("writes Win as Super for the global-shortcut plugin", () => {
    expect(toTriggerShortcut("Ctrl+Win+Space")).toBe("Ctrl+Super+Space");
    expect(toTriggerShortcut("Ctrl+Alt+Space")).toBe("Ctrl+Alt+Space");
  });

  it("reads plugin names back as the grammar's names", () => {
    expect(fromTriggerShortcut("Ctrl+Super+Space")).toBe("Ctrl+Win+Space");
    expect(fromTriggerShortcut("control+shift+KeyK")).toBe("Ctrl+Shift+K");
    expect(fromTriggerShortcut("CommandOrControl+Q")).toBe("CommandOrControl+Q");
  });

  it("round-trips recorded shortcuts", () => {
    for (const shortcut of ["Ctrl+Win+F", "Alt+Shift+Up", "Ctrl+Esc"]) {
      expect(fromTriggerShortcut(toTriggerShortcut(shortcut))).toBe(shortcut);
    }
  });
});

describe("triggerShortcutProblem", () => {
  it("accepts a chord with a modifier or a lone function key", () => {
    expect(triggerShortcutProblem("Ctrl+Alt+Space")).toBeNull();
    expect(triggerShortcutProblem("F13")).toBeNull();
  });

  it("refuses a lone ordinary key, which would stop it typing everywhere", () => {
    expect(triggerShortcutProblem("Space")).not.toBeNull();
    expect(triggerShortcutProblem("A")).not.toBeNull();
  });
});
