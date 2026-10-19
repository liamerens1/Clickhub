# Architecture

This explains **why** ClickHub is structured the way it is. Read it before changing modules,
threads or windows. If you change the structure, update this file in the same PR.

Status markers: **(now)** exists today; **(Phase N)** planned and arrives in that phase.

## Shape of the app

ClickHub is a **per-user tray app**, not a Windows Service. Services run in Session 0, which has
no access to the user's desktop or input, so a service could neither draw the ring nor see the
mouse. Instead ClickHub runs in the user's session, starts at login through the HKCU Run key
(Phase 4), allows a single instance, and starts hidden to the tray.

It is a Tauri v2 app: a Rust core owns everything that touches Windows, and WebView2 renders the
two UI surfaces (overlay and settings).

## Process and thread model

One process. Its threads and windows:

```
 clickhub.exe
 ├─ Main thread (Tauri / tao event loop)                      (now)
 │   ├─ tray icon + menu                                      (now)
 │   ├─ settings window: WebView2, opaque, created on demand  (now)
 │   └─ overlay window: WebView2, transparent, created once   (Phase 2)
 ├─ Hook thread: SetWindowsHookExW + own message loop         (Phase 1)
 ├─ Core: trigger state machine, profiles, actions            (Phase 1+)
 ├─ Watchdog: checks the hook is alive, re-installs it        (Phase 1)
 └─ WebView2 browser/renderer processes (child processes owned by WebView2)
```

- **Main thread.** Tauri's event loop. It owns all windows and the tray. It never blocks.
- **Hook thread (Phase 1).** Installs `WH_MOUSE_LL` and `WH_KEYBOARD_LL` with the raw
  `SetWindowsHookExW` from the `windows` crate. Low-level hook callbacks run on the thread that
  installed them, and only while that thread pumps messages, so the thread exists to do exactly
  that. We do not use `rdev`: it can only listen, and we must be able to swallow the trigger click.
- **Core (Phase 1+).** Turns hook events into decisions: is this the trigger, is it a hold or a
  tap, is ClickHub paused, is the foreground app full-screen or blocklisted, which ring applies.
  `core/` is pure logic with no Win32 or Tauri, so it is fully unit-testable.
- **Watchdog (Phase 1).** Windows silently removes a low-level hook whose callback exceeds
  `LowLevelHooksTimeout`, without telling the app. The watchdog detects that (heartbeat or probe)
  and re-installs the hook.

## The hot path

From physical click to first ring frame, the budget is **16 ms**:

```
 mouse button down
   → hook callback (hook thread)        match trigger against a precomputed spec; post a message
   → core (receives the message)        decide: open ring at (x, y) for profile P
   → overlay/ (Rust, main thread)       move the overlay window to (x, y); emit `ring:open`
   → overlay frontend (WebView2)        flip a class / set transform + opacity; ring is already built
   → first frame on screen
```

Rules that keep it fast:

- **The hook callback only matches and posts.** No allocation, no I/O, no IPC, no logging, no
  locks held for long. Anything slow risks Windows unhooking us, and every microsecond there is
  added latency for every mouse event system-wide. Swallowing is decided in the callback from
  precomputed state (`return 1` vs `CallNextHookEx`).
- **No IPC round trip on the path.** The Rust side sends one fire-and-forget event; it never waits
  for the frontend to answer before the frame.
- **The ring is pre-rendered.** When the active profile changes, the overlay frontend builds the
  ring DOM ahead of time. The trigger only positions and reveals it.
- **Animate only `transform` and `opacity`.** These run on the compositor without layout or paint.

## Why the overlay is never hidden

The overlay window is created **once** at startup and is **never hidden, never resized and never
focused**. When idle it is fully transparent and click-through. This is deliberate:

- **Showing a hidden WebView2 window is slow and ugly.** WebView2 throttles hidden windows: timers
  and `requestAnimationFrame` slow down and the compositor parks the surface. On show, it can flash
  the last stale frame and then play the open animation throttled. An always-present, transparent
  window is already "warm": the first frame after the trigger is a normal frame.
- **Background throttling is turned off** for the overlay for the same reason.
- **It is ring-sized, not full-screen.** Transparent WebView2 windows use per-pixel alpha
  composition, and its cost scales with the window area. A ring-sized window is cheap; a
  full-screen transparent window across a 4K monitor is not.
- **It moves, never resizes.** Resizing a WebView2 window re-allocates its surface and flickers.
  Moving is just a position change.
- **It never takes focus** (`WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`). If the ring took focus, a "send
  shortcut" action would go to the ring instead of the app the user was working in. It also never
  shows in the taskbar or Alt-Tab.
