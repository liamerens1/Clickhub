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
 ├─ clickhub-hook: SetWindowsHookExW + own message loop       (now)
 ├─ clickhub-pipeline: consumes hook events ("core")          (now)
 ├─ clickhub-watchdog: re-installs dropped hooks              (now)
 └─ WebView2 browser/renderer processes (child processes owned by WebView2)
```

- **Main thread.** Tauri's event loop. It owns all windows and the tray. It never blocks.
- **Hook thread** (`hook/thread.rs`). Installs `WH_MOUSE_LL` and `WH_KEYBOARD_LL` with the raw
  `SetWindowsHookExW` from the `windows` crate. Low-level hook callbacks run on the thread that
  installed them, and only while that thread pumps messages, so the thread exists to do exactly
  that. Other threads talk to it with `PostThreadMessageW` (re-install, stop). We do not use
  `rdev`: it can only listen, and we must be able to swallow the trigger click.
- **Pipeline** (`app/pipeline.rs`, the "core" consumer). Receives hook events, uses the pure
  logic in `core/` (press tracking, latency stats) and decides what happens. Phase 1 logs taps,
  holds and latency; Phase 2 opens the ring from here. It checks in (a heartbeat) at least every
  100 ms, even when idle.
- **Watchdog** (`hook/watchdog.rs`, policy in `core/watchdog_policy.rs`). See below.

### How the callback talks to the rest of the app

The callback is an `extern "system"` function with no user-data pointer, so its state is a
static (`hook::SHARED`) made only of atomics: the trigger filter (button + modifier bitmask +
swallow flag), the pause flag, the pipeline heartbeat, last-event ticks, and a ring buffer of
callback durations. Events go out through a bounded `std::sync::mpsc::sync_channel`, whose buffer
is preallocated, so `try_send` never allocates or blocks. If the channel is full the event is
dropped and counted, never waited on.

### Fail open

The callback swallows the trigger click only when **all** of these hold: swallowing is enabled,
ClickHub is not paused, the button and the exact modifier set match, and the pipeline heartbeat is
less than 500 ms old. A hung or crashed pipeline therefore stops swallowing within half a second,
and the click reaches the app underneath. An `Up` is swallowed exactly when its `Down` was, so no
app ever sees a lone half of a click. Injected (synthetic) input is never matched.

### Watchdog: detecting a silently removed hook

Windows silently removes a low-level hook whose callback exceeds `LowLevelHooksTimeout`, without
telling the app; the only symptom is that events stop arriving. The watchdog polls every 200 ms
and compares `GetLastInputInfo` (the newest input Windows saw in the session, from any device) with
the newest event either hook saw. If Windows saw input the hooks missed by more than 250 ms on
**two consecutive polls**, it asks the hook thread to re-install, so a real loss is repaired within
about 0.7 s. The second poll filters out a callback that merely ran late under heavy CPU load and
then caught up, which a one-poll check mistook for a loss during the first live run (while a Rust
build saturated the CPU). The hook thread also runs at `THREAD_PRIORITY_HIGHEST`: it is idle
except for callbacks, and starvation is exactly what makes Windows drop hooks. No synthetic input is needed. The keyboard hook exists mainly
for this: without it, typing would look like missed mouse input.

Re-installing cannot help when the input went to an elevated window (UIPI hides it from our
hooks), so repeated re-installs back off exponentially (2 s, doubling, max 30 s) until hook events
resume. The decision logic is pure (`core::WatchdogPolicy`) and unit-tested, including tick
wraparound.

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

### Spike verdict and mixed DPI

The Phase 1 spike (full numbers in `docs/roadmap.md`) found the WebView2 overlay well inside the
targets: emit → first frame callback p99 2.3 ms, zero dropped frames in 10,875 at ~178 Hz, and the
page stays `visible` (never throttled) while idle and transparent. **Decision: go.** No native
Direct2D overlay is needed.

Two rules came out of it:

- **Pre-warm at startup.** The first reveal after launch is a cold start (78–94 ms). One invisible
  reveal at startup (full styling at `opacity: 0.002`) brings it to about 2 ms.
- **Measure symmetrically.** A frame-interval measurement that starts while the page is idle
  includes Chromium restarting its frame loop and reports a false "dropped" frame.

When a per-monitor-DPI-aware window moves onto a monitor with a different scale factor, Windows
resizes it, which breaks "never resize". The reference machine has no mixed DPI, so this was not
measured. The plan for Phase 2 is **one pre-created overlay per monitor**, each sized once at its
own DPI and never moved off its monitor. That removes cross-monitor DPI changes by construction, at
the cost of one extra WebView2 per extra monitor. Phase 2 verifies it with one monitor temporarily
at 125%.

## The settings window

- **Opaque**, so it pays none of the transparent-composition cost.
- **Created on demand** the first time it is opened (tray "Open settings", tray left-click, or a
  second launch of the exe). **Closing it hides it**, so reopening is instant and the app keeps
  running. Only tray **Quit** exits.
- It is never on the hot path. It reads and writes config through typed Tauri commands (Phase 3).

## Rust module map (`src-tauri/src/`)

| Module      | Role                                                                                                                                 | Status             |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------ | ------------------ |
| `lib.rs`    | Declares modules, re-exports `run`. No logic.                                                                                        | now                |
| `app/`      | Startup, file logging, the event pipeline, the runtime that owns background threads, shutdown.                                       | now                |
| `tray/`     | Tray icon and menu; maps clicks to actions and shows status (Pause/Resume, tooltip).                                                 | now                |
| `settings/` | Settings window lifecycle (create, reveal, hide-on-close).                                                                           | now                |
| `config/`   | Config schema, parsing, validation, load with last-good fallback and backup of broken files.                                         | now                |
| `hook/`     | The only place with Win32 hook code. Hook thread, callbacks, shared atomics, watchdog thread.                                        | now                |
| `core/`     | Trigger matching, press tracking, latency stats, watchdog policy; later profiles and actions. Pure logic.                            | now                |
| `platform/` | Thin safe wrappers over other Win32: clocks, input state; later window styles, monitors, DPI, foreground app, full-screen detection. | now (clocks, keys) |
| `overlay/`  | Overlay window lifecycle: create once, move, reveal.                                                                                 | Phase 2            |
| `ipc/`      | Tauri commands and event names. Validates payloads, delegates.                                                                       | Phase 2–3          |

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
- Invalid config never crashes the app (see `CLAUDE.md` hard rule 9). `config::load`:
  - no file → writes the defaults and uses them;
  - valid file → uses it and saves a copy as `config.last-good.json`;
  - invalid or unreadable file → copies it to `config.invalid-<unix-seconds>.json`, **leaves the
    original untouched** so the user can fix it, and runs on the last good config (or defaults).
    The tray tooltip says so and the log has the exact error.
- Tray **Reload config** re-reads the file without restarting.
- The trigger is `{ "kind": "mouse", "button", "modifiers", "swallow" }` or
  `{ "kind": "keyboard", "shortcut" }`. `left` is not an allowed button, so a bad config can never
  take over the primary button. Keyboard chords go through `tauri-plugin-global-shortcut`
  (`RegisterHotKey`), not the low-level hook.

## Logs

Daily files at `%LOCALAPPDATA%\com.liamerens.clickhub\logs\clickhub.YYYY-MM-DD.log`, last 7 kept,
written by a non-blocking background writer. `RUST_LOG` sets the level (default `info`). The
pipeline logs a latency summary every 100 button events and on exit: delivery latency (callback
start to pipeline receipt) and callback duration, as p50/p99/max, plus dropped events.

## Debug aids

Launching with `CLICKHUB_DEBUG_MENU=1` adds two tray items used to verify Phase 1 by hand:

- **Debug: simulate hook loss**: removes the hooks without telling the watchdog, exactly as
  Windows does on a timeout. Moving the mouse afterwards should log a re-install.
- **Debug: stall core for 5 s**: blocks the pipeline thread. With `swallow: true`, the trigger
  click must reach the app underneath during the stall (fail open).

## Known platform constraints

- **UIPI:** a non-elevated process does not receive low-level hook events for input aimed at
  elevated windows. The trigger does nothing over an elevated app. We never default to running as
  admin.
- **Anti-cheat:** we never inject into other processes. Full-screen apps auto-disable the trigger
  (Phase 2), with a per-app blocklist (Phase 5).
- **Unsigned releases:** SmartScreen warns until a code-signing certificate exists.
