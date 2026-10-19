/**
 * Converts between ClickHub's shortcut grammar ("Ctrl+Win+Space") and the format
 * `tauri-plugin-global-shortcut` registers keyboard triggers with ("Ctrl+Super+Space"). The Rust
 * side checks the result again before saving.
 */

const TO_PLUGIN: Record<string, string> = { Win: "Super" };
const FROM_PLUGIN: Record<string, string> = {
  super: "Win",
  cmd: "Win",
  command: "Win",
  control: "Ctrl",
  ctrl: "Ctrl",
  option: "Alt",
  alt: "Alt",
  shift: "Shift",
  escape: "Esc",
  arrowup: "Up",
  arrowdown: "Down",
  arrowleft: "Left",
  arrowright: "Right",
};

/** Grammar text to the plugin's format. */
export function toTriggerShortcut(canonical: string): string {
  return canonical
    .split("+")
    .map((part) => TO_PLUGIN[part] ?? part)
    .join("+");
}

/** Plugin format back to the grammar's names, for display. Unknown parts are kept as written. */
export function fromTriggerShortcut(stored: string): string {
  return stored
    .split("+")
    .map((part) => {
      const trimmed = part.trim();
      return FROM_PLUGIN[trimmed.toLowerCase()] ?? trimmed.replace(/^Key([A-Z])$/i, "$1");
    })
    .join("+");
}

/**
 * A keyboard trigger needs a modifier (or a function key), otherwise ClickHub would take the
 * key away from every app while it runs.
 */
export function triggerShortcutProblem(canonical: string): string | null {
  const parts = canonical.split("+");
  const key = parts[parts.length - 1] ?? "";
  if (parts.length > 1 || /^F([1-9]|1[0-9]|2[0-4])$/.test(key)) return null;
  return "Add Ctrl, Shift, Alt or Win (or use a function key), so the key still works in your apps.";
}
