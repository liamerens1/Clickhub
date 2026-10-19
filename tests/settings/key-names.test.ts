import { describe, expect, it } from "vitest";

import { parseShortcut } from "../../src/shared/schema";

import { chordText, isModifierCode, keyFromCode } from "../../src/settings/key-names";

describe("keyFromCode", () => {
  it("maps letters, digits and function keys by physical key", () => {
    expect(keyFromCode("KeyS")).toBe("S");
    expect(keyFromCode("Digit7")).toBe("7");
    expect(keyFromCode("Numpad3")).toBe("3");
    expect(keyFromCode("F12")).toBe("F12");
    expect(keyFromCode("F24")).toBe("F24");
  });

  it("maps named keys to the grammar's names", () => {
    expect(keyFromCode("Escape")).toBe("Esc");
    expect(keyFromCode("ArrowLeft")).toBe("Left");
    expect(keyFromCode("NumpadEnter")).toBe("Enter");
    expect(keyFromCode("AudioVolumeUp")).toBe("VolumeUp");
    expect(keyFromCode("MediaTrackNext")).toBe("MediaNext");
  });

  it("returns null for keys ClickHub cannot send", () => {
    expect(keyFromCode("F25")).toBeNull();
    expect(keyFromCode("Semicolon")).toBeNull();
    expect(keyFromCode("CapsLock")).toBeNull();
  });

  it("produces only keys the shortcut grammar accepts", () => {
    const codes = ["KeyA", "Digit0", "F1", "Escape", "Space", "PrintScreen", "MediaStop"];
    for (const code of codes) {
      const key = keyFromCode(code) ?? "";
      expect(parseShortcut(`Ctrl+${key}`).ok, code).toBe(true);
    }
  });
});

describe("isModifierCode", () => {
  it("recognises both sides of every modifier", () => {
    for (const code of ["ControlLeft", "ShiftRight", "AltLeft", "MetaRight"]) {
      expect(isModifierCode(code)).toBe(true);
    }
    expect(isModifierCode("KeyA")).toBe(false);
  });
});

describe("chordText", () => {
  it("joins modifiers in canonical order before the key", () => {
    const held = { ctrl: true, shift: true, alt: false, win: true };
    expect(chordText(held, "S")).toBe("Ctrl+Shift+Win+S");
    expect(chordText({ ctrl: false, shift: false, alt: false, win: false }, "F5")).toBe("F5");
  });
});
