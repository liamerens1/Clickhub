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
 │   └─ overlay-0..n: one WebView2 per monitor, transparent,  (now)
 │       click-through, created once
 ├─ clickhub-hook: SetWindowsHookExW + own message loop       (now)
 ├─ clickhub-pipeline: hook events → ring session ("core")    (now)
 ├─ clickhub-actions: runs fired slot actions                 (now)
 ├─ clickhub-watchdog: re-installs dropped hooks              (now)
 ├─ clickhub-fullscreen: suppresses the trigger in games      (now)
 └─ WebView2 browser/renderer processes (child processes owned by WebView2)
```

- **Main thread.** Tauri's event loop. It owns all windows and the tray. It never blocks.
- **Hook thread** (`hook/thread.rs`). Installs `WH_MOUSE_LL` and `WH_KEYBOARD_LL` with the raw
  `SetWindowsHookExW` from the `windows` crate. Low-level hook callbacks run on the thread that
  installed them, and only while that thread pumps messages, so the thread exists to do exactly
  that. Other threads talk to it with `PostThreadMessageW` (re-install, stop). We do not use
  `rdev`: it can only listen, and we must be able to swallow the trigger click.
- **Pipeline** (`app/pipeline.rs`, the "core" consumer). Receives hook events and drives the pure
  `core::RingSession` (tap/hold state machine) and `core::aim` (direction → slot). It turns the
  session's effects into overlay events, sets the hook's ring-open flag, and queues fired actions.
  It checks in (a heartbeat) at least every 100 ms, even when idle, and logs latency summaries.
- **Actions** (`actions/`). Runs fired slot actions (send shortcut, launch, open URL) on its own
  thread with COM initialised, so a slow launch never stalls the pipeline.
- **Watchdog** (`hook/watchdog.rs`, policy in `core/watchdog_policy.rs`). See below.
- **Full-screen watch** (`app/fullscreen_watch.rs`). Every 250 ms checks whether the foreground
  app is full-screen (exclusive Direct3D, presentation mode, or a window covering its monitor,
  excluding the desktop) and sets the hook's suppression flag: the trigger then passes through.

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
telling the app; the only symptom is that events stop arriving. The watchdog polls every 100 ms
and compares `GetLastInputInfo` (the newest input Windows saw in the session, from any device) with
the newest event either hook saw. If Windows saw input the hooks missed by more than 250 ms on
**two consecutive polls**, it asks the hook thread to re-install, so a real loss is repaired within
about 0.5 s of the next input. The second poll filters out a callback that merely ran late under
heavy CPU load and then caught up, which a one-poll check mistook for a loss during the first live
run (while a Rust build saturated the CPU). The hook thread also runs at
`THREAD_PRIORITY_HIGHEST`: it is idle except for callbacks, and starvation is exactly what makes
Windows drop hooks. No synthetic input is needed. The keyboard hook exists mainly for this:
without it, typing would look like missed mouse input.

Re-installing cannot help when the input went to an elevated window (UIPI hides it from our
hooks), so repeated re-installs back off exponentially (2 s, doubling, max 30 s) until hook events
resume. Every re-install logs the foreground app's executable, PID and elevation
(`platform::foreground_app`), so a log line tells a real drop apart from input to an admin window.
The decision logic is pure (`core::WatchdogPolicy`) and unit-tested, including tick wraparound.

## The hot path

From physical click to first ring frame, the budget is **16 ms**:

```
 mouse button down
   → hook callback (hook thread)        match trigger from atomics; swallow; post an event
   → pipeline (RingSession)             TriggerDown → effect Open { x, y }
   → overlay/ (Rust)                    move that monitor's overlay within its monitor; emit `ring:open`
   → overlay page (WebView2)            set the root transform; start the rAF bloom; DOM already built
   → first frame on screen              page reports `overlay:frame`; Rust logs hook→emit and emit→frame
