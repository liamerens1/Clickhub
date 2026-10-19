import { type AppInfo, type AppStatus, openFolder } from "../shared/ipc";

import { Icon } from "./icon";
import { Switch } from "./switch";

interface GeneralPageProps {
  status: AppStatus | null;
  info: AppInfo | null;
  onPause: (paused: boolean) => void;
}

/** How ClickHub runs: pause, where its files live. */
export function GeneralPage({ status, info, onPause }: GeneralPageProps) {
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
          <span className="tile-hint">Arrives with the installer.</span>
        </div>
        <span className="chip">Phase 4</span>
        <Switch label="Start with Windows" checked={false} disabled onChange={() => undefined} />
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
