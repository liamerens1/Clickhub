# Releasing

How a ClickHub version gets from `dev` to users. Installers are built by GitHub Actions; nothing
reaches users until a person publishes the draft release.

## One-time setup: the updater key

Updates are verified with a minisign key pair. The **public** key is in
`src-tauri/tauri.conf.json` (`plugins.updater.pubkey`) and ships inside every build. The
**private** key signs each release and lives only on the owner's machine and in two GitHub
secrets. It is never committed and never pasted into a chat.

1. Generate it in Command Prompt (choose a password when asked):

   ```
   pnpm tauri signer generate -w "%USERPROFILE%\.tauri\clickhub-updater.key"
   ```

2. Put the contents of `%USERPROFILE%\.tauri\clickhub-updater.key.pub` into `pubkey`.
3. Store the private key and its password as repository secrets:

   ```
   gh secret set TAURI_SIGNING_PRIVATE_KEY < "%USERPROFILE%\.tauri\clickhub-updater.key"
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
   ```

4. Back the key file up somewhere safe. **Losing it means installed copies can never update
   again** (they only accept updates signed by this key); a new key needs a manual reinstall.

## Each release

1. At a phase boundary the phase PR is merged into `dev`, then `dev` into `main` (release PR).
2. The app version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` is
   already `0.<phase>.0` (see `docs/git-workflow.md`).
3. Tag the merge commit on `main` and push the tag (ask first):

   ```
   git tag -a v0.4.0 <merge-commit> -m "v0.4.0: Phase 4 - Lifecycle"
   git push origin v0.4.0
   ```

4. `.github/workflows/release.yml` checks the tag matches the version, runs `pnpm tauri build`
   with the signing secrets, writes `latest.json` (`scripts/release-manifest.ts`) and creates a
   **draft** release with `ClickHub_<version>_x64-setup.exe`, its `.sig` and `latest.json`.
5. Check the draft (download and install the setup exe on a test account if the change is big),
   then **Publish**. From that moment installed copies find it within a day (or on Check now).

## Test releases

A tag with a suffix (`v0.4.0-rc.1`) makes a **pre-release**. GitHub's `releases/latest` never
points at a pre-release, so normal installs ignore it. To test updating against one, start an
installed older copy with the endpoint override, from Command Prompt:

```
set CLICKHUB_UPDATE_ENDPOINT=https://github.com/liamerens1/Clickhub/releases/download/v0.4.0-rc.1/latest.json
"%LOCALAPPDATA%\ClickHub\clickhub.exe"
```

The signature is checked exactly as for a real release. The version in `tauri.conf.json` must
match the tag, so set it to the rc version on the branch being tested and back afterwards.

## Not code-signed: SmartScreen

The installer is not signed with a code-signing certificate (parked until one exists), so
Windows SmartScreen shows "Windows protected your PC" the first time it runs. Users choose
**More info**, then **Run anyway**. Updates installed by ClickHub itself run the installer
silently and do not show this. The updater's own signature check (above) is what protects
updates; it is separate from code signing.
