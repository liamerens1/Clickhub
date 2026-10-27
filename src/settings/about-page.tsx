import { useState } from "react";

import { type AppInfo, uninstall, type UpdateState } from "../shared/ipc";

import { ConfirmDialog } from "./confirm-dialog";
import type { Updates } from "./use-updates";

function updateHint(state: UpdateState): string {
  switch (state.kind) {
    case "idle":
      return "ClickHub checks once a day and asks before installing anything.";
    case "checking":
      return "Checking…";
    case "upToDate":
      return `You have the latest version (checked ${new Date(state.checkedAtMs).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}).`;
    case "available":
      return `Version ${state.version} is available.`;
    case "downloading":
      return "Downloading the update…";
    case "installing":
      return "Starting the installer…";
    case "failed":
      return `Couldn't check: ${state.error}`;
  }
}

interface AboutPageProps {
  info: AppInfo | null;
  updates: Updates;
}

/** Version, updates, uninstall, and limits worth knowing. */
export function AboutPage({ info, updates }: AboutPageProps) {
  const [confirming, setConfirming] = useState(false);
  const [uninstallError, setUninstallError] = useState<string | null>(null);
  const installed = info?.installed === true;
  const busy = ["checking", "downloading", "installing"].includes(updates.state.kind);

  return (
    <div className="tiles">
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">ClickHub</span>
          <span className="tile-hint">A radial control centre at your cursor.</span>
        </div>
        <span className="kbd">{info !== null ? `v${info.version}` : "…"}</span>
      </div>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Updates</span>
          <span
            className={updates.state.kind === "failed" ? "tile-hint field-error" : "tile-hint"}
            aria-live="polite"
          >
            {updateHint(updates.state)}
          </span>
        </div>
        {updates.state.kind === "available" ? (
          <button type="button" className="btn btn-primary" onClick={updates.install}>
            Install and restart
          </button>
        ) : (
          <button type="button" className="btn" disabled={busy} onClick={updates.check}>
            Check now
          </button>
        )}
      </div>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Uninstall</span>
          <span className="tile-hint">
            {installed
              ? "Removes ClickHub. The uninstaller asks whether to keep your settings."
              : "Only the installed copy of ClickHub can be uninstalled from here."}
          </span>
          {uninstallError !== null && <span className="field-error">{uninstallError}</span>}
        </div>
        <button
          type="button"
          className="btn btn-danger"
          disabled={!installed}
          onClick={() => {
            setConfirming(true);
          }}
        >
          Uninstall…
        </button>
      </div>
      {confirming && (
        <ConfirmDialog
          title="Uninstall ClickHub?"
          confirmLabel="Uninstall"
          danger
          onCancel={() => {
            setConfirming(false);
          }}
          onConfirm={() => {
            setConfirming(false);
            void uninstall().then((result) => {
              if (!result.ok) setUninstallError(result.error);
            });
          }}
        >
          <p>ClickHub will close and the Windows uninstaller will open.</p>
          <p>
            To keep your ring and trigger for a later reinstall, leave &quot;Delete the application
            data&quot; unticked.
          </p>
        </ConfirmDialog>
      )}

      <h2 className="section-label">Good to know</h2>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Apps running as administrator</span>
          <span className="tile-hint">
            Windows does not let ClickHub see clicks aimed at them, so the ring does not open over
            them. ClickHub never asks to run as administrator.
          </span>
        </div>
      </div>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Full-screen games and videos</span>
          <span className="tile-hint">
            The trigger switches itself off while a full-screen app is in front, so your button
            works there as usual.
          </span>
        </div>
      </div>
    </div>
  );
}
