# Data model: Wayland data-control capture

Reuses `Backend`, `BackendEvent`, `BackendSelection` from spec 002 (`contracts/backend.md`). Adds an internal `Offer { id, mime_types }` lifetime-bound to the current selection.
