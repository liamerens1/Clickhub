# CLAUDE.md: ClickHub

These rules override ad-hoc prompts. If a request conflicts with a hard rule, say so and ask
before going ahead.

## Read these first, every session

1. `docs/memory/project_memory.md`: local journal (gitignored). Read it before writing code or
   answering anything about project state. The latest "Next session starts here" says where to
   pick up.
2. `docs/roadmap.md`: the single source of truth for direction and the current phase.
3. `docs/coding-guidelines.md`: check it **before** writing code.
4. `docs/architecture.md`: read it before changing structure, threads or windows.
5. `docs/git-workflow.md`: branches, commits, PRs.

## What this is

ClickHub is a Windows 10/11 tray utility: a configurable radial "control center" (pie menu) that
appears at the cursor when the user presses a trigger they choose (any mouse button, including
middle and X1/X2, held or tapped, optionally with modifiers, or a keyboard chord). Selecting a
slot fires an action: launch an app/file/URL, send a shortcut, type text, run a command,
media/volume, or open a nested sub-ring. Rings can change per foreground app.

It works with any mouse, has a variable slot count, is themeable and high-contrast by default,
auto-disables in full-screen games, and must stay light and stable on multi-monitor setups.

## Hard rules (non-negotiable)

1. **Smoothness is a product requirement, measured, not eyeballed.** Every phase that touches UI
   lists these targets in its done criteria:
   - Trigger press to first ring frame: 16 ms or less.
   - Open, hover, select and close run at the display refresh rate (60/144/165 Hz) with no
     dropped frames.
   - Open animation about 120–180 ms, close about 100 ms. Animations never block input.
   - Hover highlight appears on the same frame the cursor enters a slice.
   - The settings window holds 60 fps while scrolling and dragging with 50+ actions configured.
2. **The overlay window is created once at startup and is never hidden, never resized and never
   focused.** It is ring-sized, not full-screen. When idle it is transparent and click-through,
   with WebView2 background throttling off. It is `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`: never in
   the taskbar or Alt-Tab, never takes focus (otherwise "send shortcut" hits the wrong window).
   It only moves to the cursor. The settings window is opaque.
3. **Animate only `transform` and `opacity`**, with eased or spring curves. No layout-triggering
   properties, no animated box-shadow, no large blurs. `prefers-reduced-motion` gets a fade-only
   fallback.
4. **The hot path never waits on IPC.** The ring for the active profile is pre-rendered; the
   trigger only positions and reveals it.
5. **The hook callback only matches the trigger and posts a message.** No allocation, I/O, IPC,
   logging or long-held locks. The hook lives on its own thread with its own message loop, and a
   watchdog re-installs it when Windows silently unhooks it.
6. **Fail open.** If any part of the pipeline is paused, hung, crashed, or the foreground app is
   full-screen, the user's input passes through untouched. ClickHub never swallows a mouse button
   it cannot act on.
7. **All `unsafe` and Win32 code lives in `src-tauri/src/hook/` and `src-tauri/src/platform/`
   only.** The crate denies `unsafe_code`; only those modules opt back in. The frontend never
   calls Win32 and never shells out.
8. **Never run as a Windows Service, never require admin by default, never inject into another
   process.**
9. **Config is validated at load.** serde types in Rust are mirrored by Zod in TS, and the shared
   fixtures in `fixtures/config` keep them in agreement. Invalid config never crashes the app: it
   is reported clearly, the app falls back to the last good config or defaults, and the user's
   broken file is backed up, never silently overwritten.
10. **Strict code.** No `any` in TS. No `unwrap()`/`expect()` in Rust outside tests and startup
    invariants (which carry an explicit `#[allow]` and a comment). Validate at every boundary:
    IPC payloads, config load, user input.
11. **Project memory.** Read `docs/memory/project_memory.md` before work. Append a dated entry
    before every commit. Never delete old entries; correct them in place with a note.
12. **Git.** Ask before running `git commit`. Never add `Co-Authored-By` trailers. Never commit
    directly to `main` or `dev`. Ask before any side effect outside the repo (sending input,
    installing global tools, changing GitHub settings, creating secrets, deploying, spending
    money).