```

**Edge-aware layout.** On open, `core::layout` checks which directions leave room for a label
inside the overlay window (which itself lies inside the monitor), using the widest label the page
measured (`overlay:extent`). With room all round the ring is normal; near an edge or corner the
slots spread evenly over the largest free arc, clockwise order kept and the cut on the blocked
side, and the ring grows (lines and labels reach up to 1.6x further; the empty centre stays the
same) when that gives neighbouring labels more room. Only labels within ~4.6 degrees of straight
up or down are centred on their point; the rest hang away from the centre, so neighbours near the
top or bottom never collide. The centre text shifts just enough to stay inside the window. Rust aims against these angles and sends them in `ring:open`, so what is drawn and what is
picked always agree; the focus zoom spreads relative to the (possibly tighter) spacing.

**Automatic ink.** Plain text has nothing behind it, so its colour must suit the screen. On open,
Rust downscales the screen area behind the overlay into a coarse brightness grid with one GDI
`StretchBlt` (`platform::sample_luma`; nothing is stored) and picks dark or light ink per slot
from the area under each label, plus one for the centre (`core::ring_ink`). Reading screen pixels
back from the compositor is slow (6-11 ms, measured live), so it is **off the hot path**: the
ring opens at once in the theme colour, and the `clickhub-ink` thread (`overlay::InkWorker`) sends
`ring:ink` about 10 ms later, while the lines have barely started growing and well before any
label pops in. The sampling time is logged per open (`ring ink`, `sample_us`). If sampling fails
the theme colour stays.

**Staying on top.** Each open re-asserts the overlay as topmost, and while a ring is open the
pipeline re-asserts it every 100 ms, so switching apps (Alt+Tab) cannot bury it under another
always-on-top window.

While the ring is open the hook also posts mouse moves; the pipeline computes the aim from the
offset to the ring centre and emits `ring:aim` at most every 4 ms (immediately when the focused
slot changes, with a trailing update for the exact final position). The page eases every slot
toward its focus weight in one `requestAnimationFrame` loop, so the zoom starts on the next frame.
Clicks and Esc while open are swallowed by the hook and become session inputs; the session fires
the aimed slot or closes. All overlay events are fire-and-forget: Rust never waits on the page.

Rules that keep it fast:

- **The hook callback only matches and posts.** No allocation, no I/O, no IPC, no logging, no
  locks held for long. Anything slow risks Windows unhooking us, and every microsecond there is
  added latency for every mouse event system-wide. Swallowing is decided in the callback from
  precomputed state (`return 1` vs `CallNextHookEx`).
- **No IPC round trip on the path.** The Rust side sends one fire-and-forget event; it never waits
  for the frontend to answer before the frame.
- **The ring is pre-rendered.** On `ring:config` (startup and config reload) the page builds the
  ring DOM and pre-warms it with one invisible open. The trigger only positions and reveals it.
- **Animate only `transform` and `opacity`.** These run on the compositor without layout or paint.

## Why the overlay is never hidden

There is one overlay window per monitor, each created **once** at startup and **never hidden, never
resized, never focused and never moved to another monitor**. It is always transparent and
click-through. This is deliberate:

- **Showing a hidden WebView2 window is slow and ugly.** WebView2 throttles hidden windows: timers
  and `requestAnimationFrame` slow down and the compositor parks the surface. On show, it can flash
  the last stale frame and then play the open animation throttled. An always-present, transparent
  window is already "warm": the first frame after the trigger is a normal frame.
- **It is never throttled.** Tauri's `background_throttling` option is unsupported on Windows, but
  it is not needed: a never-hidden, always-on-top window stays `visible` to WebView2.
- **It is ring-sized, not full-screen.** Transparent WebView2 windows use per-pixel alpha
  composition, and its cost scales with the window area. A ring-sized window is cheap; a
  full-screen transparent window across a 4K monitor is not.
- **It moves, never resizes.** Resizing a WebView2 window re-allocates its surface and flickers.
  Moving is just a position change.
- **It never takes focus** (`WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`). If the ring took focus, a "send
  shortcut" action would go to the ring instead of the app the user was working in. It also never
  shows in the taskbar or Alt-Tab.
- **Always click-through** (ignore-cursor-events), even while the ring is open. Aim and clicks come
  from the low-level hook, so the window never needs mouse input and never blocks a click.
- **Created hidden exactly once.** Each overlay is built hidden, moved onto its monitor, sized at
  that monitor's DPI, made no-activate, then shown, and never hidden again.

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
resizes it, which breaks "never resize". So Phase 2 uses **one pre-created overlay per monitor**
(`overlay::Overlays`), each sized once at its own DPI and only moved within its own monitor: the
ring opens in the overlay of the monitor under the cursor, positioned so the ring centre lands on
the cursor (clamped to the monitor; the page draws the ring at the given centre). That removes
cross-monitor DPI changes by construction, at the cost of one extra WebView2 per extra monitor.
Monitors connected after startup get no overlay until restart (Phase 7 handles hot-plug).

## The settings window

- **Opaque**, so it pays none of the transparent-composition cost.
- **Created on demand** the first time it is opened (tray "Open settings", tray left-click, or a
  second launch of the exe). **Closing it hides it**, so reopening is instant and the app keeps
  running. Only tray **Quit** exits.
- It is never on the hot path. It reads and writes config through typed Tauri commands in
  `ipc/` (wrapped and Zod-validated in `src/shared/ipc/settings-commands.ts`):
  - `get_config` / `reload_config` return a snapshot: the config in use, its path, and the load
    problem if `config.json` could not be used (shown as a banner with Open folder and Reload).
  - `save_config` takes a whole `Config` (serde checks its shape), checks that a keyboard trigger
    can be registered, then `config::save` validates it with the same `parse` as loading, writes
    it atomically (temp file + rename), backs up the file it replaces if that file was invalid,
    refreshes `config.last-good.json`, and the runtime applies it at once.
  - `get_status` / `set_paused`, `get_app_info`, `open_folder` (`config` or `logs` only, opened
    through `platform::shell_open` on a short-lived COM thread), and `pick_file` (the Windows file
    dialog from `tauri-plugin-dialog`, called from Rust, so the page gets no dialog permission).
  - `set_trigger_capture` switches the trigger off while the page waits for "press the button
    you want": the hook passes every button through (the same flag as Pause, OR-ed with the
    user's own pause) and the keyboard trigger is unregistered, so the press reaches the page.
    The page switches it back on when it gets a press, is cancelled, loses focus or unmounts.
- Rust tells the page about changes made elsewhere with two events addressed to the `settings`
  window: `config:changed` (re-read the config) and `status:changed` (pause state, tray problem).
- **Commands are gated per window.** `build.rs` lists the app's commands in Tauri's app manifest,
  which generates one `allow-<command>` permission each (in the gitignored
  `permissions/autogenerated/`). `capabilities/settings.json` grants them to `settings`;
  `capabilities/overlay.json` gives the overlays events only.
- **Saving is instant.** Every valid change is saved and applied straight away (text fields once
  typing pauses for ~450 ms), one save at a time and in order. A failed save is shown beside Undo
  and the page rolls back to what Rust last confirmed. Undo keeps up to 50 earlier configs for the
  session and saves the previous one.
- **Smooth with many rows.** Reordering moves rows only with `transform`, written straight to the
  DOM once per animation frame (no React render during a drag); rows use
  `content-visibility: auto`. A dev-only harness (`pnpm dev`, then `/settings/?perf=60`) loads the
  list with 60 rows; the same URL without `perf` runs the whole page in a plain browser against an
  in-memory mock of the commands (`src/shared/ipc/dev-mock.ts`, never in a build).

## Rust module map (`src-tauri/src/`)

| Module      | Role                                                                                                                                              | Status |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| `lib.rs`    | Declares modules, re-exports `run`. No logic.                                                                                                     | now    |
| `app/`      | Startup, file logging, the event pipeline, the runtime that owns background threads, shutdown.                                                    | now    |
| `tray/`     | Tray icon and menu; maps clicks to actions and shows status (Pause/Resume, tooltip).                                                              | now    |
| `settings/` | Settings window lifecycle (create, reveal, hide-on-close).                                                                                        | now    |
| `config/`   | Config schema, parsing, validation, load with last-good fallback and backup of broken files.                                                      | now    |
| `hook/`     | The only place with Win32 hook code. Hook thread, callbacks, shared atomics, watchdog thread.                                                     | now    |
| `core/`     | Trigger matching, press tracking, ring aim and tap/hold session, latency stats, watchdog policy. Pure logic.                                      | now    |
| `platform/` | Thin safe wrappers over other Win32: clocks, input state, `SendInput`, `ShellExecuteW`, foreground app and elevation, full-screen, window styles. | now    |
| `overlay/`  | One overlay window per monitor: create once, move within its monitor, emit ring events, log frame latency.                                        | now    |
| `actions/`  | Runs fired slot actions on a dedicated thread through `platform/`.                                                                                | now    |
| `ipc/`      | Tauri commands for the settings window and the events sent to it. Validates payloads, delegates.                                                  | now    |

`unsafe` is denied crate-wide. `hook/` and `platform/` opt back in with `#![allow(unsafe_code)]`.
That keeps every unsafe block in two reviewable places.

