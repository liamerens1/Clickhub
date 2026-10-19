/**
 * Pure motion maths for the ring: easing curves, the open bloom timeline and the graded focus
 * zoom. No DOM, so it is unit-tested in `tests/overlay/motion.test.ts`. Angles are radians,
 * clockwise from the top.
 */

export const clamp01 = (value: number): number => (value < 0 ? 0 : value > 1 ? 1 : value);
export const easeOutCubic = (t: number): number => 1 - (1 - t) ** 3;
export const easeInQuad = (t: number): number => t * t;

const TAU = Math.PI * 2;

/** Shortest distance between two angles, in 0..PI. */
export function angularDistance(a: number, b: number): number {
  const difference = (((a - b) % TAU) + TAU) % TAU;
  return Math.min(difference, TAU - difference);
}

/** Overshoots to ~1.08 before settling at 1: the label "pop". */
export function backOut(t: number): number {
  const overshoot = 1.9;
  const u = t - 1;
  return u * u * ((overshoot + 1) * u + overshoot) + 1;
}

/** The approved "relaxed" bloom (about 300 ms for 8 slots). */
export const BLOOM = {
  /** Delay between neighbouring lines, spreading from the top round both sides. */
  staggerMs: 22,
  lineMs: 150,
  textMs: 130,
  /** A label starts popping when its line is this far grown. */
  textStartsAt: 0.6,
  centreMs: 120,
  closeMs: 100,
} as const;

/** Position in the bloom order: 0 for the slot nearest the top, growing outward. */
export function bloomOrder(angle: number, spacing: number): number {
  return Math.round(angularDistance(angle, 0) / spacing);
}

export interface BloomFrame {
  /** 0..1 how far the line has grown. */
  line: number;
  /** 0..1 label opacity. */
  text: number;
  /** Label scale factor, overshooting slightly above 1 while popping. */
  pop: number;
}

/** Where a slot with bloom position `order` is, `elapsedMs` after the open started. */
export function bloomFrame(order: number, elapsedMs: number): BloomFrame {
  const start = order * BLOOM.staggerMs;
  const line = easeOutCubic(clamp01((elapsedMs - start) / BLOOM.lineMs));
  const textT = clamp01((elapsedMs - start - BLOOM.lineMs * BLOOM.textStartsAt) / BLOOM.textMs);
  return { line, text: easeOutCubic(textT), pop: backOut(textT) };
}

/** Total length of the open animation when the last slot has bloom position `maxOrder`. */
export function bloomDurationMs(maxOrder: number): number {
  return maxOrder * BLOOM.staggerMs + BLOOM.lineMs * BLOOM.textStartsAt + BLOOM.textMs;
}

/** Slots within this many neighbour-spacings of the aim get some zoom. */
export const ZOOM_SPREAD = 1.6;

/**
 * Focus weight 0..1 for a slot at `slotAngle` when aiming at `aimAngle`: 1 straight at it,
 * easing smoothly to 0 at `ZOOM_SPREAD` spacings away. Scaling by `spacing` keeps the zoom the
 * same shape when the ring is squeezed near a screen edge.
 */
export function focusWeight(slotAngle: number, aimAngle: number, spacing: number): number {
  const distance = angularDistance(slotAngle, aimAngle) / spacing;
  if (distance >= ZOOM_SPREAD) return 0;
  return (Math.cos((distance / ZOOM_SPREAD) * Math.PI) + 1) / 2;
}

/** Frame-rate independent exponential smoothing factor for a time constant `tauMs`. */
export function smoothing(dtMs: number, tauMs: number): number {
  return 1 - Math.exp(-dtMs / tauMs);
}
