import { useState } from "react";

import type { Slot } from "../shared/schema";

import { SlotList } from "./slot-list";

/**
 * Development only (`?perf=60`): the slot list with many synthetic rows, to measure scrolling
 * and dragging against the 60 fps target with 50+ actions. A ring holds at most 12 slots, so
 * this is the only way to load the list that heavily before profiles arrive.
 */
export function PerfHarness({ rows }: { rows: number }) {
  const [slots, setSlots] = useState<Slot[]>(() =>
    Array.from({ length: rows }, (_, index) => ({
      label: `Action ${String(index + 1)}`,
      action:
        index % 3 === 0
          ? { kind: "shortcut", keys: "Ctrl+Shift+S" }
          : index % 3 === 1
            ? { kind: "launch", path: "C:\\Windows\\System32\\notepad.exe", args: [] }
            : { kind: "url", url: "https://example.com/page" },
    })),
  );
  const [open, setOpen] = useState<number | null>(null);
  return (
    <div className="app">
      <main className="content" data-perf-scroller>
        <div className="content-inner">
          <h1 className="page-title">Perf harness: {rows} rows</h1>
          <SlotList
            slots={slots}
            open={open}
            onToggle={(index) => {
              setOpen((current) => (current === index ? null : index));
            }}
            onHover={() => undefined}
            onMove={(from, to) => {
              setSlots((list) => {
                const next = [...list];
                const [moved] = next.splice(from, 1);
                if (moved !== undefined) next.splice(to, 0, moved);
                return next;
              });
            }}
            onDragStart={() => {
              setOpen(null);
            }}
            renderEditor={() => <p className="field-note">Editor</p>}
          />
        </div>
      </main>
    </div>
  );
}
