import type { Ink, RingConfig } from "../shared/ipc";

import {
  BLOOM,
  bloomDurationMs,
  bloomFrame,
  bloomOrder,
  clamp01,
  easeInQuad,
  easeOutCubic,
  focusWeight,
  smoothing,
} from "./motion";
import { buildRing, maxLabelWidth, paintSlot, placeSlots, type SlotNodes } from "./ring-view";

/** Focus zoom follows the aim with this time constant (approved in the mockup). */
const FOCUS_TAU_MS = 55;

type Phase = "closed" | "opening" | "open" | "closing";

interface SlotState {
  nodes: SlotNodes;
  focus: number;
  target: number;
  /** Position in the bloom for the current open. */
  order: number;
}

/**
 * Drives the ring with a single `requestAnimationFrame` loop that runs only while something is
 * moving. Writes only `transform` and `opacity` (hard rule 3) and never awaits anything.
 */
export class RingAnimator {
  private slots: SlotState[] = [];
  private config: RingConfig = { slots: [] };
  private phase: Phase = "closed";
  private phaseStart = 0;
  private lastFrame = 0;
  private frame = 0;
  private nearest: number | null = null;
  private spacing = Math.PI / 4;
  private centreX = 0;
  private centreY = 0;
  private bloomMs: number = bloomDurationMs(4);
  private readonly reducedMotion: boolean;

  constructor(
    private readonly root: HTMLElement,
    private readonly spokes: HTMLElement,
    private readonly centreName: HTMLElement,
    private readonly centreHint: HTMLElement,
  ) {
    this.reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    this.root.style.opacity = "0";
  }

  /** Builds the ring for `config`. Returns the widest label at full zoom, in CSS pixels. */
  configure(config: RingConfig): number {
    this.config = config;
    this.slots = buildRing(this.spokes, config).map((nodes) => ({
      nodes,
      focus: 0,
      target: 0,
      order: 0,
    }));
    this.phase = "closed";
    this.paint(performance.now());
    return maxLabelWidth(this.slots.map((slot) => slot.nodes));
  }

  /**
   * Opens at `(cx, cy)` in window CSS pixels with slots at `angles` (radians clockwise from the
   * top). Calls `onFirstFrame` on the first painted frame.
   */
  open(
    cx: number,
    cy: number,
    angles: readonly number[],
    spacing: number,
    radiusScale: number,
    inks: readonly Ink[],
    centreInk: Ink,
    onFirstFrame?: () => void,
  ): void {
    this.root.style.transform = `translate(${cx.toFixed(1)}px, ${cy.toFixed(1)}px)`;
    this.centreX = cx;
    this.centreY = cy;
    placeSlots(
      this.slots.map((slot) => slot.nodes),
      angles,
      radiusScale,
      inks,
    );
    const centre = this.centreName.parentElement;
    if (centre !== null) {
      centre.classList.toggle("ink-dark", centreInk === "dark");
      centre.classList.toggle("ink-light", centreInk === "light");
    }
    this.spacing = spacing;
    let maxOrder = 0;
    for (const slot of this.slots) {
      slot.focus = 0;
      slot.target = 0;
      slot.order = bloomOrder(slot.nodes.angle, spacing);
      maxOrder = Math.max(maxOrder, slot.order);
    }
    this.bloomMs = bloomDurationMs(maxOrder);
    this.setCentre(null);
    this.phase = "opening";
    this.phaseStart = performance.now();
    this.kick();
    if (onFirstFrame !== undefined)
      requestAnimationFrame(() => {
        onFirstFrame();
      });
  }

  aim(angle: number, nearest: number | null): void {
    for (const slot of this.slots) {
      slot.target = nearest === null ? 0 : focusWeight(slot.nodes.angle, angle, this.spacing);
    }
    if (nearest !== this.nearest) this.setCentre(nearest);
    this.kick();
  }

  close(): void {
    if (this.phase === "closed" || this.phase === "closing") return;
    this.phase = "closing";
    this.phaseStart = performance.now();
    this.kick();
  }

