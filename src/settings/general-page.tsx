import { useEffect, useState } from "react";

import {
  type AppInfo,
  type AppStatus,
  type Autostart,
  getAutostart,
  openFolder,
  setAutostart,
} from "../shared/ipc";

import { Icon } from "./icon";
import { Switch } from "./switch";

interface GeneralPageProps {
  status: AppStatus | null;
  info: AppInfo | null;
  onPause: (paused: boolean) => void;
}

/** How ClickHub runs: pause, starting with Windows, where its files live. */
export function GeneralPage({ status, info, onPause }: GeneralPageProps) {
  const [autostart, setAutostartState] = useState<Autostart | null>(null);
  const [autostartError, setAutostartError] = useState<string | null>(null);

  useEffect(() => {
    void getAutostart().then((result) => {
      if (result.ok) setAutostartState(result.value);
      else setAutostartError(result.error);
    });
  }, []);

  return (
    <div className="tiles">
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Pause ClickHub</span>
          <span className="tile-hint">While paused, your trigger works as it normally would.</span>
        </div>
        <Switch
          label="Pause ClickHub"
          checked={status?.paused ?? false}
          disabled={status === null}
          onChange={onPause}
        />
      </div>
      {status?.problem != null && (
        <div className="tile">
          <div className="tile-text">
            <span className="tile-title">Needs attention</span>
            <span className="tile-hint">{status.problem}</span>
          </div>
          <button
            type="button"
            className="btn"
            onClick={() => {
              void openFolder("logs");
            }}
          >
            Open the log
          </button>
        </div>
      )}
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Start with Windows</span>
          <span className="tile-hint">
            {autostart?.available === false
              ? "Only the installed copy of ClickHub can start with Windows."
              : "ClickHub starts in the tray when you sign in."}
          </span>
          {autostartError !== null && <span className="field-error">{autostartError}</span>}
        </div>
        <Switch
          label="Start with Windows"
          checked={autostart?.enabled ?? false}
          disabled={autostart?.available !== true}
          onChange={(enabled) => {
            void setAutostart(enabled).then((result) => {
              if (result.ok) {
                setAutostartState(result.value);
                setAutostartError(null);
              } else {
                setAutostartError(result.error);
              }
            });
          }}
        />
      </div>

      <h2 className="section-label">Files</h2>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Settings file</span>
          <span className="tile-hint tile-mono">{info?.configPath ?? "…"}</span>
        </div>
        <button
          type="button"
          className="btn"
          onClick={() => {
            void openFolder("config");
          }}
        >
          <Icon name="folder" />
          Open folder
        </button>
      </div>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Logs</span>
          <span className="tile-hint tile-mono">{info?.logDir ?? "…"}</span>
        </div>
        <button
          type="button"
          className="btn"
          onClick={() => {
            void openFolder("logs");
          }}
        >
          <Icon name="folder" />
          Open folder
        </button>
      </div>
    </div>
  );
}
