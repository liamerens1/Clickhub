import { useEffect, useState } from "react";

import type { SaveState } from "./use-config-store";

/** How long "Saved" stays before fading out. */
const SAVED_VISIBLE_MS = 1800;

/** A quiet note beside Undo: "Saving…", then "Saved" (fading), or why a save failed. */
export function SaveStatus({ save }: { save: SaveState }) {
  const [fadedFor, setFadedFor] = useState<SaveState | null>(null);

  useEffect(() => {
    if (save.kind !== "saved") return;
    const timer = window.setTimeout(() => {
      setFadedFor(save);
    }, SAVED_VISIBLE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [save]);

  const text =
    save.kind === "saving"
      ? "Saving…"
      : save.kind === "saved"
        ? "Saved"
        : save.kind === "error"
          ? `Couldn't save: ${save.message}`
          : "";
  const hidden = save.kind === "idle" || fadedFor === save;
  return (
    <span
      className={`save-status${save.kind === "error" ? " is-error" : ""}${hidden ? " is-hidden" : ""}`}
      role="status"
      aria-live="polite"
    >
      {text}
    </span>
  );
}
