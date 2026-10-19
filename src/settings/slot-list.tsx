import { type PointerEvent, type ReactNode, useRef, useState } from "react";

import type { Slot } from "../shared/schema";

import { actionHint } from "./action-hint";
import { Icon } from "./icon";

interface SlotListProps {
  slots: readonly Slot[];
  /** The row whose editor is open. */
  open: number | null;
  onToggle: (index: number) => void;
  onHover: (index: number | null) => void;
  /** Moves the slot at `from` to `to`. */
  onMove: (from: number, to: number) => void;
  /** Called when a drag starts (the page closes the open editor so rows line up). */
  onDragStart: () => void;
  renderEditor: (index: number) => ReactNode;
}

interface Drag {
  from: number;
  to: number;
  startY: number;
  lastY: number;
  pitch: number;
  frame: number;
}

/** Gap between rows in CSS (`.slot-list { gap }`). */
const ROW_GAP = 6;

/**
 * The ring's slots, clockwise from the top. Drag a row by its grip to reorder (or Alt+Up and
 * Alt+Down). While dragging, rows move only with `transform`, written straight to the DOM once
 * per frame, so a drag never re-renders React and holds the display's frame rate.
 */
export function SlotList({
  slots,
  open,
  onToggle,
  onHover,
  onMove,
  onDragStart,
  renderEditor,
}: SlotListProps) {
  const [sorting, setSorting] = useState(false);
  const list = useRef<HTMLUListElement>(null);
  const rows = useRef<(HTMLLIElement | null)[]>([]);
  const drag = useRef<Drag | null>(null);

  const apply = () => {
    const state = drag.current;
    if (state === null) return;
    state.frame = 0;
    const dy = state.lastY - state.startY;
    const last = slots.length - 1;
    const to = Math.min(last, Math.max(0, state.from + Math.round(dy / state.pitch)));
    state.to = to;
    rows.current.forEach((row, index) => {
      if (row === null) return;
      let shift = 0;
      if (index === state.from) shift = dy;
      else if (state.from < to && index > state.from && index <= to) shift = -state.pitch;
      else if (to < state.from && index >= to && index < state.from) shift = state.pitch;
      row.style.transform = shift === 0 ? "" : `translateY(${String(shift)}px)`;
    });
  };

  const start = (event: PointerEvent<HTMLButtonElement>, index: number) => {
    if (event.button !== 0) return;
    const head = rows.current[index]?.querySelector<HTMLElement>(".slot-head");
    if (head == null) return;
    event.preventDefault();
    try {
      event.currentTarget.setPointerCapture(event.pointerId);
    } catch {
      // Synthetic or already-released pointers cannot be captured; the drag still works while
      // the pointer stays over the grip.
    }
    // Rows are their head plus a 1 px border top and bottom once every editor is closed.
    const pitch = head.getBoundingClientRect().height + 2 + ROW_GAP;
    drag.current = {
      from: index,
      to: index,
      startY: event.clientY,
      lastY: event.clientY,
      pitch,
      frame: 0,
    };
    rows.current[index]?.classList.add("is-dragged");
    onDragStart();
    setSorting(true);
  };

  const move = (event: PointerEvent<HTMLButtonElement>) => {
    const state = drag.current;
    if (state === null) return;
    state.lastY = event.clientY;
    if (state.frame === 0) state.frame = requestAnimationFrame(apply);
  };

  const end = () => {
    const state = drag.current;
    if (state === null) return;
    cancelAnimationFrame(state.frame);
    drag.current = null;
    // Drop the transition first so rows snap into their new places with no animation back.
    list.current?.classList.remove("is-sorting");
    for (const row of rows.current) {
      if (row === null) continue;
      row.style.transform = "";
      row.classList.remove("is-dragged");
    }
    setSorting(false);
    if (state.to !== state.from) onMove(state.from, state.to);
  };

  /** Moves a slot one place with the keyboard; focus follows it to its new row. */
  const nudge = (index: number, by: number, focusSelector: string) => {
    const to = index + by;
    if (to < 0 || to >= slots.length) return;
    onMove(index, to);
    requestAnimationFrame(() => {
      rows.current[to]?.querySelector<HTMLElement>(focusSelector)?.focus();
    });
  };

  return (
    <ul
      ref={list}
      className={sorting ? "slot-list is-sorting" : "slot-list"}
      onPointerLeave={() => {
        onHover(null);
      }}
    >
      {slots.map((slot, index) => {
        const isOpen = open === index;
        return (
          <li
            // Rows are positions in the ring; the slot at a position is what changes.
            key={index}
            ref={(row) => {
              rows.current[index] = row;
            }}
            className={isOpen ? "slot-row is-open" : "slot-row"}
            onPointerEnter={() => {
              onHover(index);
            }}
          >
            <div
              className="slot-head"
              role="button"
              tabIndex={0}
              aria-expanded={isOpen}
              onClick={() => {
                onToggle(index);
              }}
              onKeyDown={(event) => {
                if (event.target !== event.currentTarget) return;
                if (event.altKey && (event.key === "ArrowUp" || event.key === "ArrowDown")) {
                  event.preventDefault();
                  nudge(index, event.key === "ArrowUp" ? -1 : 1, ".slot-head");
                } else if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  onToggle(index);
                }
              }}
            >
              <button
                type="button"
                className="slot-grip"
                aria-label={`Move ${slot.label}. Drag, or use the arrow keys.`}
                onPointerDown={(event) => {
                  start(event, index);
                }}
                onPointerMove={move}
                onPointerUp={end}
                onPointerCancel={end}
                onLostPointerCapture={end}
                onClick={(event) => {
                  event.stopPropagation();
                }}
                onKeyDown={(event) => {
                  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
                    event.preventDefault();
                    event.stopPropagation();
                    nudge(index, event.key === "ArrowUp" ? -1 : 1, ".slot-grip");
                  }
                }}
              >
                <Icon name="grip" />
              </button>
              <span className="slot-label">{slot.label}</span>
              <span className="slot-hint chip">{actionHint(slot.action)}</span>
              <span className="slot-chevron">
                <Icon name="chevron" />
              </span>
            </div>
            {isOpen && renderEditor(index)}
          </li>
        );
      })}
    </ul>
  );
}
