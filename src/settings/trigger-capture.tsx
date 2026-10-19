import { useEffect, useState } from "react";

import { setTriggerCapture } from "../shared/ipc";
import type { Modifier, MouseButton } from "../shared/schema";

import { buttonFromDom, modifiersFromEvent } from "./mouse-buttons";

interface TriggerCaptureProps {
  onCaptured: (button: MouseButton, modifiers: Modifier[]) => void;
  onCancel: () => void;
}

/**
 * "Press the button you want." While shown, the current trigger is switched off (so the press
 * reaches this window instead of opening the ring) and the next mouse button pressed anywhere in
 * the window becomes the trigger, with any Ctrl/Shift/Alt/Win held at the time. Esc, Cancel or
 * leaving the window stops listening and switches the trigger back on.
 */
export function TriggerCapture({ onCaptured, onCancel }: TriggerCaptureProps) {
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    void setTriggerCapture(true);
    const onDown = (event: MouseEvent) => {
      if (event.target instanceof Element && event.target.closest("[data-capture-ignore]")) {
        return;
      }
      event.preventDefault();
      const button = buttonFromDom(event.button);
      if (button === null) {
        setProblem(
          "The left button can't open the ring: it would stop clicking everywhere. Try another one.",
        );
        return;
      }
      onCaptured(button, modifiersFromEvent(event));
    };
    // Stop the side buttons navigating and the right button opening a menu.
    const block = (event: Event) => {
      event.preventDefault();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("mousedown", onDown, true);
    window.addEventListener("mouseup", block, true);
    window.addEventListener("auxclick", block, true);
    window.addEventListener("contextmenu", block, true);
    window.addEventListener("keydown", onKey);
    window.addEventListener("blur", onCancel);
    return () => {
      window.removeEventListener("mousedown", onDown, true);
      window.removeEventListener("mouseup", block, true);
      window.removeEventListener("auxclick", block, true);
      window.removeEventListener("contextmenu", block, true);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("blur", onCancel);
      void setTriggerCapture(false);
    };
  }, [onCaptured, onCancel]);

  return (
    <div className="capture-panel" role="status" aria-live="polite">
      <span className="listening-dot" />
      <p className="capture-title">Press the mouse button you want</p>
      <p className="capture-hint">
        Anywhere in this window. Hold Ctrl, Shift, Alt or Win as well if you want them.
      </p>
      {problem !== null && <span className="field-error">{problem}</span>}
      <button type="button" className="btn" data-capture-ignore onClick={onCancel}>
        Cancel
      </button>
    </div>
  );
}
