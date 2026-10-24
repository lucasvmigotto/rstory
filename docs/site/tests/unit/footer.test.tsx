import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Footer } from "../../src/components/Footer";
import { I18nProvider } from "../../src/i18n/context";
import { SITE } from "../../src/site-meta";

describe("Footer", () => {
  it("shows every version entry, linked to its tree, then the license and the theme switch", () => {
    render(
      <I18nProvider initial="en-US">
        <Footer />
      </I18nProvider>,
    );
    const group = screen.getByRole("list", { name: "Versions" });
    for (const e of SITE.versions.entries) {
      const el = within(group).getByText(e.text);
      if (e.href) expect(el.closest("a")).toHaveAttribute("href", e.href);
    }
    expect(group.textContent).toMatch(/^rstory@v\d+\.\d+\.\d+\S*\s*·\s*docs@v\d+\.\d+\.\d+/);
    const license = screen.getByRole("link", { name: /GPL v3\+/ });
    expect(license.getAttribute("href")).toMatch(/\/blob\/[^/]+\/LICENSE$/);
    expect(license.getAttribute("href")).not.toContain("blob/main");
    expect(screen.getByRole("button", { name: "Dark theme" })).toBeInTheDocument();
  });

  it("takes the versions from Cargo.toml and package.json", () => {
    expect(SITE.versions.entries[0]?.text).toContain(`rstory@v`);
    expect(SITE.version).toMatch(/^\d+\.\d+\.\d+/);
    expect(SITE.versions.entries[1]?.text).toContain(`docs@v${SITE.docsVersion}`);
  });
});
