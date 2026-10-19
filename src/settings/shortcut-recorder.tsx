import { useEffect, useState } from "react";

import { chordText, isModifierCode, keyFromCode } from "./key-names";
import { Keys } from "./keys";

interface ShortcutRecorderProps {
  /** Canonical shortcut ("Ctrl+Shift+S"), or "" when none is set yet. */
  value: string;
  onChange: (shortcut: string) => void;
  /** Told when listening starts and stops (the trigger page switches the trigger off). */
  onListeningChange?: (listening: boolean) => void;
  label: string;
}

/**
 * "Press the keys you want." Listens to the next key press in this window. Windows keeps most
 * Win shortcuts (Win+D, Win+L, ...) for itself, so they never reach the window: the Win toggle
 * adds Win to whatever is pressed next.
 */
export function ShortcutRecorder({
  value,
  onChange,
  onListeningChange,
  label,
}: ShortcutRecorderProps) {
  const [listening, setListening] = useState(false);
  const [addWin, setAddWin] = useState(false);
  const [held, setHeld] = useState("");
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    onListeningChange?.(listening);
  }, [listening, onListeningChange]);

  useEffect(() => {
    if (!listening) return;
    const finish = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      const modifiers = {
        ctrl: event.ctrlKey,
        shift: event.shiftKey,
        alt: event.altKey,
        win: event.metaKey || addWin,
      };
      if (isModifierCode(event.code)) {
        setHeld(event.type === "keydown" ? chordText(modifiers, "").slice(0, -1) : "");
        return;
      }
      // Windows only reports Print Screen when it is released.
      if (event.type === "keyup" && event.code !== "PrintScreen") return;
      const key = keyFromCode(event.code);
      if (key === null) {
        setProblem(`ClickHub can't send "${event.key}". Try another key.`);
        return;
      }
      setProblem(null);
      setListening(false);
      setHeld("");
      onChange(chordText(modifiers, key));
    };
    const stop = () => {
      setListening(false);
      setHeld("");
    };
    window.addEventListener("keydown", finish, true);
    window.addEventListener("keyup", finish, true);
    window.addEventListener("blur", stop);
    return () => {
      window.removeEventListener("keydown", finish, true);
      window.removeEventListener("keyup", finish, true);
      window.removeEventListener("blur", stop);
    };
  }, [listening, addWin, onChange]);

  useEffect(
    () => () => {
      onListeningChange?.(false);
    },
    [onListeningChange],
  );

  return (
    <div className="field">
      <div className="recorder">
        <div
          className={listening ? "recorder-display is-listening" : "recorder-display"}
          aria-live="polite"
          aria-label={label}
        >
          {listening ? (
            <>
              <span className="listening-dot" />
              {held !== "" ? <Keys shortcut={held} /> : <span>Press the keys…</span>}
            </>
          ) : value !== "" ? (
            <Keys shortcut={value} />
          ) : (
            <span className="field-note">Not set</span>
          )}
        </div>
        {listening ? (
          <>
            <button
              type="button"
              className="chip"
              aria-pressed={addWin}
              onClick={() => {
                setAddWin((on) => !on);
              }}
            >
              + Win
            </button>
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => {
                setListening(false);
                setHeld("");
              }}
            >
              Cancel
            </button>
          </>
        ) : (
          <button
            type="button"
            className="btn"
            onClick={() => {
              setProblem(null);
              setAddWin(false);
              setListening(true);
            }}
          >
            {value === "" ? "Record" : "Record again"}
          </button>
        )}
      </div>
      {problem !== null && <span className="field-error">{problem}</span>}
      {listening && problem === null && (
        <span className="field-note">
          Windows keeps most Win shortcuts for itself: switch on + Win, then press the rest.
        </span>
      )}
    </div>
  );
}
