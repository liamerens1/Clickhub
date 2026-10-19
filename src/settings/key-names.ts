/**
 * Maps a DOM `KeyboardEvent.code` to the key names of ClickHub's shortcut grammar
 * (`src/shared/schema/shortcut.ts`). `code` names the physical key, so the result does not
 * depend on Shift or the keyboard layout's characters.
 */

const NAMED: Record<string, string> = {
  Escape: "Esc",
  Tab: "Tab",
  Enter: "Enter",
  NumpadEnter: "Enter",
  Space: "Space",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  PrintScreen: "PrintScreen",
  AudioVolumeUp: "VolumeUp",
  VolumeUp: "VolumeUp",
  AudioVolumeDown: "VolumeDown",
  VolumeDown: "VolumeDown",
  AudioVolumeMute: "VolumeMute",
  VolumeMute: "VolumeMute",
  MediaPlayPause: "MediaPlayPause",
  MediaTrackNext: "MediaNext",
  MediaTrackPrevious: "MediaPrevious",
  MediaStop: "MediaStop",
};

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "ShiftLeft",
  "ShiftRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
  "OSLeft",
  "OSRight",
]);

/** True for Ctrl, Shift, Alt and Win keys, which never end a shortcut. */
export function isModifierCode(code: string): boolean {
  return MODIFIER_CODES.has(code);
}

/** The grammar's name for the key, or `null` if ClickHub cannot send it. */
export function keyFromCode(code: string): string | null {
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter?.[1] !== undefined) return letter[1];
  const digit = /^(?:Digit|Numpad)([0-9])$/.exec(code);
  if (digit?.[1] !== undefined) return digit[1];
  const fn = /^F([1-9][0-9]?)$/.exec(code);
  if (fn?.[1] !== undefined && Number(fn[1]) <= 24) return `F${fn[1]}`;
  return NAMED[code] ?? null;
}

export interface HeldModifiers {
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  win: boolean;
}

/** Joins held modifiers and a key in the grammar's canonical order: "Ctrl+Shift+Alt+Win+K". */
export function chordText(held: HeldModifiers, key: string): string {
  const parts: string[] = [];
  if (held.ctrl) parts.push("Ctrl");
  if (held.shift) parts.push("Shift");
  if (held.alt) parts.push("Alt");
  if (held.win) parts.push("Win");
  parts.push(key);
  return parts.join("+");
}
