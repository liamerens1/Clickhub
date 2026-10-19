import { describe, expect, it } from "vitest";

import { actionHint } from "../../src/settings/action-hint";

describe("actionHint", () => {
  it("shows a shortcut's keys", () => {
    expect(actionHint({ kind: "shortcut", keys: "Win+Shift+S" })).toBe("Win+Shift+S");
  });

  it("shows the file name of a launch path", () => {
    expect(actionHint({ kind: "launch", path: "C:/Program Files/App/app.exe", args: [] })).toBe(
      "Open app.exe",
    );
    expect(actionHint({ kind: "launch", path: "explorer.exe", args: [] })).toBe(
      "Open explorer.exe",
    );
  });

  it("shows the host of a link", () => {
    expect(actionHint({ kind: "url", url: "https://www.google.com/search?q=x" })).toBe(
      "Open www.google.com",
    );
  });
});
