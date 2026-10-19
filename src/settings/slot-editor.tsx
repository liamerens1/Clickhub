import { useCallback, useEffect, useRef, useState } from "react";

import { pickFile } from "../shared/ipc";
import { type Action, type Slot, slotSchema } from "../shared/schema";

import { Icon } from "./icon";
import { formatArgs, parseArgs } from "./launch-args";
import { PresetMenu } from "./preset-menu";
import { Segmented } from "./segmented";
import { ShortcutRecorder } from "./shortcut-recorder";

/** Typing is saved once the user pauses this long (and the slot is valid). */
const TYPING_SAVE_MS = 450;

interface SlotEditorProps {
  slot: Slot;
  onCommit: (slot: Slot) => void;
  onRemove: () => void;
  canRemove: boolean;
}

interface Draft {
  label: string;
  kind: Action["kind"];
  keys: string;
  path: string;
  args: string;
  url: string;
}

type Parsed = { ok: true; slot: Slot } | { ok: false; label?: string; action?: string };

const KINDS = [
  { value: "shortcut", label: "Shortcut" },
  { value: "launch", label: "App or file" },
  { value: "url", label: "Link" },
] as const;

function fromSlot(slot: Slot): Draft {
  const { action } = slot;
  return {
    label: slot.label,
    kind: action.kind,
    keys: action.kind === "shortcut" ? action.keys : "",
    path: action.kind === "launch" ? action.path : "",
    args: action.kind === "launch" ? formatArgs(action.args) : "",
    url: action.kind === "url" ? action.url : "https://",
  };
}

function parseDraft(draft: Draft): Parsed {
  const action =
    draft.kind === "shortcut"
      ? { kind: draft.kind, keys: draft.keys }
      : draft.kind === "launch"
        ? { kind: draft.kind, path: draft.path, args: parseArgs(draft.args) }
        : { kind: draft.kind, url: draft.url.trim() };
  const result = slotSchema.safeParse({ label: draft.label, action });
  if (result.success) return { ok: true, slot: result.data };
  const parsed: Parsed = { ok: false };
  for (const issue of result.error.issues) {
    const field = issue.path[0] === "label" ? "label" : "action";
    parsed[field] ??= issue.message;
  }
  return parsed;
}

const sameSlot = (a: Slot, b: Slot) => JSON.stringify(a) === JSON.stringify(b);

/** Edits one slot in place. Valid changes are saved as they happen; invalid ones wait. */
export function SlotEditor({ slot, onCommit, onRemove, canRemove }: SlotEditorProps) {
  const [draft, setDraft] = useState(() => fromSlot(slot));
  const [shown, setShown] = useState(slot);
  const timer = useRef<number | undefined>(undefined);

  // The slot changed from outside (Undo, a preset, a reload): show it, unless it is simply the
  // value this editor just saved.
  if (slot !== shown) {
    setShown(slot);
    const parsed = parseDraft(draft);
    if (!(parsed.ok && sameSlot(parsed.slot, slot))) setDraft(fromSlot(slot));
  }

  useEffect(
    () => () => {
      window.clearTimeout(timer.current);
    },
    [],
  );

  const update = useCallback(
    (patch: Partial<Draft>, when: "now" | "typing") => {
      const next = { ...draft, ...patch };
      setDraft(next);
      window.clearTimeout(timer.current);
      const parsed = parseDraft(next);
      if (!parsed.ok || sameSlot(parsed.slot, slot)) return;
      if (when === "now") {
        onCommit(parsed.slot);
      } else {
        timer.current = window.setTimeout(() => {
          onCommit(parsed.slot);
        }, TYPING_SAVE_MS);
      }
    },
    [draft, slot, onCommit],
  );

  const flush = () => {
    window.clearTimeout(timer.current);
    const parsed = parseDraft(draft);
    if (parsed.ok && !sameSlot(parsed.slot, slot)) onCommit(parsed.slot);
  };

  const parsed = parseDraft(draft);
  const labelError = parsed.ok ? undefined : parsed.label;
  const actionError = parsed.ok ? undefined : parsed.action;
  const showActionError =
    actionError !== undefined &&
    (draft.kind === "shortcut"
      ? draft.keys !== ""
      : draft.kind === "launch"
        ? draft.path !== ""
        : draft.url !== "https://");

  const onKeys = useCallback(
    (keys: string) => {
      update({ keys }, "now");
    },
    [update],
  );

  return (
    <div className="slot-editor">
      <label className="field">
        <span className="field-label">Label</span>
        <input
          className="input"
          value={draft.label}
          aria-invalid={labelError !== undefined}
          onChange={(event) => {
            update({ label: event.target.value }, "typing");
          }}
          onBlur={flush}
        />
        {labelError !== undefined && <span className="field-error">{labelError}</span>}
      </label>

      <div className="field">
        <span className="field-label">What it does</span>
        <div>
          <Segmented
            label="Action type"
            value={draft.kind}
            options={KINDS}
            onChange={(kind) => {
              update({ kind }, "now");
            }}
          />
        </div>
      </div>

      {draft.kind === "shortcut" && (
        <ShortcutRecorder label="Shortcut to send" value={draft.keys} onChange={onKeys} />
      )}

      {draft.kind === "launch" && (
        <>
          <div className="field">
            <span className="field-label">App, file or folder</span>
            <div className="field-row">
              <input
                className="input"
                value={draft.path}
                placeholder="C:\Program Files\App\app.exe"
                aria-label="App, file or folder"
                aria-invalid={showActionError}
                onChange={(event) => {
                  update({ path: event.target.value }, "typing");
                }}
                onBlur={flush}
              />
              <button
                type="button"
                className="btn"
                onClick={() => {
                  void pickFile().then((result) => {
                    if (result.ok && result.value !== null) update({ path: result.value }, "now");
                  });
                }}
              >
                <Icon name="folder" />
                Browse…
              </button>
            </div>
          </div>
          <label className="field">
            <span className="field-label">Arguments (optional)</span>
            <input
              className="input"
              value={draft.args}
              placeholder='--new-window "C:\My folder"'
              onChange={(event) => {
                update({ args: event.target.value }, "typing");
              }}
              onBlur={flush}
            />
          </label>
        </>
      )}

      {draft.kind === "url" && (
        <label className="field">
          <span className="field-label">Link</span>
          <input
            className="input"
            value={draft.url}
            inputMode="url"
            aria-invalid={showActionError}
            onChange={(event) => {
              update({ url: event.target.value }, "typing");
            }}
            onBlur={flush}
          />
        </label>
      )}

      {showActionError && <span className="field-error">{actionError}</span>}

      <div className="slot-editor-footer">
        <PresetMenu
          onPick={(preset) => {
            setDraft(fromSlot(preset));
            if (!sameSlot(preset, slot)) onCommit(preset);
          }}
          up
          trigger={({ open, toggle }) => (
            <button type="button" className="btn btn-ghost" aria-expanded={open} onClick={toggle}>
              <Icon name="sparkle" />
              Use a preset
            </button>
          )}
        />
        <button
          type="button"
          className="btn btn-ghost btn-danger"
          disabled={!canRemove}
          title={canRemove ? undefined : "A ring needs at least 2 slots"}
          onClick={onRemove}
        >
          <Icon name="trash" />
          Remove
        </button>
      </div>
    </div>
  );
}
