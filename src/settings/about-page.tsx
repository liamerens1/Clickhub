import type { AppInfo } from "../shared/ipc";

/** Version, limits worth knowing, and what is coming. */
export function AboutPage({ info }: { info: AppInfo | null }) {
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
          <span className="tile-title">Check for updates</span>
          <span className="tile-hint">Arrives with the installer.</span>
        </div>
        <span className="chip">Phase 4</span>
      </div>
      <div className="tile">
        <div className="tile-text">
          <span className="tile-title">Uninstall</span>
          <span className="tile-hint">Arrives with the installer.</span>
        </div>
        <span className="chip">Phase 4</span>
      </div>

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
