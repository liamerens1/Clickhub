import type { Ink, RingConfig } from "../shared/ipc";

/**
 * Geometry from the approved mockup, in CSS pixels. `labelRadius` and `labelPush` must match
 * `LABEL_RADIUS` and `LABEL_PUSH` in `src-tauri/src/core/ring_layout.rs`.
 */
export const GEOMETRY = {
  /** Lines start here, leaving the centre empty. */
  innerRadius: 72,
  outerRadius: 132,
  labelRadius: 150,
  /** How much further the focused line reaches (fraction of its length). */
  lineGrow: 0.16,
  /** How far the focused label slides outward. */
  labelPush: 20,
  /** Labels are rendered at this zoom and scaled down at rest, so zooming never blurs. */
  maxZoom: 1.32,
  /**
   * A label is centred on its point only within ~4.6 degrees of straight up or down; otherwise
   * it hangs away from the centre, so neighbours never collide. Matches `CENTRED_BELOW` in Rust.
   */
  centredBelow: 0.08,
} as const;

/** One slot's DOM nodes and its geometry for the current open. */
export interface SlotNodes {
  line: HTMLDivElement;
  label: HTMLDivElement;
  /** Radians clockwise from the top. */
  angle: number;
  /** Screen-space unit vector: x grows right, y grows down. */
  dx: number;
  dy: number;
  /** Horizontal anchor: 0 = text starts at the point, -50 = centred, -100 = ends at it. */
  anchorPercent: 0 | -50 | -100;
  /** Lines and labels reach this much further out when the ring is squeezed. */
  radiusScale: number;
}

/** Builds the ring's DOM inside `container`. Called on `ring:config`, never per frame. */
export function buildRing(container: HTMLElement, config: RingConfig): SlotNodes[] {
  container.replaceChildren();
  return config.slots.map((slot) => {
    const line = document.createElement("div");
    line.className = "line";
    line.style.width = `${String(GEOMETRY.outerRadius - GEOMETRY.innerRadius)}px`;

    const label = document.createElement("div");
    label.className = "label";
    label.textContent = slot.label;

    container.append(line, label);
    return { line, label, angle: 0, dx: 0, dy: -1, anchorPercent: -50, radiusScale: 1 };
  });
}

/** Widest label at full zoom (labels are rendered at full zoom size), in CSS pixels. */
export function maxLabelWidth(nodes: SlotNodes[]): number {
  return nodes.reduce((widest, slot) => Math.max(widest, slot.label.offsetWidth), 0);
}

/**
 * Places each slot at `angles[i]` for this open. Labels on the right hang to the right of their
 * point, labels on the left to the left, and only near-vertical ones are centred. Clears any ink
 * from the previous open (the theme colour applies until `ring:ink` arrives). Runs once per open.
 */
export function placeSlots(
  nodes: SlotNodes[],
  angles: readonly number[],
  radiusScale: number,
): void {
  for (const [index, slot] of nodes.entries()) {
    const angle = angles[index] ?? 0;
    slot.angle = angle;
    slot.dx = Math.sin(angle);
    slot.dy = -Math.cos(angle);
    slot.radiusScale = radiusScale;
    slot.anchorPercent = Math.abs(slot.dx) < GEOMETRY.centredBelow ? -50 : slot.dx > 0 ? 0 : -100;
    slot.label.style.transformOrigin = `${String(-slot.anchorPercent)}% 50%`;
    for (const element of [slot.line, slot.label]) {
      element.classList.remove("ink-dark", "ink-light");
    }
  }
}

/** Applies dark or light ink to `element`. */
export function applyInk(element: HTMLElement, ink: Ink): void {
  element.classList.toggle("ink-dark", ink === "dark");
  element.classList.toggle("ink-light", ink === "light");
}

/** Applies `inks[i]` to slot `i`'s line and label. */
export function inkSlots(nodes: SlotNodes[], inks: readonly Ink[]): void {
  for (const [index, slot] of nodes.entries()) {
    const ink = inks[index];
    if (ink === undefined) continue;
    applyInk(slot.line, ink);
    applyInk(slot.label, ink);
  }
}

/**
 * Writes one slot's transforms. `focus` is the 0..1 zoom weight; `line`, `text` and `pop` come
 * from the bloom (1 when fully open). Only `transform` and `opacity` change.
 */
export function paintSlot(
  nodes: SlotNodes,
  focus: number,
  line: number,
  text: number,
  pop: number,
  reducedMotion: boolean,
): void {
  // CSS rotate() is clockwise from the +x axis; our angles are clockwise from the top.
  const rotateDeg = (nodes.angle * 180) / Math.PI - 90;
  const grow = reducedMotion ? 1 : 1 + focus * GEOMETRY.lineGrow;
  // When squeezed, lines reach further out (the empty centre stays the same size).
  const baseLength = GEOMETRY.outerRadius - GEOMETRY.innerRadius;
  const stretch = (GEOMETRY.outerRadius * nodes.radiusScale - GEOMETRY.innerRadius) / baseLength;
  const lineScale = stretch * (reducedMotion ? 1 : grow * line);
  nodes.line.style.transform = `rotate(${rotateDeg.toFixed(3)}deg) translateX(${(
    GEOMETRY.innerRadius * grow
  ).toFixed(2)}px) scaleX(${lineScale.toFixed(4)})`;
  nodes.line.style.opacity = ((0.3 + focus * 0.65) * (line > 0 ? 1 : 0)).toFixed(3);

  const push = reducedMotion ? 0 : focus * GEOMETRY.labelPush;
  const radius = GEOMETRY.labelRadius * nodes.radiusScale + push;
  const zoom = reducedMotion
    ? 1 / GEOMETRY.maxZoom
    : (1 + focus * (GEOMETRY.maxZoom - 1)) / GEOMETRY.maxZoom;
  const scale = reducedMotion ? zoom : zoom * (0.55 + 0.45 * pop);
  nodes.label.style.transform = `translate(${(radius * nodes.dx).toFixed(2)}px, ${(
    radius * nodes.dy
  ).toFixed(2)}px) translate(${String(nodes.anchorPercent)}%, -50%) scale(${scale.toFixed(4)})`;
  nodes.label.style.opacity = ((0.55 + focus * 0.45) * text).toFixed(3);
}
