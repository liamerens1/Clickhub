import { type ReactNode, useEffect, useRef, useState } from "react";

import type { Slot } from "../shared/schema";

import { actionHint } from "./action-hint";
import { PRESET_GROUPS } from "./presets";

interface PresetMenuProps {
  /** Renders the button that opens the menu. */
  trigger: (props: { open: boolean; toggle: () => void }) => ReactNode;
  onPick: (slot: Slot) => void;
  note?: string;
  /** Open upwards (for buttons near the bottom of the page). */
  up?: boolean;
}

/** A popover listing the ready-made slots. Closes on pick, Esc or a click elsewhere. */
export function PresetMenu({ trigger, onPick, note, up = false }: PresetMenuProps) {
  const [open, setOpen] = useState(false);
  const anchor = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const close = (event: Event) => {
      if (
        event instanceof KeyboardEvent ? event.key === "Escape" : !inside(anchor.current, event)
      ) {
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", close);
    document.addEventListener("keydown", close);
    return () => {
      document.removeEventListener("pointerdown", close);
      document.removeEventListener("keydown", close);
    };
  }, [open]);

  return (
    <div className="popover-anchor" ref={anchor}>
      {trigger({
        open,
        toggle: () => {
          setOpen((value) => !value);
        },
      })}
      {open && (
        <div className={up ? "popover is-up" : "popover"} role="menu">
          {note !== undefined && <p className="popover-note">{note}</p>}
          {PRESET_GROUPS.map((group) => (
            <div key={group.name} role="group" aria-label={group.name}>
              <div className="popover-group">{group.name}</div>
              {group.slots.map((slot) => (
                <button
                  key={slot.label}
                  type="button"
                  role="menuitem"
                  className="popover-item"
                  onClick={() => {
                    setOpen(false);
                    onPick(slot);
                  }}
                >
                  <span>{slot.label}</span>
                  <span className="popover-item-hint">{actionHint(slot.action)}</span>
                </button>
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function inside(element: HTMLElement | null, event: Event): boolean {
  return element !== null && event.target instanceof Node && element.contains(event.target);
}
