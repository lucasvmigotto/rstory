# Data model: Terminal picker

## PickerState

`{ query: String, items: Vec<EntrySummary>, filtered: Vec<(idx, score, match_positions)>, selected: Option<EntryId>, preview: Visible|Hidden, mode: Normal | ConfirmDelete, read_only: bool, status_line: Option<String> }`

Update: `fn update(state, Msg) -> (state, Vec<Cmd>)` where `Cmd = Restore(id) | Pin(id) | Unpin(id) | Remove(id) | Quit(code)`.
