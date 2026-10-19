# Roadmap

The single source of truth for direction. The phase table in `CLAUDE.md` must always match this
file. Update the status here first, then mirror it there.

Every phase that touches UI includes the smoothness targets from `CLAUDE.md` (hard rule 1) in its
done criteria. "Measured" means a number from a tool (QueryPerformanceCounter timestamps, PresentMon,
`requestAnimationFrame` timing, Chrome DevTools performance traces), recorded in the memory entry
and the PR.

Statuses: **Not started** · **In progress** · **In review** · **Done** · **Parked**.

Reference machine for measurements: RTX 3070 Ti, i7-10700K, 16 GB RAM, two monitors (1920×1080
and 2560×1440, both at 100% scaling; the main one at ~178 Hz). Mixed-DPI checks need one monitor
set to another scale temporarily.

---

## Phase 0 — Foundation

**Status:** Done (2026-10-19, PR #1, released to `main` as `v0.0.0`)

**Goal:** repo, docs, rules, CI and an empty Tauri tray app. No feature code.

**Done when:**

- `CLAUDE.md`, `docs/coding-guidelines.md`, `docs/architecture.md`, `docs/git-workflow.md`,
  `docs/roadmap.md` and `.github/pull_request_template.md` exist; `docs/memory/` is gitignored and
  has its first entry.
- CI is green on `windows-latest`: install, Rust fmt/clippy/test, frontend lint/format/typecheck/
  test, config fixture parity, repo hygiene.
- The app starts hidden to the tray (no visible window), single instance: a second launch exits
  and opens the running instance's settings window.
- The tray menu has Open settings (empty settings window), Pause (disabled placeholder) and Quit.
- Closing the settings window hides it; the app keeps running.
- Quit exits the process cleanly and leaves no tray icon behind.
- One trivial test in each language.

## Phase 1 — Trigger

**Status:** In progress (branch `feat/trigger/1`; spike on `spike/overlay/1`)

**Goal:** a low-level mouse/keyboard hook on its own thread, a trigger read from the config file
(hand-edited for now), optional swallowing of the trigger click, a working Pause, and the overlay
smoothness spike.

**Done when:**

- The chosen trigger logs an event within 5 ms of the hook timestamp: p99 over 1,000 presses,
  measured with QueryPerformanceCounter on both sides.
- The hook callback itself stays well under 1 ms at p99 (it only matches and posts).
- The hook survives 1 hour of normal use; a forced unhook is detected and re-installed by the
  watchdog within 1 s.
- Other buttons and keys are unaffected, including while the trigger is held.
- Fail-open verified: with the core thread stalled or paused, the trigger click reaches the app
  underneath.
- Pause in the tray stops the trigger and shows its state.
- Keyboard-chord fallback via `tauri-plugin-global-shortcut` works.
- **Overlay spike** (throwaway): a ring in an always-present transparent overlay window, measured
  on the reference machine against the smoothness targets. Records:
  - **Go/no-go** for the WebView2 overlay. No-go means a native overlay (Direct2D / Windows
    Composition) with Tauri kept for settings.
  - **The mixed-DPI approach.** Windows resizes a per-monitor-DPI-aware window that crosses to a
    monitor with a different scale, which conflicts with "never resize". Candidates: one
    pre-created overlay per monitor, or one overlay that handles `WM_DPICHANGED` itself and keeps
    its physical size.
  - The decision goes in this file and in the memory journal **before Phase 2 starts**.

### Overlay spike results (2026-10-19, branch `spike/overlay/1`, not merged)

Setup: a 360×360 transparent, always-on-top, click-through window with
`WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, created once and never hidden. A vanilla-TS page holds a
pre-rendered 8-slice SVG ring that animates only `transform` and `opacity` (open 150 ms with a
slight overshoot, close 100 ms) plus a simulated hover (class flip). A Rust thread moved the window
across both monitors and emitted `ring:open` / `ring:close`, 120 cycles per run, 4 runs. Frame
timing from `requestAnimationFrame` intervals; no synthetic input was sent.

Final run (run 4), reference machine at ~178 Hz (5.6 ms frames):

| Measure                                       | Result                                            | Target          |
| --------------------------------------------- | ------------------------------------------------- | --------------- |
| Rust emit → overlay listener                  | p50 0.32 ms, p99 1.28 ms                          | (part of 16 ms) |
| Rust emit → first frame callback after reveal | p50 0.5 ms, p99 2.3 ms                            | (part of 16 ms) |
| First reveal after launch                     | 1.9 ms with a startup pre-warm (78–94 ms without) | ≤ 16 ms         |
| Dropped frames, open + close animations       | **0 of 10,875 frames**                            | 0               |
| Page visibility while idle-transparent        | `visible` in all 120 samples                      | never throttled |
| Window resized by a move                      | never                                             | never           |

Hook delivery adds p99 ≈ 0.05 ms (see the latency summary), so trigger → first frame is about
1–8 ms (≤ 2.3 ms to schedule + up to one 5.6 ms vsync to present), well inside 16 ms.

Findings along the way:

- Runs 2–3 reported ~0.2% "dropped" frames. Every one was interval #0 of the close measurement:
  Chromium restarting its frame loop after idle, which the open measurement did not include.
  Measured symmetrically (run 4), there were none. Real mid-animation drops: zero in all runs.
- The first reveal after launch is a cold start (78–94 ms). An invisible pre-warm reveal at
  startup (full styling at `opacity: 0.002`) removes it.
- Tauri's `background_throttling` is unsupported on Windows. It is not needed: a never-hidden,
  always-on-top window stays `visible`, so WebView2 never throttles it.

**Decision: GO** for the WebView2 overlay (no native Direct2D fallback needed).

**Mixed DPI: not testable on the reference machine.** Both monitors (1920×1080 and 2560×1440) run
at 100% scaling, so no move ever crossed a DPI boundary. Recommendation for Phase 2: **one
pre-created overlay per monitor**, each sized once at its own DPI and never moved off its
monitor. It removes cross-monitor DPI changes by construction instead of fighting
`WM_DPICHANGED`, at the cost of one extra WebView2 per extra monitor. Phase 2 verifies it by
temporarily setting one monitor to 125% (done by the user; Claude does not change system
settings).

## Phase 2 — Ring

**Status:** Not started

**Goal:** the overlay at the cursor with static actions read from `config.json`, point-and-click
selection, high-contrast default theme, and basic full-screen auto-disable.

**Done when:**

- Trigger press to first ring frame is 16 ms or less (measured).
- Open/hover/select/close run at full refresh rate with no dropped frames on both monitors
  (PresentMon), including with one monitor temporarily at 125% scaling.
- One overlay per monitor, created once at startup and pre-warmed with an invisible reveal; none
  is ever hidden, resized or moved to another monitor.
- Hover highlight lands on the same frame the cursor enters a slice.
- The overlay never takes focus: a "send shortcut" slot reaches the app that was in front.
- Launch app/file, open URL and send shortcut actions work.
- Config is loaded from `%APPDATA%\com.liamerens.clickhub\config.json` and validated; invalid
  config is reported clearly (tray notification), the broken file is backed up, and the app falls
  back to last-good or defaults.
- When the foreground app is full-screen, the trigger passes through untouched.
- With Windows "Animation effects" off, the ring fades only.

## Phase 3 — Settings UI

**Status:** Not started

**Goal:** ring editor, trigger picker ("press the button you want"), persistence.

**Done when:**

- The user rebinds the trigger and edits slots without touching JSON.
- Config survives a restart.
- Config errors are shown in the settings window with what to fix.
- The settings window holds 60 fps while scrolling and dragging with 50+ actions (DevTools trace).

## Phase 4 — Lifecycle

**Status:** Not started

**Goal:** installer, autostart, uninstall, updater.

**Done when:**

- Per-user NSIS install with no admin prompt.
- ClickHub starts on login (HKCU Run key via `tauri-plugin-autostart`), hidden to the tray.
- Uninstall removes the Run entry. Config is kept unless the user ticks the uninstaller's "Delete
  the application data" box.
- Settings has an Uninstall button that launches the uninstaller.
- The update check works against a test release. This needs an updater signing key stored as a
  GitHub secret, which is created only after explicit approval. Releases stay unsigned
  (SmartScreen warning documented).

## Phase 5 — Profiles

**Status:** Not started

**Goal:** per-app rings, nested sub-rings, per-app blocklist, game exclusions.

**Done when:**

- The ring changes with the foreground app.
- Nested sub-rings open and close within the smoothness targets.
- Blocklisted apps and full-screen games never trigger the ring.

## Phase 6 — Actions & modes

**Status:** Not started

**Goal:** the remaining action types (type text, run command, media/volume), marking and turbo
interaction modes, and themes.

**Done when:**

- Each action type has an automated test and a manual check.
- Marking mode selects a slot by gesture without stopping the cursor; turbo mode selects on
  release of the held trigger.
- Built-in themes meet WCAG AA contrast on both light and dark backgrounds.

## Phase 7 — v1

**Status:** Not started

**Goal:** hardening and the first release.

**Done when:**

- An 8-hour soak with normal use shows no hook loss, leaks or crashes.
- Monitor hot-plug, DPI changes and sleep/resume are survived.
- README documents install, SmartScreen, the elevated-window limitation and uninstall.
- `dev` is merged into `main` and tagged `v1.0.0`.

---

## Parked

| Item               | Why parked                                                          |
| ------------------ | ------------------------------------------------------------------- |
| Haptics            | Needs vendor-specific hardware APIs; out of scope for any-mouse v1. |
| Plugin marketplace | Needs a stable action API and a security model first.               |
| macOS / Linux      | v1 is Windows 10/11 only; the hook and overlay are Win32-specific.  |
| Code signing       | Needs a certificate. Until then releases are unsigned.              |
