import { messages as enUS } from "./en-US";
import type { Locale } from "./locales";
import type { Messages } from "./types";

// Adding a locale: a folder like ./en-US, one line here, and its tag in LOCALES. No component changes.
export const MESSAGES: Record<Locale, Messages> = { "en-US": enUS };
