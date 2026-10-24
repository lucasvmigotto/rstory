/// <reference types="vite/client" />
import type { Versions } from "../scripts/footer-versions";

declare global {
  const __SITE__: {
    repo: string;
    version: string;
    docsVersion: string;
    license: string;
    versions: Versions;
    siteUrl: string;
  };
}