## Frontend map (`src/`)

| Folder      | Role                                                                                                                                                                    | Status |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| `settings/` | React + TS. Vite entry `settings/index.html`. Sidebar sections (Ring, Trigger, General, About), the ring editor with live preview, trigger capture, `use-config-store`. | now    |
| `shared/`   | Zod schemas (mirroring Rust serde types) and typed IPC wrappers (`shared/ipc`, the only code importing `@tauri-apps/*`).                                                | now    |
| `overlay/`  | Vanilla TS ring: `motion.ts` (pure easing, bloom, focus weights), `ring-view.ts` (DOM build + transform writes), `ring-animator.ts` (the rAF loop), `main.ts` (events). | now    |

The overlay is vanilla TS on purpose: no framework runtime and no reconciliation on aim changes.
The approved design (roadmap, Phase 2) is plain 1 px lines and plain text, no backgrounds. Every
line and label is its own compositor layer (`will-change`); one `requestAnimationFrame` loop eases
values and writes only `transform` and `opacity`, then stops when nothing moves. Labels are drawn
at their full zoomed size and scaled down at rest, so zooming never re-rasterizes or blurs text.
An SVG + CSS-transition mockup of the same design lagged; this structure is the fix.

Overlay events: `ring:config` (slot labels and hints), `ring:open` (centre, seq, emit time),
`ring:aim` (continuous angle in slot units + nearest slot), `ring:close`; and from the page,
`overlay:ready` and `overlay:frame` (emit → first frame delay). Payloads are validated with Zod on
the page; the Rust structs live in `overlay/view.rs`.

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
- Tray **Reload config** (or Reload in settings) re-reads the file without restarting.
- The settings window saves through `config::save` (see above); it is the only other writer.
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
