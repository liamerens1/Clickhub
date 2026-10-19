import { z } from "zod";

/**
 * Keyboard chord such as "Ctrl+Shift+Esc". Mirrors `src-tauri/src/config/shortcut.rs`: zero or
 * more distinct modifiers, then exactly one key; case-insensitive; spaces around parts ignored.
 */

const MODIFIERS: Record<string, string> = {
  ctrl: "Ctrl",
  control: "Ctrl",
  shift: "Shift",
  alt: "Alt",
  win: "Win",
  windows: "Win",
};

const NAMED_KEYS = [
  "Esc",
  "Tab",
  "Enter",
  "Space",
  "Backspace",
  "Delete",
  "Insert",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  "Up",
  "Down",
  "Left",
  "Right",
  "PrintScreen",
  "VolumeUp",
  "VolumeDown",
  "VolumeMute",
  "MediaPlayPause",
  "MediaNext",
  "MediaPrevious",
  "MediaStop",
] as const;

const ALIASES: Record<string, string> = { escape: "Esc", return: "Enter", del: "Delete" };

export type ParseShortcutResult = { ok: true; canonical: string } | { ok: false; error: string };

function parseKey(token: string): string | null {
  if (/^[a-z]$/i.test(token)) return token.toUpperCase();
  if (/^[0-9]$/.test(token)) return token;
  const fn = /^f([1-9][0-9]?)$/i.exec(token);
  if (fn?.[1] !== undefined) {
    const number = Number(fn[1]);
    return number <= 24 ? `F${String(number)}` : null;
  }
  const canonical = ALIASES[token.toLowerCase()] ?? token;
  return NAMED_KEYS.find((name) => name.toLowerCase() === canonical.toLowerCase()) ?? null;
}

/** Parses a shortcut into its canonical form ("Ctrl+Shift+Esc"). Never throws. */
export function parseShortcut(text: string): ParseShortcutResult {
  const tokens = text.split("+").map((token) => token.trim());
  if (tokens.some((token) => token === "")) {
    return { ok: false, error: `shortcut "${text}" has an empty part` };
  }
  const last = tokens[tokens.length - 1] ?? "";
  const modifiers: string[] = [];
  for (const token of tokens.slice(0, -1)) {
    const modifier = MODIFIERS[token.toLowerCase()];
    if (modifier === undefined) {
      return { ok: false, error: `"${token}" in shortcut "${text}" is not a modifier` };
    }
    if (modifiers.includes(modifier)) {
      return { ok: false, error: `shortcut "${text}" repeats "${token}"` };
    }
    modifiers.push(modifier);
  }
  if (MODIFIERS[last.toLowerCase()] !== undefined) {
    return { ok: false, error: `shortcut "${text}" must end with a key, not a modifier` };
  }
  const key = parseKey(last);
  if (key === null) return { ok: false, error: `"${last}" is not a supported key` };
  return { ok: true, canonical: [...modifiers, key].join("+") };
}

export const shortcutSchema = z.string().transform((text, context) => {
  const result = parseShortcut(text);
  if (!result.ok) {
    context.addIssue({ code: "custom", message: result.error });
    return z.NEVER;
  }
  return result.canonical;
});
