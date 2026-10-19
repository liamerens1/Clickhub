# ClickHub

A configurable radial "control center" for Windows. Press a trigger you choose (any mouse button,
held or tapped, optionally with modifiers, or a keyboard chord) and a ring of actions appears at
the cursor: launch apps, files or URLs, send shortcuts, type text, run commands, control media,
or open nested rings. Rings can change per app.

- Works with any mouse, not one brand.
- Variable slot count, themeable, high-contrast by default.
- Auto-disables in full-screen games, with a per-app blocklist.
- Light, stable and smooth on multi-monitor setups.

> **Status:** Phase 1 (trigger). The app detects your trigger button and logs it; the ring
> itself arrives in Phase 2. See [docs/roadmap.md](docs/roadmap.md).

## Configuration

ClickHub reads `%APPDATA%\com.liamerens.clickhub\config.json` and writes a default one on first
run. Edit it, then choose **Reload config** from the tray menu.

```json
{
  "version": 1,
  "trigger": { "kind": "mouse", "button": "x2", "modifiers": [], "swallow": false }
}
```

- `button`: `"right"`, `"middle"`, `"x1"` (Back) or `"x2"` (Forward). The left button is not
  allowed.
- `modifiers`: any of `"ctrl"`, `"shift"`, `"alt"`, `"win"`. They must all be held, and no others.
- `swallow`: `true` hides the trigger click from the app underneath.
- Keyboard instead: `{ "kind": "keyboard", "shortcut": "Ctrl+Alt+Space" }`.

If the file is invalid, ClickHub keeps running on the last good config, saves a copy of the broken
file next to it, and says so in the tray tooltip. Logs are in
`%LOCALAPPDATA%\com.liamerens.clickhub\logs`.

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
