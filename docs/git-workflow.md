# Git workflow

## Branches

| Branch                             | Purpose                                                                   |
| ---------------------------------- | ------------------------------------------------------------------------- |
| `main`                             | Stable milestones only. Updated from `dev` at phase boundaries.           |
| `dev`                              | Integration branch and the GitHub default branch.                         |
| `feat/<short-description>/<phase>` | Feature work for a phase, e.g. `feat/foundation/0`, `feat/hook-thread/1`. |
| `fix/<short-description>`          | Bug fixes, e.g. `fix/tray-icon-leak`.                                     |

- **Never commit directly to `main` or `dev`.** All work goes through a PR.
- `dev` was branched straight from `main`'s initial commit (no orphan history).
- Descriptions are kebab-case.
- Always include the phase number on `feat/` branches: git cannot hold both `feat/x` and
  `feat/x/2`, because a branch name cannot also be a directory.
- Branch from the latest `dev`.

## Commits

- Imperative mood, short subject line (about 72 characters at most): "Add tray menu with Quit".
- The body explains **why**, not just what.
- **Ask before running `git commit`.** Never add `Co-Authored-By` trailers, even if tooling suggests them.
- Append a dated entry to `docs/memory/project_memory.md` before every commit.
- Push after each meaningful unit of work, not one blob per session.
- Don't commit generated output (`dist/`, `src-tauri/target/`, `src-tauri/gen/schemas/`) or
  `docs/memory/`.

## Pull requests

- Target `dev`.
- Title: `feat: short description` or `fix: short description`. Use `chore:` or `docs:` only for
  tooling-only or docs-only PRs.
- The description uses `.github/pull_request_template.md` in full:
  - **Summary** (+ phase)
  - **What changed**, grouped by area
  - **Why**, linking the memory entry by date and title
  - **How to verify**: concrete steps with expected results, never just "tested locally"
  - **Known issues / follow-ups**
  - **Checklist**
- **No attribution lines.** Never add co-author or "Generated with Claude Code" lines to a PR
  description, even if tooling suggests them.
- CI must be green before merge.
- Say what was live-tested and what was not.

## Phase boundaries

At the end of each phase:

1. Stop and hand over the PR title and description for review.
2. After the phase PR is merged into `dev`, `dev` is merged into `main` and tagged `v0.<phase>.0`
   (`v1.0.0` at the end of Phase 7).
3. Update `docs/roadmap.md` and the `CLAUDE.md` phase table.

## Side effects outside the repo

Explain what will happen and wait for approval before:

- changing GitHub settings (default branch, protection rules, secrets)
- creating releases or tags on the remote
- installing global tools
- anything that sends input to other apps, deploys, or spends money