13. **The roadmap is the source of truth.** The phase table below must always match
    `docs/roadmap.md`. Build in phase order and stop at every phase boundary for review, with the
    PR title and description.
14. **Verify for real.** Run it. Say exactly what was live-tested and what was not. Green checks
    alone are not verification.
15. **Use the Edit tool, not scripted string replacement, for text containing Windows paths**
    (`\b` becomes a backspace). The repo-hygiene test fails on control characters.
16. **The user works in Command Prompt.** Give cmd syntax, one command per line.

## Git workflow (summary)

- `main`: stable milestones only. `dev`: integration and GitHub default branch. Never commit to
  either directly.
- Work branches: `feat/<short-description>/<phase>` (e.g. `feat/foundation/0`) and
  `fix/<short-description>`.
- PRs target `dev`, titled `feat: …` / `fix: …` (`chore:` / `docs:` only for tooling- or
  docs-only PRs), with the full template in `.github/pull_request_template.md`.
- Commits: imperative mood, explain why. Push after each meaningful unit of work.
- `dev` merges into `main` at each phase boundary after review, tagged `v0.<phase>.0`.

Full rules: `docs/git-workflow.md`.

## Stack

- **Tauri 2.12** (Rust core + WebView2 frontend), per-user tray app, single instance.
- **Rust 1.97.1**, edition 2024, pinned in `rust-toolchain.toml`. `windows` 0.62 for Win32 (from
  Phase 1), serde, thiserror, tracing.
- **Node 24 LTS**, **pnpm 11.28.6** (`packageManager` field), exact versions, lockfiles committed.
- **Frontend**: Vite 8 with one entry per window. Settings is React 19 + TypeScript 6.0. The
  overlay (Phase 2) is vanilla TypeScript. Zod 4 for schemas.
- **Quality**: ESLint 10 + typescript-eslint (strictTypeChecked), Prettier 3, Vitest 5, clippy
  (pedantic, `-D warnings`), rustfmt.
- **Plugins**: single-instance now; global-shortcut (Phase 1, keyboard fallback only);
  autostart and updater (Phase 4).
- **Installer** (Phase 4): NSIS, `currentUser` install mode, no admin prompt.

## Commands

Run from the repo root in Command Prompt.

```
pnpm install
pnpm tauri dev
pnpm check
pnpm tauri build
```

- `pnpm tauri dev`: run the app with hot reload.
- `pnpm check`: lint, format check, typecheck and frontend tests.
- `pnpm tauri build`: release build plus NSIS installer.

Rust checks (from `src-tauri`):

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Fix formatting with `pnpm format` and `cargo fmt`.

## Phases (short version; must match docs/roadmap.md)

| Phase               | Goal                                                        | Status      |
| ------------------- | ----------------------------------------------------------- | ----------- |
| 0 — Foundation      | Repo, docs, rules, CI, empty tray app                       | In review   |
| 1 — Trigger         | Low-level hook thread, configurable button, overlay spike   | Not started |
| 2 — Ring            | Overlay at cursor, static actions, full-screen auto-disable | Not started |
| 3 — Settings UI     | Ring editor, trigger picker, persistence                    | Not started |
| 4 — Lifecycle       | Installer, autostart, uninstall, updater                    | Not started |
| 5 — Profiles        | Per-app rings, nested sub-rings, per-app blocklist          | Not started |
| 6 — Actions & modes | Remaining actions, marking/turbo modes, themes              | Not started |
| 7 — v1              | Hardening and release                                       | Not started |
| Parked              | Haptics, plugin marketplace, macOS/Linux, code signing      | Parked      |

## Known constraints ("don't fix these")

- **Elevated windows (UIPI):** a non-elevated app cannot hook input aimed at elevated windows.
  The trigger does nothing over them. Document it; never default to running as admin.
- **Session 0:** Windows Services cannot draw on the user's desktop or receive their input.
  ClickHub is a per-user tray app and will never run as a service.
- **Anti-cheat:** never inject into game processes. Full-screen exclusive apps auto-disable the
  trigger.
- **Signing:** releases are unsigned until a code-signing certificate exists, so SmartScreen
  warns on first run. Document it; don't work around it.
