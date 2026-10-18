## Summary

<!-- One or two sentences. -->

**Phase:** <!-- e.g. 0 — Foundation -->

## What changed

<!-- Group by area. Delete areas that don't apply. -->

### Rust core (`src-tauri/`)

-

### Frontend (`src/`)

-

### Docs and rules

-

### Tooling and CI

-

## Why

<!-- The reasoning. Link the memory entry: "docs/memory/project_memory.md: YYYY-MM-DD: <title>". -->

## How to verify

<!-- Concrete steps and expected results. Never just "tested locally". -->

1.

**Live-tested:**

**Not live-tested:**

## Known issues / follow-ups

-

## Checklist

- [ ] Memory entry written in `docs/memory/project_memory.md`
- [ ] `docs/coding-guidelines.md` followed (boundaries, naming, no `any`, no stray `unwrap`/`expect`)
- [ ] `pnpm check` passes (lint, format, typecheck, tests)
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] Project hard rules in `CLAUDE.md` respected (smoothness, overlay, hook, fail open)
- [ ] `docs/roadmap.md` and the `CLAUDE.md` phase table updated if status changed
- [ ] Targets `dev`
