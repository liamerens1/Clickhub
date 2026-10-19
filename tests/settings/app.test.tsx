// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";

import { App } from "../../src/settings/app";

afterEach(cleanup);

describe("settings App", () => {
  it("renders the ClickHub heading", () => {
    render(<App />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("ClickHub");
  });
});
