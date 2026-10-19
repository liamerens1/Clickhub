import { describe, expect, it } from "vitest";

import { buttonFromDom, modifiersFromEvent } from "../../src/settings/mouse-buttons";

describe("buttonFromDom", () => {
  it("maps DOM buttons to trigger buttons", () => {
    expect(buttonFromDom(1)).toBe("middle");
    expect(buttonFromDom(2)).toBe("right");
    expect(buttonFromDom(3)).toBe("x1");
    expect(buttonFromDom(4)).toBe("x2");
  });

  it("never offers the left button", () => {
    expect(buttonFromDom(0)).toBeNull();
    expect(buttonFromDom(7)).toBeNull();
  });
});

describe("modifiersFromEvent", () => {
  it("lists held modifiers in config order", () => {
    expect(
      modifiersFromEvent({ ctrlKey: false, shiftKey: true, altKey: false, metaKey: true }),
    ).toEqual(["shift", "win"]);
    expect(
      modifiersFromEvent({ ctrlKey: false, shiftKey: false, altKey: false, metaKey: false }),
    ).toEqual([]);
  });
});
