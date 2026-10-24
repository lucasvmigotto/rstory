// Applies the saved theme before first paint. Storage may be unavailable.
let saved = null;
try {
  saved = localStorage.getItem("rstory-docs-theme");
} catch (_) {}
if (saved === "light" || saved === "dark") {
  document.documentElement.dataset.theme = saved;
} else if (matchMedia("(prefers-color-scheme: light)").matches) {
  document.documentElement.dataset.theme = "light";
}
