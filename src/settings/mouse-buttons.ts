import type { Modifier, MouseButton } from "../shared/schema";

export const BUTTON_NAMES: Record<MouseButton, string> = {
  right: "Right button",
  middle: "Middle button (wheel click)",
  x1: "Back side button (X1)",
  x2: "Forward side button (X2)",
};

export const MODIFIER_NAMES: Record<Modifier, string> = {
  ctrl: "Ctrl",
  shift: "Shift",
  alt: "Alt",
  win: "Win",
};

export const MODIFIER_ORDER: readonly Modifier[] = ["ctrl", "shift", "alt", "win"];

/**
 * The trigger button for a DOM `MouseEvent.button`, or `null` for the left button (never allowed:
 * a bad config must never take over the primary button) and anything unknown.
 */
export function buttonFromDom(button: number): MouseButton | null {
  switch (button) {
    case 1:
      return "middle";
    case 2:
      return "right";
    case 3:
      return "x1";
    case 4:
      return "x2";
    default:
      return null;
  }
}

/** The modifiers held during a DOM mouse event, in config order. */
export function modifiersFromEvent(event: {
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}): Modifier[] {
  const held: Record<Modifier, boolean> = {
    ctrl: event.ctrlKey,
    shift: event.shiftKey,
    alt: event.altKey,
    win: event.metaKey,
  };
  return MODIFIER_ORDER.filter((modifier) => held[modifier]);
}
