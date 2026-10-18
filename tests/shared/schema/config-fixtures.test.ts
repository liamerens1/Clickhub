import { readdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { parseConfig } from "../../../src/shared/schema";

// Same fixtures as src-tauri/tests/config.rs. Both parsers must agree on every file.
const fixturesDir = resolve(import.meta.dirname, "../../../fixtures/config");

function fixtures(kind: "valid" | "invalid"): string[] {
  return readdirSync(join(fixturesDir, kind)).filter((name) => name.endsWith(".json"));
}

describe("config schema against shared fixtures", () => {
  it("has fixtures in both folders", () => {
    expect(fixtures("valid").length).toBeGreaterThan(0);
    expect(fixtures("invalid").length).toBeGreaterThan(0);
  });

  it.each(fixtures("valid"))("accepts valid/%s", (name) => {
    const result = parseConfig(readFileSync(join(fixturesDir, "valid", name), "utf8"));
    expect(result).toMatchObject({ ok: true });
  });

  it.each(fixtures("invalid"))("rejects invalid/%s", (name) => {
    const result = parseConfig(readFileSync(join(fixturesDir, "invalid", name), "utf8"));
    expect(result).toMatchObject({ ok: false });
  });
});
