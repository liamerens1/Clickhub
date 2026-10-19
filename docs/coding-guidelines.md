# Coding guidelines

Check this **before** writing code. It covers both Rust (`src-tauri/`) and TypeScript (`src/`).
Where a rule is enforced by tooling, the tool is named; where it is not, review enforces it.

## 1. Module boundaries

Every module has one job and a list of things it may **never** do. If a change needs to break a
boundary, the structure is wrong: discuss it, and update `docs/architecture.md` in the same PR.

### Rust (`src-tauri/src/`)

| Module      | Does                                                                                 | May never…                                                                                              |
| ----------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| `lib.rs`    | Declares modules, re-exports `run`                                                   | Contain logic.                                                                                          |
| `app/`      | Startup, logging, run loop, shutdown                                                 | Hold feature logic; call Win32 directly.                                                                |
| `hook/`     | Low-level hooks, hook thread, watchdog                                               | Touch UI, Tauri, IPC, config or files. In the callback: allocate, log, lock for long, block, or do I/O. |
| `core/`     | Trigger matching, profiles, action resolution                                        | Use Win32, Tauri, threads it doesn't own, or I/O. It is pure logic.                                     |
| `platform/` | Safe wrappers over Win32 (window styles, monitors, DPI, foreground app, full-screen) | Hold business logic; depend on Tauri.                                                                   |
| `overlay/`  | Overlay window: create once, move, reveal                                            | Hide, resize or focus the overlay; wait on the frontend in the hot path.                                |
| `settings/` | Settings window lifecycle                                                            | Sit on the hot path; call Win32 directly.                                                               |
| `config/`   | Schema, parse, validate, load/save/backup                                            | Panic on bad input. No other module reads or writes the config file.                                    |
| `tray/`     | Tray icon and menu, dispatch                                                         | Hold logic beyond dispatching to another module.                                                        |
| `ipc/`      | Tauri commands and event names                                                       | Hold logic; trust a payload before validating it.                                                       |

- `unsafe` is denied crate-wide (`[lints.rust] unsafe_code = "deny"`). Only `hook/` and
  `platform/` opt back in with `#![allow(unsafe_code)]` at the top of their `mod.rs`. Every
  `unsafe` block has a `// SAFETY:` comment.
- `hook/` exposes only start, stop and an event channel. Its callback reads precomputed state.

### TypeScript (`src/`)

| Folder      | Does                                   | May never…                                                                                                        |
| ----------- | -------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `settings/` | React settings window                  | Import from `overlay/`; call `invoke` directly (use `shared/ipc`); use Node APIs.                                 |
| `overlay/`  | Vanilla TS ring (Phase 2)              | Import React or `settings/`; await IPC inside a frame; animate anything but `transform`/`opacity`; use Node APIs. |
| `shared/`   | Zod schemas, types, typed IPC wrappers | Touch the DOM, import React, have side effects. Only `shared/ipc` imports `@tauri-apps/*`.                        |

- The frontend never calls Win32 and never shells out. Anything OS-level goes through a typed
  Tauri command.
- Enforced by ESLint `no-restricted-imports` in `eslint.config.js`.

## 2. Naming

