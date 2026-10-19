import { useState } from "react";

import { type Config, MAX_SLOTS, MIN_SLOTS, type Slot } from "../shared/schema";

import { Icon } from "./icon";
import { PresetMenu } from "./preset-menu";
import { RingPreview } from "./ring-preview";
import { SlotEditor } from "./slot-editor";
import { SlotList } from "./slot-list";

interface RingPageProps {
  config: Config;
  commit: (config: Config) => void;
}

/** Where an open row ends up after the slot at `from` moves to `to`. */
function followMove(open: number | null, from: number, to: number): number | null {
  if (open === null) return null;
  if (open === from) return to;
  if (from < open && open <= to) return open - 1;
  if (to <= open && open < from) return open + 1;
  return open;
}

/** The ring editor: the slot list beside a live preview. */
export function RingPage({ config, commit }: RingPageProps) {
  const [open, setOpen] = useState<number | null>(null);
  const [hover, setHover] = useState<number | null>(null);
  const { slots } = config.ring;

  const setSlots = (next: Slot[]) => {
    commit({ ...config, ring: { ...config.ring, slots: next } });
  };

  const move = (from: number, to: number) => {
    const next = [...slots];
    const [moved] = next.splice(from, 1);
    if (moved === undefined) return;
    next.splice(to, 0, moved);
    setOpen(followMove(open, from, to));
    setSlots(next);
  };

  const full = slots.length >= MAX_SLOTS;

  return (
    <div className="ring-layout">
      <div>
        <SlotList
          slots={slots}
          open={open}
          onToggle={(index) => {
            setOpen((current) => (current === index ? null : index));
          }}
          onHover={setHover}
          onMove={move}
          onDragStart={() => {
            setOpen(null);
          }}
          renderEditor={(index) => {
            const slot = slots[index];
            if (slot === undefined) return null;
            return (
              <SlotEditor
                key={index}
                slot={slot}
                canRemove={slots.length > MIN_SLOTS}
                onCommit={(edited) => {
                  setSlots(slots.map((existing, at) => (at === index ? edited : existing)));
                }}
                onRemove={() => {
                  setOpen(null);
                  setSlots(slots.filter((_, at) => at !== index));
                }}
              />
            );
          }}
        />
        <div className="list-footer">
          <PresetMenu
            up
            note="Pick a starting point. You can change everything afterwards."
            onPick={(preset) => {
              if (full) return;
              setOpen(slots.length);
              setSlots([...slots, preset]);
            }}
            trigger={({ open: menuOpen, toggle }) => (
              <button
                type="button"
                className="btn btn-block"
                aria-expanded={menuOpen}
                disabled={full}
                onClick={toggle}
              >
                <Icon name="plus" />
                Add slot
              </button>
            )}
          />
          {full && <span className="field-note">A ring holds up to {MAX_SLOTS} slots.</span>}
        </div>
      </div>
      <aside className="preview-card">
        <p className="preview-title">Preview</p>
        <RingPreview slots={slots} focus={hover ?? open} />
        <p className="preview-note">Changes reach your ring as you make them.</p>
      </aside>
    </div>
  );
}
