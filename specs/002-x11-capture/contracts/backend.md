# Backend contract (internal)

Owner: spec 002. Consumers: 003 (implements), 004 (uses), 011 (reads selection).

- `events()` yields `SelectionChanged{types, self_owned}` once per ownership change, `SelectionCleared` when no owner, `Disconnected` when the display connection is lost (terminal for that instance).
- `read_text(limit)` reads the current selection's best text type; aborts with `TooLarge` once `limit` bytes are exceeded; never logs content.
- `serve(text)` resolves `Ok` only after ownership is confirmed; serving continues until another owner appears (`SelectionChanged{self_owned:false}` follows).
- Types are the owner's offered names verbatim (X11 target atom names or Wayland MIME types).
