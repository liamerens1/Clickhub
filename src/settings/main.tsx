import "./settings.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./app";

const rootElement = document.getElementById("root");
if (rootElement === null) {
  throw new Error("Settings root element #root is missing from index.html");
}
const root = createRoot(rootElement);

/**
 * In a plain browser during development there is no Rust side, so a mock stands in for it (and
 * `?perf=<rows>` opens the list performance harness). Both are dynamic imports behind
 * `import.meta.env.DEV`, so neither reaches a build.
 */
async function start() {
  let page = <App />;
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    const { installDevMock } = await import("../shared/ipc/dev-mock");
    installDevMock();
    const rows = Number(new URLSearchParams(window.location.search).get("perf"));
    if (rows > 0) {
      const { PerfHarness } = await import("./perf-harness");
      page = <PerfHarness rows={rows} />;
    }
  }
  root.render(<StrictMode>{page}</StrictMode>);
}

void start();
