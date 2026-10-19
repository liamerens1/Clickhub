import type { Slot } from "../shared/schema";

/** Ready-made slots offered when adding a slot or "Use a preset". */
export interface PresetGroup {
  name: string;
  slots: Slot[];
}

const shortcut = (label: string, keys: string): Slot => ({
  label,
  action: { kind: "shortcut", keys },
});
const launch = (label: string, path: string): Slot => ({
  label,
  action: { kind: "launch", path, args: [] },
});

export const PRESET_GROUPS: readonly PresetGroup[] = [
  {
    name: "Editing",
    slots: [
      shortcut("Copy", "Ctrl+C"),
      shortcut("Cut", "Ctrl+X"),
      shortcut("Paste", "Ctrl+V"),
      shortcut("Undo", "Ctrl+Z"),
      shortcut("Redo", "Ctrl+Y"),
      shortcut("Select all", "Ctrl+A"),
      shortcut("Save", "Ctrl+S"),
      shortcut("Find", "Ctrl+F"),
      shortcut("Clipboard history", "Win+V"),
    ],
  },
  {
    name: "Windows",
    slots: [
      shortcut("Screenshot", "Win+Shift+S"),
      shortcut("Show desktop", "Win+D"),
      shortcut("Task view", "Win+Tab"),
      shortcut("Task Manager", "Ctrl+Shift+Esc"),
      shortcut("Lock", "Win+L"),
      shortcut("Close window", "Alt+F4"),
      launch("File Explorer", "explorer.exe"),
      launch("Settings", "ms-settings:"),
    ],
  },
  {
    name: "Browser",
    slots: [
      shortcut("New tab", "Ctrl+T"),
      shortcut("Close tab", "Ctrl+W"),
      shortcut("Reopen tab", "Ctrl+Shift+T"),
      shortcut("Refresh", "F5"),
      { label: "Browser", action: { kind: "url", url: "https://www.google.com" } },
    ],
  },
  {
    name: "Media",
    slots: [
      shortcut("Play / pause", "MediaPlayPause"),
      shortcut("Next track", "MediaNext"),
      shortcut("Previous track", "MediaPrevious"),
      shortcut("Volume up", "VolumeUp"),
      shortcut("Volume down", "VolumeDown"),
      shortcut("Mute", "VolumeMute"),
    ],
  },
  {
    name: "Apps",
    slots: [
      launch("Calculator", "calc.exe"),
      launch("Notepad", "notepad.exe"),
      launch("Terminal", "wt.exe"),
    ],
  },
];
