import { HashRouter, Link, Route, Routes } from "react-router-dom";
import { DocPage } from "./components/DocPage";
import { Footer } from "./components/Footer";
import { Header } from "./components/Header";
import { PAGES } from "./content/pages";
import { useI18n } from "./i18n/context";

function NotFound() {
  const { ui } = useI18n();
  return (
    <div className="prose mx-auto max-w-3xl px-4 pt-10">
      <h1>{ui.notFound.title}</h1>
      <p>{ui.notFound.body}</p>
      <p>
        <Link to="/">{ui.notFound.home}</Link>
      </p>
    </div>
  );
}

export function Shell() {
  const { ui } = useI18n();
  return (
    <>
      {/* A hash link would change the route, so the skip link moves focus instead. */}
      <button
        type="button"
        className="skip-link"
        onClick={() => document.getElementById("main")?.focus()}
      >
        {ui.skipToContent}
      </button>
      <Header />
      <main id="main" tabIndex={-1} className="outline-none">
        <Routes>
          {PAGES.map((p) => (
            <Route key={p.id} path={p.path} element={<DocPage page={p} />} />
          ))}
          <Route path="*" element={<NotFound />} />
        </Routes>
      </main>
      <Footer />
    </>
  );
}

// HashRouter: static hosts give no SPA fallback, so deep links stay in the hash.
export function App() {
  return (
    <HashRouter>
      <Shell />
    </HashRouter>
  );
}
