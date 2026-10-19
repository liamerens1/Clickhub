import "./overlay.css";

import {
  emitOverlayExtent,
  emitOverlayFrame,
  emitOverlayReady,
  onRingAim,
  onRingClose,
  onRingConfig,
  onRingOpen,
} from "../shared/ipc";

import { RingAnimator } from "./ring-animator";

function required(id: string): HTMLElement {
  const element = document.getElementById(id);
  if (element === null) throw new Error(`overlay element #${id} is missing from index.html`);
  return element;
}

const animator = new RingAnimator(
  required("ring"),
  required("spokes"),
  required("centre-name"),
  required("centre-hint"),
);

let openSeq = 0;
let warmed = false;

await onRingConfig((config) => {
  void emitOverlayExtent(animator.configure(config));
  if (!warmed && config.slots.length > 0) {
    warmed = true;
    animator.prewarm();
  }
});

await onRingOpen((open) => {
  openSeq = open.seq;
  animator.open(
    open.cx,
    open.cy,
    open.angles,
    open.spacing,
    open.radiusScale,
    open.inks,
    open.centreInk,
    () => {
      void emitOverlayFrame(open.seq, Date.now() - open.emittedMs);
    },
  );
});

await onRingAim((aim) => {
  if (aim.seq === openSeq) animator.aim(aim.angle, aim.nearest);
});

await onRingClose((close) => {
  if (close.seq === openSeq) animator.close();
});

await emitOverlayReady();
