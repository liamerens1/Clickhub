import { describe, expect, it } from "vitest";

import {
  angularDistance,
  backOut,
  BLOOM,
  bloomDurationMs,
  bloomFrame,
  bloomOrder,
  focusWeight,
  smoothing,
} from "../../src/overlay/motion";

const TAU = Math.PI * 2;
const EIGHTH = TAU / 8;

describe("angularDistance", () => {
  it("wraps round the top", () => {
    expect(angularDistance(0.1, TAU - 0.1)).toBeCloseTo(0.2);
    expect(angularDistance(0, Math.PI)).toBeCloseTo(Math.PI);
  });
});

describe("focusWeight", () => {
  it("is strongest at the aim and eases to zero at the spread", () => {
    expect(focusWeight(2 * EIGHTH, 2 * EIGHTH, EIGHTH)).toBeCloseTo(1);
    const neighbour = focusWeight(3 * EIGHTH, 2 * EIGHTH, EIGHTH);
    expect(neighbour).toBeGreaterThan(0.2);
    expect(neighbour).toBeLessThan(0.5);
    expect(focusWeight(4 * EIGHTH, 2 * EIGHTH, EIGHTH)).toBe(0);
  });

  it("is continuous between slots", () => {
    const between = 2.5 * EIGHTH;
    expect(focusWeight(2 * EIGHTH, between, EIGHTH)).toBeCloseTo(
      focusWeight(3 * EIGHTH, between, EIGHTH),
    );
  });

  it("wraps round the top of the ring", () => {
    expect(focusWeight(0, TAU - 0.2 * EIGHTH, EIGHTH)).toBeGreaterThan(0.9);
    expect(focusWeight(7 * EIGHTH, 0.2 * EIGHTH, EIGHTH)).toBeGreaterThan(0.1);
  });

  it("keeps the same shape when slots are squeezed together", () => {
    const tight = EIGHTH / 2;
    expect(focusWeight(3 * tight, 2 * tight, tight)).toBeCloseTo(
      focusWeight(3 * EIGHTH, 2 * EIGHTH, EIGHTH),
    );
  });
});

describe("bloom", () => {
  it("spreads from the top round both sides", () => {
    const orders = [0, 1, 2, 3, 4, 5, 6, 7].map((i) => bloomOrder(i * EIGHTH, EIGHTH));
    expect(orders).toEqual([0, 1, 2, 3, 4, 3, 2, 1]);
  });

  it("grows each line before its label pops in", () => {
    const early = bloomFrame(0, BLOOM.lineMs * 0.3);
    expect(early.line).toBeGreaterThan(0);
    expect(early.text).toBe(0);
    const done = bloomFrame(0, bloomDurationMs(0));
    expect(done.line).toBeCloseTo(1);
    expect(done.text).toBeCloseTo(1);
    expect(done.pop).toBeCloseTo(1);
  });

  it("starts later slots later", () => {
    const t = BLOOM.staggerMs * 2;
    expect(bloomFrame(0, t).line).toBeGreaterThan(bloomFrame(4, t).line);
  });

  it("lasts about 300 ms for 8 slots (the approved relaxed timing)", () => {
    expect(bloomDurationMs(4)).toBeGreaterThan(250);
    expect(bloomDurationMs(4)).toBeLessThan(320);
  });

  it("pops past 1 before settling", () => {
    const peak = Math.max(...Array.from({ length: 50 }, (_, i) => backOut(i / 49)));
    expect(peak).toBeGreaterThan(1.03);
    expect(backOut(1)).toBeCloseTo(1);
    expect(backOut(0)).toBeCloseTo(0);
  });
});

describe("smoothing", () => {
  it("does not depend on frame rate", () => {
    const at60 = 1 - (1 - smoothing(1000 / 60, 55)) ** 6;
    const at180 = 1 - (1 - smoothing(1000 / 180, 55)) ** 18;
    expect(at60).toBeCloseTo(at180, 6);
  });
});
