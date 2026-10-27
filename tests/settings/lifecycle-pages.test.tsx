// @vitest-environment jsdom
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppInfo } from "../../src/shared/ipc";

import { AboutPage } from "../../src/settings/about-page";
import { GeneralPage } from "../../src/settings/general-page";
import { UpdateBanner } from "../../src/settings/update-banner";
import type { Updates } from "../../src/settings/use-updates";

afterEach(async () => {
  cleanup();
  await new Promise((resolve) => setTimeout(resolve, 0));
  clearMocks();
});

const info = (installed: boolean): AppInfo => ({
  version: "0.4.0",
  installed,
  configPath: "config.json",
  logDir: null,
});

const updates = (state: Updates["state"]): Updates => ({
  state,
  check: vi.fn(),
  install: vi.fn(),
});

describe("About page", () => {
  it("offers Check now, and Install once an update is found", () => {
    const idle = updates({ kind: "idle" });
    const { rerender } = render(<AboutPage info={info(true)} updates={idle} />);
    fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(idle.check).toHaveBeenCalledOnce();

    const found = updates({ kind: "available", version: "0.4.1", notes: null });
    rerender(<AboutPage info={info(true)} updates={found} />);
    expect(screen.getByText("Version 0.4.1 is available.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Install and restart" }));
    expect(found.install).toHaveBeenCalledOnce();
  });

  it("asks before uninstalling, and Cancel does nothing", async () => {
    const calls: string[] = [];
    mockIPC((command) => {
      calls.push(command);
      return null;
    });
    render(<AboutPage info={info(true)} updates={updates({ kind: "idle" })} />);

    fireEvent.click(screen.getByRole("button", { name: "Uninstall…" }));
    expect(screen.getByRole("alertdialog")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(calls).toEqual([]);

    fireEvent.click(screen.getByRole("button", { name: "Uninstall…" }));
    fireEvent.click(screen.getByRole("button", { name: "Uninstall" }));
    await waitFor(() => {
      expect(calls).toEqual(["uninstall"]);
    });
  });

  it("does not offer Uninstall for a copy that was not installed", () => {
    render(<AboutPage info={info(false)} updates={updates({ kind: "idle" })} />);
    const button = screen.getByRole("button", { name: "Uninstall…" });
    expect((button as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("Update banner", () => {
  it("shows download progress", () => {
    render(<UpdateBanner updates={updates({ kind: "downloading", downloaded: 50, total: 200 })} />);
    expect(screen.getByText("Downloading the update (25%)")).toBeTruthy();
  });

  it("is hidden when there is nothing to install", () => {
    const { container } = render(
      <UpdateBanner updates={updates({ kind: "upToDate", checkedAtMs: 0 })} />,
    );
    expect(container.innerHTML).toBe("");
  });
});

describe("General page", () => {
  it("switches starting with Windows off", async () => {
    let enabled = true;
    mockWindows("settings");
    mockIPC((command, payload) => {
      if (command === "set_autostart") {
        enabled = (payload as { enabled: boolean }).enabled;
      }
      return { available: true, enabled };
    });
    render(
      <GeneralPage status={{ paused: false, problem: null }} info={info(true)} onPause={vi.fn()} />,
    );
    const toggle = screen.getByRole("switch", { name: "Start with Windows" });
    await waitFor(() => {
      expect(toggle.getAttribute("aria-checked")).toBe("true");
    });
    fireEvent.click(toggle);
    await waitFor(() => {
      expect(toggle.getAttribute("aria-checked")).toBe("false");
    });
    expect(enabled).toBe(false);
  });

  it("cannot switch it on for a copy that was not installed", async () => {
    mockIPC(() => ({ available: false, enabled: false }));
    render(
      <GeneralPage
        status={{ paused: false, problem: null }}
        info={info(false)}
        onPause={vi.fn()}
      />,
    );
    await waitFor(() => {
      expect(screen.getByText(/Only the installed copy/)).toBeTruthy();
    });
    const toggle = screen.getByRole("switch", { name: "Start with Windows" });
    expect((toggle as HTMLButtonElement).disabled).toBe(true);
  });
});
