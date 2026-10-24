import type { DocsContent } from "../../types";
import { concepts } from "./concepts";
import { guides } from "./guides";
import { project } from "./project";
import { reference } from "./reference";
import { start } from "./start";

// DocsContent has one entry per page id: a missing page or locale entry is a type error.
export const docs: DocsContent = { ...start, ...guides, ...concepts, ...reference, ...project };
