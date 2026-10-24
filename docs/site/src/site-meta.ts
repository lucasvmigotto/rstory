import { sourceUrl } from "../scripts/footer-versions";

export const SITE = __SITE__;
export const src = (path: string) => sourceUrl(SITE.repo, SITE.versions.ref, path);
