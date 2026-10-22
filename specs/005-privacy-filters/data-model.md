# Data model: Privacy filters

## FilterDecision

`Record(Vec<u8>) | SkipPaused | SkipSensitive | SkipOversize | IgnoreNoText | IgnoreEmpty`

Order (first match wins): paused → sensitive (types contain a hint) → no text type → read(limit = max_entry_bytes + 1) exceeds → empty → Record.

## SENSITIVE_HINTS

`["x-kde-passwordManagerHint"]` — exact, case-sensitive.
