import type { Updates } from "./use-updates";

/** Shown above every page while an update is available or being installed. */
export function UpdateBanner({ updates }: { updates: Updates }) {
  const { state } = updates;
  if (state.kind === "available") {
    return (
      <div className="banner banner-info" role="status">
        <div className="banner-text">
          <p className="banner-title">ClickHub {state.version} is available</p>
          <p className="banner-body">
            Installing takes a few seconds. ClickHub closes and starts again on its own.
          </p>
          <div className="banner-actions">
            <button type="button" className="btn btn-primary" onClick={updates.install}>
              Install and restart
            </button>
          </div>
        </div>
      </div>
    );
  }
  if (state.kind === "downloading" || state.kind === "installing") {
    const percent =
      state.kind === "downloading" && state.total !== null && state.total > 0
        ? Math.min(100, Math.round((state.downloaded / state.total) * 100))
        : null;
    return (
      <div className="banner banner-info" role="status" aria-live="polite">
        <div className="banner-text">
          <p className="banner-title">
            {state.kind === "installing"
              ? "Starting the installer…"
              : `Downloading the update${percent !== null ? ` (${String(percent)}%)` : "…"}`}
          </p>
          <div className="progress" aria-hidden="true">
            <div
              className="progress-bar"
              style={{ transform: `scaleX(${String((percent ?? 100) / 100)})` }}
            />
          </div>
        </div>
      </div>
    );
  }
  return null;
}
