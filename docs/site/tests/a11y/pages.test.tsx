import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { configureAxe } from "vitest-axe";
import { App } from "../../src/App";
import { PAGES } from "../../src/content/pages";
import { I18nProvider } from "../../src/i18n/context";

// Color contrast needs real layout; tests/unit/tokens.test.ts checks the token pairs instead.
const axe = configureAxe({ rules: { "color-contrast": { enabled: false } } });

function mount(path: string) {
  window.location.hash = `#${path}`;
  return render(
    <I18nProvider initial="en-US">
      <App />
    </I18nProvider>,
  );
}

describe("accessibility", () => {
  for (const p of PAGES) {
    it(`${p.path} has no axe violations`, async () => {
      const { container } = mount(p.path);
      expect(await axe(container)).toHaveNoViolations();
    }, 20000);
  }

  it("has a skip link, one main landmark and one h1 per page", () => {
    mount("/install");
    expect(screen.getByRole("button", { name: "Skip to content" })).toBeInTheDocument();
    expect(screen.getAllByRole("main")).toHaveLength(1);
    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
  });

  it("puts a maturity badge with a word on every page", () => {
    mount("/project/roadmap");
    expect(screen.getAllByText("Planned").length).toBeGreaterThan(0);
  });

  it("moves focus to the heading on navigation", async () => {
    mount("/reference/config");
    await waitFor(() => expect(document.activeElement?.tagName).toBe("H1"));
  });

  it("keeps the theme switch pressed state in sync", async () => {
    mount("/");
    const light = screen.getByRole("button", { name: "Light theme" });
    await act(async () => fireEvent.click(light));
    expect(light).toHaveAttribute("aria-pressed", "true");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(screen.getByRole("button", { name: "Dark theme" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
  });

  it("search opens, finds a page and closes with Escape, returning focus", async () => {
    mount("/");
    const open = screen.getByRole("button", { name: "Search" });
    await act(async () => fireEvent.click(open));
    const box = screen.getByRole("searchbox");
    await act(async () => fireEvent.change(box, { target: { value: "passc" } }));
    expect(screen.getAllByRole("link", { name: /Pause and passc/ }).length).toBeGreaterThan(0);
    await act(async () => fireEvent.keyDown(box, { key: "Escape" }));
    expect(screen.queryByRole("searchbox")).toBeNull();
    expect(document.activeElement).toBe(open);
  });

  it("renders a not-found page for an unknown route", () => {
    mount("/nope");
    expect(screen.getByRole("heading", { level: 1, name: "Page not found" })).toBeInTheDocument();
  });
});
