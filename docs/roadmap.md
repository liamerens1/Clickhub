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

**Status:** Done (2026-10-19, PR #3; release PR #4 to `main` as `v0.1.0`). Carried into Phase 2
as explicit checks: fail-open with `swallow: true` live, presses ignored while paused, the
keyboard trigger, and a 1-hour run.

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

**Status:** Done (2026-10-19, PR #5; release PR #6 to `main` as `v0.2.0`). Not yet live-tested,
carried into Phase 3 as explicit checks: fail-open with swallowing on (stalled core), a "send
shortcut" slot landing in Notepad (focus never stolen), one monitor at 125%, the reduced-motion
fade, the keyboard-chord trigger and a 1-hour run. Known issue: on the 180 Hz monitor the ring
animates at the 165 Hz monitor's cadence (follow-up).

**Goal:** the ring at the cursor with static actions read from `config.json`, tap or hold
interaction, the approved spoke design, and basic full-screen auto-disable.

**Design (approved 2026-10-19 from interactive mockups):** no backgrounds or borders. Plain 1 px
lines radiate from a wide empty centre (~72 px) to plain-text labels; the centre shows the
focused action's name and shortcut as plain text. Text and lines follow Windows light/dark.
Direction from the centre picks the item. Focus is a graded, continuous zoom: the aimed item's
line grows ~16% and its label zooms to ~1.3x, neighbours zoom less, others stay at rest. Open is
a ~300 ms staged bloom (lines grow outward with a small stagger, each label pops in with a slight
overshoot as its line arrives); close ~100 ms. Built as HTML layers animated by one
`requestAnimationFrame` loop with text pre-sized at its zoomed size (the SVG + CSS-transition
mockup lagged).

**Interaction:** tap the trigger and the ring stays open: click to fire the aimed item, click in
the centre, right-click or Esc to close. Hold the trigger, aim, and release to fire; releasing in
the centre cancels. Swallowing is on by default from this phase.

**Done when:**

- Trigger press to first ring frame is 16 ms or less (measured).
- Open/hover/select/close run at full refresh rate with no dropped frames on both monitors
  (PresentMon), including with one monitor temporarily at 125% scaling.
- One overlay per monitor, created once at startup and pre-warmed with an invisible open; none
  is ever hidden, resized or moved to another monitor.
- The focus zoom starts moving on the same frame the aim changes.
- Tap and hold both work as described above; clicks and Esc used by an open ring never reach the
  app underneath.
- Near screen edges and corners every item stays visible and reachable: the ring stays centred on
  the cursor and the slots spread evenly over the free arc, keeping their clockwise order (chosen
  by the user over shifting the ring).
- Fail-open verified live with swallowing on: a stalled core lets the trigger and clicks through.
- The overlay never takes focus: a "send shortcut" slot reaches the app that was in front.
- Launch app/file, open URL and send shortcut actions work.
- Config is loaded from `%APPDATA%\com.liamerens.clickhub\config.json` and validated; invalid
  config is reported clearly (tray notification), the broken file is backed up, and the app falls
  back to last-good or defaults.
- When the foreground app is full-screen, the trigger passes through untouched.
- With Windows "Animation effects" off, the ring fades only.

## Phase 3 — Settings UI

**Status:** Done (2026-10-19, PR #7; release PR #8 to `main` as `v0.3.0`). Live-tested by the
user: the settings flows, the keyboard-chord trigger and "send shortcut" landing in Notepad.
Carried into Phase 4 as explicit checks (the user chose to release now): fail-open with
swallowing on (stalled core), one monitor at 125%, the reduced-motion fade, the ink colour flicker
check, a 1-hour run, and a frame-rate measurement inside the real settings window (so far
measured in headless Edge: 0 slow frames of 720 with 60 rows).

**Goal:** ring editor, trigger picker ("press the button you want"), persistence.

**Design (chosen 2026-10-19 from an interactive mockup):** soft and rounded throughout (rounded
panels, rows, buttons and sidebar items; no hard borders), following Windows light/dark.

- **Layout:** a slim left sidebar with sections Ring, Trigger, General and About (Profiles in
  Phase 5 and Themes in Phase 6 join it later); the selected section fills the right.
- **Saving:** every change applies to the live ring at once and is written to `config.json`; Undo
  (button and Ctrl+Z) steps back through this session's changes. Text fields save when they are
  valid and the user pauses typing.
- **Ring editor:** a list of slots, clockwise from the top, reordered by dragging (or Alt+Up/Down)
  and edited in place, beside a small live preview of the ring.
- **Actions:** a shortcut recorder (press the keys; a Win toggle covers chords Windows keeps for
  itself), a Browse button using the Windows file dialog for apps and files, and quick presets
  (Copy, Paste, Undo, Screenshot, Task Manager, Show desktop, ...).

**Done when:**

- The user rebinds the trigger and edits slots without touching JSON.
- Config survives a restart.
- Config errors are shown in the settings window with what to fix.
- The settings window holds 60 fps while scrolling and dragging with 50+ actions (DevTools trace
  or `requestAnimationFrame` timing). A ring holds at most 12 slots, so until profiles arrive in
  Phase 5 this is measured on the same list component with a synthetic 60-row list.
- The Phase 2 checks carried over above are run and recorded.

## Phase 4 — Lifecycle

**Status:** In progress (branch `feat/lifecycle/4`)

**Goal:** installer, autostart, uninstall, updater.

**Design (chosen by the user 2026-10-19):**

- **Updates:** checked quietly at startup and once a day against GitHub Releases. A newer
  version shows a tray notification and a banner in settings with Install; nothing installs
  without the user's click. About also has Check now.
- **Autostart:** on by default for an installed copy (applied once, on its first run); a switch
  in General turns it off. Builds run from the repo never register themselves.
- **Releases:** pushing a `v*` tag runs a GitHub Actions workflow that builds the NSIS installer
  on `windows-latest`, signs the update files with the updater key (a GitHub secret, created only
  after the user approves) and creates a draft GitHub Release that the user publishes.
- **Uninstall:** the settings button asks first, then starts the normal uninstaller and quits
  ClickHub so it can remove the files.

**Done when:**

- Per-user NSIS install with no admin prompt.
- ClickHub starts on login (HKCU Run key via `tauri-plugin-autostart`), hidden to the tray.
- Uninstall removes the Run entry. Config is kept unless the user ticks the uninstaller's "Delete
  the application data" box.
- Settings has an Uninstall button that launches the uninstaller.
- The update check works against a test release. This needs an updater signing key stored as a
  GitHub secret, which is created only after explicit approval. Releases stay unsigned
  (SmartScreen warning documented).
- The Phase 3 checks carried over above are run and recorded.

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
interaction modes, and selectable ring themes.

**Done when:**

- Each action type has an automated test and a manual check.
- Marking mode selects a slot by gesture without stopping the cursor; turbo mode selects on
  release of the held trigger.
- **Theme presets for the ring** (requested 2026-10-19): the user picks one of several built-in
  presets in settings, e.g. the current plain spokes, plus variants with different line styles,
  label weights, sizes or accent colours. The choice is saved in `config.json`, switches without
  a restart, and every preset keeps the smoothness targets and the automatic light/dark ink.
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