  /** One invisible open/close at startup so the first real open is not a cold start. */
  prewarm(): void {
    this.root.classList.add("prewarm");
    const count = this.slots.length;
    const spacing = (Math.PI * 2) / Math.max(1, count);
    this.open(
      0,
      0,
      this.slots.map((_, index) => index * spacing),
      spacing,
      1,
      this.slots.map(() => "light"),
      "light",
    );
    window.setTimeout(() => {
      this.close();
      window.setTimeout(() => {
        this.root.classList.remove("prewarm");
      }, BLOOM.closeMs + 50);
    }, this.bloomMs + 50);
  }

  private setCentre(nearest: number | null): void {
    this.nearest = nearest;
    const slot = nearest === null ? undefined : this.config.slots[nearest];
    this.centreName.textContent = slot?.label ?? "";
    this.centreHint.textContent = slot?.hint ?? "";
    this.keepCentreInView();
  }

  /**
   * Shifts the centre text just enough to stay inside the window when the ring opens near a
   * screen edge. Reads its size once per focus change, never per frame.
   */
  private keepCentreInView(): void {
    const centre = this.centreName.parentElement;
    if (centre === null) return;
    const margin = 8;
    const halfWidth = centre.offsetWidth / 2;
    const halfHeight = centre.offsetHeight / 2;
    const clamp = (value: number, low: number, high: number) =>
      Math.min(Math.max(value, low), Math.max(low, high));
    const shiftX = clamp(
      0,
      halfWidth + margin - this.centreX,
      window.innerWidth - halfWidth - margin - this.centreX,
    );
    const shiftY = clamp(
      0,
      halfHeight + margin - this.centreY,
      window.innerHeight - halfHeight - margin - this.centreY,
    );
    centre.style.transform = `translate(-50%, -50%) translate(${shiftX.toFixed(1)}px, ${shiftY.toFixed(1)}px)`;
  }

  private kick(): void {
    if (this.frame === 0) {
      this.lastFrame = performance.now();
      this.frame = requestAnimationFrame((now) => {
        this.tick(now);
      });
    }
  }

  private tick(now: number): void {
    this.frame = 0;
    const busy = this.paint(now);
    if (busy) {
      this.frame = requestAnimationFrame((next) => {
        this.tick(next);
      });
    }
  }

  /** Paints one frame. Returns true while anything is still moving. */
  private paint(now: number): boolean {
    const dt = Math.max(0, now - this.lastFrame);
    this.lastFrame = now;
    const elapsed = now - this.phaseStart;
    const follow = smoothing(dt, FOCUS_TAU_MS);
    let busy = false;

    let closeLevel = 1;
    if (this.phase === "closing") {
      closeLevel = 1 - easeInQuad(clamp01(elapsed / BLOOM.closeMs));
      if (elapsed >= BLOOM.closeMs) this.phase = "closed";
      else busy = true;
    }
    if (this.phase === "closed") closeLevel = 0;

    const opening = this.phase === "opening";
    const rootLevel = opening || this.phase === "open" ? 1 : closeLevel;
    this.root.style.opacity = rootLevel.toFixed(3);
    const centre = opening ? easeOutCubic(clamp01(elapsed / BLOOM.centreMs)) : rootLevel;
    const centreEl = this.centreName.parentElement;
    if (centreEl !== null) centreEl.style.opacity = centre.toFixed(3);

    for (const slot of this.slots) {
      const delta = slot.target - slot.focus;
      if (Math.abs(delta) > 0.001) {
        slot.focus += delta * follow;
        busy = true;
      } else {
        slot.focus = slot.target;
      }
      let line = 1;
      let text = 1;
      let pop = 1;
      if (opening && !this.reducedMotion) {
        ({ line, text, pop } = bloomFrame(slot.order, elapsed));
      } else if (opening) {
        line = text = easeOutCubic(clamp01(elapsed / BLOOM.centreMs));
      } else if (this.phase === "closing" || this.phase === "closed") {
        line = this.reducedMotion ? 1 : closeLevel;
        text = closeLevel;
        pop = 0.9 + 0.1 * closeLevel;
      }
      paintSlot(slot.nodes, slot.focus, line, text, pop, this.reducedMotion);
    }

    if (opening) {
      const duration = this.reducedMotion ? BLOOM.centreMs : this.bloomMs;
      if (elapsed >= duration) this.phase = "open";
      else busy = true;
    }
    return busy;
  }
}
