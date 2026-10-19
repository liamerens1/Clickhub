import { describe, expect, it } from "vitest";

import { parseShortcut } from "../../../src/shared/schema";

// Same cases as src-tauri/tests/config/shortcut.rs: both parsers must agree.
function canonical(text: string): string {
  const result = parseShortcut(text);
  if (!result.ok) throw new Error(`${text} should parse: ${result.error}`);
  return result.canonical;
}

describe("parseShortcut", () => {
  it("parses modifiers and keys", () => {
    expect(canonical("Ctrl+Shift+Esc")).toBe("Ctrl+Shift+Esc");
    expect(canonical("a")).toBe("A");
    expect(canonical("7")).toBe("7");
    expect(canonical("F24")).toBe("F24");
    expect(canonical("MediaPlayPause")).toBe("MediaPlayPause");
  });

  it("is case-insensitive and ignores spaces", () => {
    expect(canonical(" win + shift + s ")).toBe("Win+Shift+S");
    expect(canonical("CONTROL+windows+pageup")).toBe("Ctrl+Win+PageUp");
  });

  it("accepts aliases", () => {
    expect(canonical("Escape")).toBe("Esc");
    expect(canonical("Return")).toBe("Enter");
    expect(canonical("Ctrl+Alt+Del")).toBe("Ctrl+Alt+Delete");
  });

  it.each([
    "",
    "+",
    "Ctrl+",
    "Ctrl",
    "Ctrl+Ctrl+A",
    "Hyper+A",
    "F0",
    "F25",
    "F01",
    "Ctrl+AB",
    "Shift+Ctrl",
    "A+B",
  ])("rejects %j", (bad) => {
    expect(parseShortcut(bad).ok).toBe(false);
  });
});
