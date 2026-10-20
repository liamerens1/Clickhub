# ClickHub

A radial "control center" for Windows. Press a trigger you choose and a ring of actions appears
at the cursor. Aim at one and release (or click) to fire it.

- **Any trigger:** right, middle, Back or Forward mouse button, held or tapped, optionally with
  Ctrl/Shift/Alt/Win, or a keyboard chord.
- **Actions:** send a shortcut, launch an app or file, open a URL. More arrive in later phases.
- **Works with any mouse**, 2 to 12 slots, follows Windows light/dark.
- **Stays out of the way:** never takes focus, turns itself off in full-screen apps and games,
  and lets your clicks through if anything goes wrong.

> **Status:** early development (Phase 3 of 7, settings window). Expect rough edges.

## Download

**[Download the latest release](https://github.com/liamerens1/Clickhub/releases/latest)**

Run `ClickHub_<version>_x64-setup.exe`. It installs for your user only, with no admin prompt.

Only pre-releases exist so far, and GitHub's "latest" link skips them. If that link shows no
release, pick the newest one on the [releases page](https://github.com/liamerens1/Clickhub/releases).

Builds are not code-signed yet, so Windows SmartScreen warns on first run: choose **More info**,
then **Run anyway**.

## Using it

ClickHub lives in the system tray. The default trigger is the **Forward** mouse button.

- **Tap** the trigger: the ring stays open. Click an item to fire it; click the centre,
  right-click or press Esc to close.
- **Hold** the trigger: aim and release to fire. Releasing in the centre cancels.

Click the tray icon to open settings, or right-click it for **Pause**, **Reload config** and
**Quit**. In settings you can
change the trigger, add, edit and reorder slots, and every change applies right away.

## Configuration

Settings are saved to `%APPDATA%\com.liamerens.clickhub\config.json`. You can also edit it by
hand, then choose **Reload config** from the tray menu.

```json
{
  "version": 1,
  "trigger": { "kind": "mouse", "button": "x2", "modifiers": [], "swallow": true },
  "ring": {
    "slots": [
      { "label": "Copy", "action": { "kind": "shortcut", "keys": "Ctrl+C" } },
      { "label": "Notes", "action": { "kind": "launch", "path": "C:/Windows/notepad.exe" } },
      { "label": "Docs", "action": { "kind": "url", "url": "https://example.com" } }
    ]
  }
}
```

- `button`: `"right"`, `"middle"`, `"x1"` (Back) or `"x2"` (Forward). The left button is not
  allowed.
- `modifiers`: any of `"ctrl"`, `"shift"`, `"alt"`, `"win"`. All of them must be held, and no
  others.
- `swallow`: `true` hides the trigger click from the app underneath.
- Keyboard trigger instead: `{ "kind": "keyboard", "shortcut": "Ctrl+Alt+Space" }`.

If the file is invalid, ClickHub keeps running on the last good config, saves a copy of the broken
file next to it, and tells you what is wrong. Logs are in
`%LOCALAPPDATA%\com.liamerens.clickhub\logs`.

## Known limitations

- **Admin windows:** Windows does not let a normal app see mouse input aimed at apps running as
  administrator, so the trigger does nothing over them. ClickHub never runs as admin by default.
- **Unsigned builds:** see [Download](#download).

## Roadmap

| Phase               | Goal                                                         | Status      |
| ------------------- | ------------------------------------------------------------ | ----------- |
| 0 — Foundation      | Repo, rules, CI, empty tray app                              | Done        |
| 1 — Trigger         | Low-level hook thread, configurable button                   | Done        |
| 2 — Ring            | Ring at the cursor, static actions, full-screen auto-disable | Done        |
| 3 — Settings UI     | Ring editor, trigger picker, saving                          | In progress |
| 4 — Lifecycle       | Installer, start on login, uninstall, updater                | Not started |
| 5 — Profiles        | Per-app rings, nested sub-rings, per-app blocklist           | Not started |
| 6 — Actions & modes | More actions, marking/turbo modes, themes, two-tier branches | Not started |
| 7 — v1              | Hardening and release                                        | Not started |

**Planned for Phase 6: two-tier branching.** Hovering an item fans out a second tier of
branches beyond it, so you can aim straight through to a sub-action without opening a new ring.
It is part of the theme work, offered as a ring style alongside the other presets.

## Building from source

Requirements:

- Windows 10 or 11
- [Rust](https://rustup.rs/) (the version is pinned in `rust-toolchain.toml` and installed
  automatically)
- Node 24 LTS and pnpm 11 (`npm install -g pnpm@11.28.6`)
- Visual Studio Build Tools with "Desktop development with C++"
- WebView2 runtime (preinstalled on Windows 11)

From the repo root in Command Prompt:

```
pnpm install
pnpm tauri dev
```

Checks:

```
pnpm check
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Release build and installer:

```
pnpm tauri build
```