| Thing                                  | Convention                                                  | Example                         |
| -------------------------------------- | ----------------------------------------------------------- | ------------------------------- |
| Rust modules and files                 | `snake_case`                                                | `menu_action.rs`                |
| Rust types, traits, enum variants      | `PascalCase`                                                | `MenuAction::OpenSettings`      |
| Rust functions, variables              | `snake_case`                                                | `open_settings`                 |
| Rust constants, statics                | `SCREAMING_SNAKE_CASE`                                      | `CONFIG_VERSION`                |
| Rust test functions                    | `snake_case`, describe the behaviour                        | `rejects_every_invalid_fixture` |
| TS files                               | `kebab-case`                                                | `ring-editor.tsx`               |
| TS test files                          | Mirror the source name + `.test`                            | `ring-editor.test.tsx`          |
| React components                       | `PascalCase`                                                | `RingEditor`                    |
| TS functions, variables                | `camelCase`                                                 | `parseConfig`                   |
| TS constants (module-level, primitive) | `SCREAMING_SNAKE_CASE`                                      | `CONFIG_VERSION`                |
| TS types, interfaces                   | `PascalCase`, no `I` prefix                                 | `Config`, not `IConfig`         |
| Zod schemas                            | `camelCase` + `Schema`; inferred type drops it              | `configSchema` → `Config`       |
| Tauri commands                         | `snake_case` in Rust, `camelCase` TS wrapper                | `get_config` / `getConfig()`    |
| IPC event names                        | `domain:action`                                             | `ring:open`, `config:changed`   |
| Window labels                          | lowercase single word                                       | `overlay`, `settings`           |
| Tray menu ids                          | `snake_case`                                                | `open_settings`                 |
| Config keys (JSON)                     | `camelCase`; Rust uses `#[serde(rename_all = "camelCase")]` | `slotCount`                     |
| CSS classes                            | `kebab-case`                                                | `ring-slice`                    |
| CSS custom properties                  | `--ch-` prefix                                              | `--ch-bg`                       |
| Fixture files                          | `kebab-case`, named after what they test                    | `unknown-key.json`              |
| Branches                               | `feat/<desc>/<phase>`, `fix/<desc>`                         | `feat/foundation/0`             |

## 3. One main thing per file

- A file holds one main type, component or function plus its private helpers.
- Rust: `mod.rs` declares submodules and re-exports; it holds no logic. `menu_action.rs` holds
  `MenuAction`; `icon.rs` builds the tray.
- TS: one exported component per `.tsx`; one schema per schema file.
- If a file needs a second public thing, it becomes a folder.

## 4. Exports

### Rust: `mod.rs` and `pub use`

- Each module folder has a `mod.rs` that declares private submodules and re-exports the public
  surface with `pub use`. Callers write `crate::config::parse`, never
  `crate::config::parse::parse`.
- Default to private. Use `pub(crate)` for crate-internal APIs. Use `pub` only for what
  integration tests in `src-tauri/tests/` need.

### TypeScript: barrels

- Folders imported from elsewhere (`shared/schema`, `shared/ipc`) have an `index.ts` barrel that
  re-exports their public surface. Import from the barrel: `../shared/schema`.
- Window entry folders (`settings/`, `overlay/`) do not need barrels; nothing imports them.
- No `export *`. Name what you export.

## 5. Import order

### Rust (blank line between groups; rustfmt sorts within each group, review enforces the grouping)

1. `std` / `core`
2. External crates (`tauri`, `serde`, …)
3. `crate::` and `super::`

### TypeScript (enforced by `eslint-plugin-simple-import-sort`)

1. Side-effect imports (`import "./settings.css"`)
2. Node built-ins (`node:fs`; tests and tooling only)
3. Packages (`react`, `zod`, `@tauri-apps/api`)
4. `shared/` imports
5. Relative imports

Use `import type` for type-only imports (`@typescript-eslint/consistent-type-imports`).

## 6. Strict typing

### TypeScript

- `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
  `noPropertyAccessFromIndexSignature` and `verbatimModuleSyntax` are on (`tsconfig.json`).
- **No `any`**, explicit or implicit (`@typescript-eslint/no-explicit-any`, `strictTypeChecked`).
  Use `unknown` and narrow it.
- No non-null assertions (`!`). Check and handle the `null`.
- Types for data crossing a boundary come from Zod (`z.infer`), never hand-written duplicates.

### Rust

- **No `unwrap()` or `expect()`** outside tests (clippy `unwrap_used` and `expect_used` deny them;
  `clippy.toml` allows them in tests).
- **Startup invariants** are the one exception. Use an explicit
  `#[allow(clippy::expect_used)]` with a comment explaining why failure is impossible or
  unrecoverable at startup.
- No `todo!`, `unimplemented!`, `dbg!`, `println!` or `eprintln!` in shipped code (clippy denies
  them). Use `tracing`.
- `clippy::pedantic` is on at warn level, and CI turns warnings into errors.

