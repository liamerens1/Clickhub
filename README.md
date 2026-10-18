# ClickHub

A configurable radial "control center" for Windows. Press a trigger you choose (any mouse button,
held or tapped, optionally with modifiers, or a keyboard chord) and a ring of actions appears at
the cursor: launch apps, files or URLs, send shortcuts, type text, run commands, control media,
or open nested rings. Rings can change per app.

- Works with any mouse, not one brand.
- Variable slot count, themeable, high-contrast by default.
- Auto-disables in full-screen games, with a per-app blocklist.
- Light, stable and smooth on multi-monitor setups.

> **Status:** Phase 0 (foundation). The app starts in the tray and does nothing else yet. See
> [docs/roadmap.md](docs/roadmap.md).

## Requirements

- Windows 10 or 11
- [Rust](https://rustup.rs/) (the version is pinned in `rust-toolchain.toml` and installed
  automatically)
- Node 24 LTS and pnpm 11 (`npm install -g pnpm@11.28.6`)
- Visual Studio Build Tools with "Desktop development with C++"
- WebView2 runtime (preinstalled on Windows 11)

## Development

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

## Known limitations

- **Elevated windows:** Windows does not let a normal app see mouse input aimed at apps running as
  administrator, so the trigger does nothing over them. ClickHub never runs as admin by default.
- **Unsigned builds:** releases are not code-signed yet, so Windows SmartScreen warns on first
  run.

## Docs

- [CLAUDE.md](CLAUDE.md): project rules
- [docs/roadmap.md](docs/roadmap.md): phases and status
- [docs/architecture.md](docs/architecture.md): why it is built this way
- [docs/coding-guidelines.md](docs/coding-guidelines.md): conventions
- [docs/git-workflow.md](docs/git-workflow.md): branches, commits, PRs
