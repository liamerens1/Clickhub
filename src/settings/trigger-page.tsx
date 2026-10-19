import { useCallback, useState } from "react";

import { setTriggerCapture } from "../shared/ipc";
import {
  type Config,
  DEFAULT_TRIGGER,
  type Modifier,
  type MouseButton,
  type MouseTrigger,
} from "../shared/schema";

import { BUTTON_NAMES, MODIFIER_NAMES, MODIFIER_ORDER } from "./mouse-buttons";
import { Segmented } from "./segmented";
import { ShortcutRecorder } from "./shortcut-recorder";
import { Switch } from "./switch";
import { TriggerCapture } from "./trigger-capture";
import { fromTriggerShortcut, toTriggerShortcut, triggerShortcutProblem } from "./trigger-shortcut";

interface TriggerPageProps {
  config: Config;
  commit: (config: Config) => void;
}

type Kind = "mouse" | "keyboard";

const KINDS = [
  { value: "mouse", label: "Mouse button" },
  { value: "keyboard", label: "Keyboard shortcut" },
] as const;

const DEFAULT_MOUSE: MouseTrigger =
  DEFAULT_TRIGGER.kind === "mouse"
    ? DEFAULT_TRIGGER
    : { kind: "mouse", button: "x2", modifiers: [], swallow: true };

/** What opens the ring: a mouse button (picked by pressing it) or a keyboard shortcut. */
export function TriggerPage({ config, commit }: TriggerPageProps) {
  const { trigger } = config;
  // The keyboard view can be shown before a shortcut exists (nothing to save until one is
  // recorded), so the visible kind is local and follows the saved trigger otherwise.
  const [view, setView] = useState<Kind>(trigger.kind);
  const [savedKind, setSavedKind] = useState<Kind>(trigger.kind);
  if (trigger.kind !== savedKind) {
    // The saved trigger changed kind (Undo, a reload): show it.
    setSavedKind(trigger.kind);
    setView(trigger.kind);
  }
  const [lastMouse, setLastMouse] = useState<MouseTrigger>(
    trigger.kind === "mouse" ? trigger : DEFAULT_MOUSE,
  );
  const [capturing, setCapturing] = useState(false);
  const [shortcutProblem, setShortcutProblem] = useState<string | null>(null);

  const setMouse = (patch: Partial<MouseTrigger>) => {
    const next: MouseTrigger = { ...(trigger.kind === "mouse" ? trigger : lastMouse), ...patch };
    setLastMouse(next);
    commit({ ...config, trigger: next });
  };

  const onCaptured = useCallback(
    (button: MouseButton, modifiers: Modifier[]) => {
      setCapturing(false);
      const base = trigger.kind === "mouse" ? trigger : lastMouse;
      const next: MouseTrigger = { ...base, button, modifiers };
      setLastMouse(next);
      commit({ ...config, trigger: next });
    },
    [commit, config, lastMouse, trigger],
  );
  const onCancel = useCallback(() => {
    setCapturing(false);
  }, []);
  const onListening = useCallback((listening: boolean) => {
    void setTriggerCapture(listening);
  }, []);
  const onShortcut = useCallback(
    (canonical: string) => {
      const problem = triggerShortcutProblem(canonical);
      setShortcutProblem(problem);
      if (problem === null) {
        commit({
          ...config,
          trigger: { kind: "keyboard", shortcut: toTriggerShortcut(canonical) },
        });
      }
    },
    [commit, config],
  );

  const mouse = trigger.kind === "mouse" ? trigger : lastMouse;

  return (
    <div className="tiles">
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Open the ring with</span>
          <span className="tile-hint">
            Tap it to keep the ring open and click a slot, or hold it, aim and let go.
          </span>
        </div>
        <Segmented
          label="Trigger type"
          value={view}
          options={KINDS}
          onChange={(kind) => {
            setView(kind);
            setCapturing(false);
            if (kind === "mouse" && trigger.kind !== "mouse") {
              commit({ ...config, trigger: lastMouse });
            }
          }}
        />
      </div>

      {view === "mouse" && (
        <>
          {capturing ? (
            <TriggerCapture onCaptured={onCaptured} onCancel={onCancel} />
          ) : (
            <div className="tile">
              <div className="tile-text">
                <span className="tile-hint">Button</span>
                <span className="trigger-current">
                  {[
                    ...MODIFIER_ORDER.filter((m) => mouse.modifiers.includes(m)).map(
                      (m) => MODIFIER_NAMES[m],
                    ),
                    BUTTON_NAMES[mouse.button],
                  ].join(" + ")}
                </span>
                {mouse.button === "right" && mouse.swallow && mouse.modifiers.length === 0 && (
                  <span className="tile-hint">
                    Right-click menus will not open while this is the trigger. Add a key below, or
                    switch off hiding the click.
                  </span>
                )}
              </div>
              <button
                type="button"
                className="btn btn-primary"
                onClick={() => {
                  setCapturing(true);
                }}
              >
                Press a new one
              </button>
            </div>
          )}
          <div className="tile">
            <div className="tile-text">
              <span className="tile-title">Hold these keys too</span>
              <span className="tile-hint">
                The ring opens only while exactly these are held. None means none held.
              </span>
            </div>
            <div className="chips" role="group" aria-label="Modifier keys">
              {MODIFIER_ORDER.map((modifier) => {
                const on = mouse.modifiers.includes(modifier);
                return (
                  <button
                    key={modifier}
                    type="button"
                    className="chip"
                    aria-pressed={on}
                    onClick={() => {
                      const modifiers = on
                        ? mouse.modifiers.filter((held) => held !== modifier)
                        : MODIFIER_ORDER.filter(
                            (held) => held === modifier || mouse.modifiers.includes(held),
                          );
                      setMouse({ modifiers });
                    }}
                  >
                    {MODIFIER_NAMES[modifier]}
                  </button>
                );
              })}
            </div>
          </div>
          <div className="tile">
            <div className="tile-text">
              <span className="tile-title">Hide the click from the app underneath</span>
              <span className="tile-hint">
                Off: the button also does its usual job, like Back in a browser.
              </span>
            </div>
            <Switch
              label="Hide the click from the app underneath"
              checked={mouse.swallow}
              onChange={(swallow) => {
                setMouse({ swallow });
              }}
            />
          </div>
        </>
      )}

      {view === "keyboard" && (
        <div className="tile">
          <div className="tile-text">
            <span className="tile-title">Shortcut</span>
            <span className="tile-hint">The ring opens at the mouse pointer.</span>
            <div className="tile-body">
              <ShortcutRecorder
                label="Keyboard trigger"
                value={trigger.kind === "keyboard" ? fromTriggerShortcut(trigger.shortcut) : ""}
                onChange={onShortcut}
                onListeningChange={onListening}
              />
              {shortcutProblem !== null && <span className="field-error">{shortcutProblem}</span>}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