- **Idle means click-through** (`WS_EX_TRANSPARENT` / ignore-cursor-events), so the invisible
  window never blocks clicks.

### Open question for the Phase 1 spike: mixed DPI

When a per-monitor-DPI-aware window moves onto a monitor with a different scale factor, Windows
resizes it. That breaks "never resize" on mixed-DPI setups. The spike measures two candidates and
records the decision in `docs/roadmap.md`:

1. **One overlay per monitor:** each created once at its monitor's DPI and never moved off it. It
   costs one WebView2 per monitor in memory.
2. **One overlay that handles `WM_DPICHANGED` itself:** keeps its physical size fixed and scales
   the ring in CSS.

If WebView2 cannot hit the smoothness targets at all, the fallback is a native overlay (Direct2D /
Windows Composition) with Tauri kept for settings. That is decided before Phase 2.

## The settings window

- **Opaque**, so it pays none of the transparent-composition cost.
- **Created on demand** the first time it is opened (tray "Open settings", tray left-click, or a
  second launch of the exe). **Closing it hides it**, so reopening is instant and the app keeps
  running. Only tray **Quit** exits.
- It is never on the hot path. It reads and writes config through typed Tauri commands (Phase 3).

## Rust module map (`src-tauri/src/`)

| Module      | Role                                                                                                      | Status       |
| ----------- | --------------------------------------------------------------------------------------------------------- | ------------ |
| `lib.rs`    | Declares modules, re-exports `run`. No logic.                                                             | now          |
| `app/`      | Startup, logging, run loop, shutdown orchestration.                                                       | now          |
| `tray/`     | Tray icon and menu; dispatches menu actions.                                                              | now          |
| `settings/` | Settings window lifecycle (create, reveal, hide-on-close).                                                | now          |
| `config/`   | Config schema, parsing and validation; later load/save/backup.                                            | now (schema) |
| `hook/`     | The only place with Win32 hook code. Hook thread, callback, watchdog.                                     | Phase 1      |
| `core/`     | Trigger matching, profiles, action resolution. Pure logic.                                                | Phase 1      |
| `platform/` | Thin safe wrappers over other Win32: window styles, monitors, DPI, foreground app, full-screen detection. | Phase 1–2    |
| `overlay/`  | Overlay window lifecycle: create once, move, reveal.                                                      | Phase 2      |
| `ipc/`      | Tauri commands and event names. Validates payloads, delegates.                                            | Phase 2–3    |

`unsafe` is denied crate-wide. `hook/` and `platform/` opt back in with `#![allow(unsafe_code)]`.
That keeps every unsafe block in two reviewable places.

## Frontend map (`src/`)

| Folder      | Role                                                             | Status       |
| ----------- | ---------------------------------------------------------------- | ------------ |
| `settings/` | React + TS. Vite entry `settings/index.html`.                    | now          |
| `shared/`   | Zod schemas (mirroring Rust serde types) and typed IPC wrappers. | now (schema) |
| `overlay/`  | Vanilla TS, no framework. Vite entry `overlay/index.html`.       | Phase 2      |

The overlay is vanilla TS on purpose: no framework runtime, no virtual-DOM reconciliation on
pointer moves. Hover is a direct class or transform write on the slice the pointer entered, so it
lands on the same frame. The settings window is a forms-and-lists app where React pays for itself.

## Config

- Stored as JSON at `%APPDATA%\com.liamerens.clickhub\config.json` (Tauri's app config dir). That
  location is what makes the NSIS uninstaller's "Delete the application data" checkbox work.
- Rust (`config/`, serde with `deny_unknown_fields`) is authoritative at runtime. TS (`shared/schema`,
  Zod `.strict()`) mirrors it for the settings UI.
- `fixtures/config/{valid,invalid}` are run through **both** parsers in CI, so the two cannot
  drift. A real example: serde's derived `Deserialize` accepts a JSON array as a struct, and the
  parity test caught it on day one.
- A top-level `version` field allows migrations.
- Invalid config never crashes the app (see `CLAUDE.md` hard rule 9).

## Known platform constraints

- **UIPI:** a non-elevated process does not receive low-level hook events for input aimed at
  elevated windows. The trigger does nothing over an elevated app. We never default to running as
  admin.
- **Anti-cheat:** we never inject into other processes. Full-screen apps auto-disable the trigger
  (Phase 2), with a per-app blocklist (Phase 5).
- **Unsigned releases:** SmartScreen warns until a code-signing certificate exists.
