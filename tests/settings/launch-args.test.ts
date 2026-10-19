import { describe, expect, it } from "vitest";

import { formatArgs, parseArgs } from "../../src/settings/launch-args";

describe("launch arguments", () => {
  it("splits on spaces and keeps quoted spaces", () => {
    expect(parseArgs('--new-window  "C:/My folder" -x')).toEqual([
      "--new-window",
      "C:/My folder",
      "-x",
    ]);
    expect(parseArgs("   ")).toEqual([]);
    expect(parseArgs('""')).toEqual([""]);
  });

  it("quotes only the arguments that need it", () => {
    expect(formatArgs(["--new-window", "C:/My folder", ""])).toBe('--new-window "C:/My folder" ""');
  });

  it("round-trips", () => {
    const args = ["-a", "two words", "x"];
    expect(parseArgs(formatArgs(args))).toEqual(args);
  });
});
