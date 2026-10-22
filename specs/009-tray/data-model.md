# Data model: SNI tray

`TrayMenu { entries: Vec<(EntryId, label, pinned)>, status_label, pause_item: Pause|Resume, show_pause_minute: bool, show_open_picker: bool, clear: Idle|Confirming(n) }` — pure function of daemon snapshot.
