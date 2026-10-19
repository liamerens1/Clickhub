/**
 * Writes `latest.json`, the update manifest the Tauri updater reads, for one release.
 * Run by `.github/workflows/release.yml` after `pnpm tauri build`:
 *
 *   node scripts/release-manifest.ts <tag> <out-file>
 *
 * It checks that the tag matches the app version, finds the NSIS installer and its updater
 * signature, and points the download at this tag's GitHub Release asset. Fails loudly on any
 * mismatch so a broken manifest is never published.
 */
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const REPO = "liamerens1/Clickhub";
const BUNDLE_DIR = join("src-tauri", "target", "release", "bundle", "nsis");

function fail(message: string): never {
  console.error(`release-manifest: ${message}`);
  process.exit(1);
}

const [tag, outFile] = process.argv.slice(2);
if (tag === undefined || outFile === undefined) fail("usage: <tag> <out-file>");

const config: unknown = JSON.parse(readFileSync(join("src-tauri", "tauri.conf.json"), "utf8"));
const version =
  typeof config === "object" && config !== null && "version" in config ? config.version : null;
if (typeof version !== "string") fail("tauri.conf.json has no version");
if (tag !== `v${version}`) fail(`tag ${tag} does not match the app version ${version}`);

const installer = readdirSync(BUNDLE_DIR).find((name) => name.endsWith("-setup.exe"));
if (installer === undefined) fail(`no NSIS installer in ${BUNDLE_DIR}`);
if (!installer.includes(`_${version}_`)) fail(`${installer} is not version ${version}`);
const signature = readFileSync(join(BUNDLE_DIR, `${installer}.sig`), "utf8").trim();
if (signature === "") fail(`${installer}.sig is empty`);

const platform = {
  signature,
  url: `https://github.com/${REPO}/releases/download/${tag}/${encodeURIComponent(installer)}`,
};
const manifest = {
  version,
  notes: `See https://github.com/${REPO}/releases/tag/${tag}`,
  pub_date: new Date().toISOString(),
  // The updater looks for "<target>-<installer>" first, then "<target>".
  platforms: { "windows-x86_64-nsis": platform, "windows-x86_64": platform },
};
writeFileSync(outFile, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`release-manifest: wrote ${outFile} for ${installer}`);
