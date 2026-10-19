import { type ReactNode, useEffect, useState } from "react";

import {
  type AppInfo,
  type AppStatus,
  getAppInfo,
  getStatus,
  onStatusChanged,
  setPaused,
} from "../shared/ipc";

import { AboutPage } from "./about-page";
import { ConfigProblemBanner } from "./config-problem-banner";
import { GeneralPage } from "./general-page";
import { Icon } from "./icon";
import { RingPage } from "./ring-page";
import { SaveStatus } from "./save-status";
import { type Section, Sidebar } from "./sidebar";
import { TriggerPage } from "./trigger-page";
import { useConfigStore } from "./use-config-store";

const HEADINGS: Record<Section, { title: string; description: string }> = {
  ring: {
    title: "Ring",
    description: "The slots in your ring, clockwise from the top. Drag to reorder.",
  },
  trigger: { title: "Trigger", description: "What opens the ring." },
  general: { title: "General", description: "How ClickHub runs." },
  about: { title: "About", description: "Version and things worth knowing." },
};

/** True when Ctrl+Z should undo a settings change rather than undo typing in a text box. */
function isUndoShortcut(event: KeyboardEvent): boolean {
  const typing =
    event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement;
  return event.ctrlKey && !event.shiftKey && !event.altKey && event.code === "KeyZ" && !typing;
}

/** Root of the settings window: the section sidebar and the selected page. */
export function App() {
  const store = useConfigStore();
  const [section, setSection] = useState<Section>("ring");
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    void getStatus().then((result) => {
      if (result.ok) setStatus(result.value);
    });
    void getAppInfo().then((result) => {
      if (result.ok) setInfo(result.value);
    });
    const unlisten = onStatusChanged(setStatus);
    return () => {
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, []);

  const { undo } = store;
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (isUndoShortcut(event)) {
        event.preventDefault();
        undo();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [undo]);

  const heading = HEADINGS[section];
  const { config } = store;

  let page: ReactNode;
  if (store.loadError !== null && config === null) {
    page = <p className="empty-state">Could not read the settings: {store.loadError}</p>;
  } else if (config === null) {
    page = <p className="empty-state">Loading…</p>;
  } else if (section === "ring") {
    page = <RingPage config={config} commit={store.commit} />;
  } else if (section === "trigger") {
    page = <TriggerPage config={config} commit={store.commit} />;
  } else if (section === "general") {
    page = (
      <GeneralPage
        status={status}
        info={info}
        onPause={(paused) => {
          void setPaused(paused).then((result) => {
            if (result.ok) setStatus(result.value);
          });
        }}
      />
    );
  } else {
    page = <AboutPage info={info} />;
  }

  return (
    <div className="app">
      <Sidebar active={section} onSelect={setSection} />
      <main className="content">
        <div className="content-inner">
          <header className="page-header">
            <div>
              <h1 className="page-title">{heading.title}</h1>
              <p className="page-description">{heading.description}</p>
            </div>
            <div className="header-actions">
              <SaveStatus save={store.save} />
              <button
                type="button"
                className="btn"
                disabled={!store.canUndo}
                title="Undo (Ctrl+Z)"
                onClick={store.undo}
              >
                <Icon name="undo" />
                Undo
              </button>
            </div>
          </header>
          {store.snapshot?.problem != null && (
            <ConfigProblemBanner problem={store.snapshot.problem} onReload={store.reload} />
          )}
          {page}
        </div>
      </main>
    </div>
  );
}