## 7. Validation at every boundary

Validate data where it enters, then trust the types inside.

| Boundary                  | Validated by                                                                          |
| ------------------------- | ------------------------------------------------------------------------------------- |
| Config file load          | `config::parse` (serde, `deny_unknown_fields`, version check, root must be an object) |
| Config in the settings UI | `configSchema` (Zod `.strict()`)                                                      |
| IPC: frontend → Rust      | serde types on every Tauri command argument, plus semantic checks in `ipc/`           |
| IPC: Rust → frontend      | Zod parse in the `shared/ipc` wrapper before the payload is used                      |
| User input in settings    | Zod schema for the form before saving                                                 |

- The Rust and TS schemas mirror each other. Every change to the config shape adds or updates
  fixtures in `fixtures/config/{valid,invalid}`. Both test suites run every fixture.
- Parsers never throw or panic on bad input. They return a typed error with a plain-English
  message (`ConfigError`, `ParseConfigResult`).

## 8. Error handling

### Rust

- Library-style modules return `Result<T, ModuleError>` with a `thiserror` enum. Messages are
  plain English because they may reach the user.
- Use `?` to propagate. Convert at module boundaries with `#[from]` or `map_err`.
- At the top of a user action (a tray click, a command), log the error and degrade gracefully;
  don't crash. Example: `settings::open` logs and returns if the window can't be created.
- Panics are bugs. The hook thread must never panic. Wrap its work so a failure is reported and
  the watchdog restarts it.

### TypeScript

- Expected failures are values, not exceptions: `{ ok: true, … } | { ok: false, error }`.
- `throw` only for programmer errors that can't happen in a correct build (e.g. a missing
  `#root`).
- Never swallow an error silently. Show it to the user or log it.

## 9. Logging

- Rust uses `tracing` (`info!`, `warn!`, `error!`, `debug!`) with structured fields:
  `tracing::error!(%error, "could not create the settings window")`.
- Level is set by `RUST_LOG` (default `info`). File logging under `%LOCALAPPDATA%` arrives in
  Phase 1.
- Never log inside the hook callback. Count, and report from another thread.
- Never log config contents that may hold personal data (typed text, commands) at `info` or above.
- The frontend logs through `console` only during development. User-visible problems go through
  the UI.

## 10. Tests

- **Tests mirror the source tree.**
  - TS: `src/settings/app.tsx` → `tests/settings/app.test.tsx`.
  - Rust: `src-tauri/src/config/` → `src-tauri/tests/config.rs` (integration tests against the
    public surface). Private helpers may have an inline `#[cfg(test)] mod tests` in the same file.
- Vitest runs in Node by default. Component tests start with `// @vitest-environment jsdom`.
- Test behaviour, not implementation. Name tests after what they prove.
- Shared fixtures live in `fixtures/` and are read by both suites.
- `tests/repo/hygiene.test.ts` fails on control characters, CRs or a BOM in any text file. Use the
  Edit tool, not scripted replacement, for text with Windows paths.
- Performance targets are verified by measurement (see `docs/roadmap.md`), not by unit tests.

## 11. Formatting

- Prettier (`.prettierrc.json`, 100 columns, LF) for TS, CSS, HTML, JSON, YAML and Markdown.
- rustfmt defaults for Rust.
- `.editorconfig` and `.gitattributes` keep every text file UTF-8, LF, final newline.

## 12. Performance rules for UI code

These come from `CLAUDE.md` hard rules 1–4 and apply to every UI change:

- Animate only `transform` and `opacity`. Never animate `width`, `height`, `top`, `left`,
  `margin`, `box-shadow` or `filter: blur` on large areas.
- No IPC awaits inside animation or pointer handlers on the overlay.
- No layout reads (`getBoundingClientRect`, `offsetWidth`) inside pointer-move handlers. Cache
  geometry when the ring is built; hit-test with math.
- Respect `prefers-reduced-motion` with a fade-only fallback.
- Settings lists with many items use virtualisation or `content-visibility` so 50+ actions stay at
  60 fps.
