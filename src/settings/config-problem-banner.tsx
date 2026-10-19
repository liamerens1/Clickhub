import { type ConfigSnapshot, openFolder } from "../shared/ipc";

import { Icon } from "./icon";

interface ConfigProblemBannerProps {
  problem: NonNullable<ConfigSnapshot["problem"]>;
  onReload: () => void;
}

/** Explains that `config.json` could not be used, what is running instead, and how to fix it. */
export function ConfigProblemBanner({ problem, onReload }: ConfigProblemBannerProps) {
  const using = problem.fallback === "lastGood" ? "your last working settings" : "the defaults";
  return (
    <div className="banner" role="alert">
      <div className="banner-text">
        <p className="banner-title">Your settings file has a problem</p>
        <p className="banner-body">
          {problem.error}. ClickHub is running on {using} until it is fixed.
          {problem.backup !== null && ` A copy of the file was saved as ${problem.backup}.`} Fix the
          file and reload it, or change anything here to replace it with what you see.
        </p>
        <div className="banner-actions">
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
          <button type="button" className="btn" onClick={onReload}>
            <Icon name="refresh" />
            Reload the file
          </button>
        </div>
      </div>
    </div>
  );
}
