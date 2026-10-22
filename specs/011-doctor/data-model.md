# Data model: Doctor

`Check { id: &'static str, status: Ok|Info|Warn|Fail, message: String, hint: Option<String> }`; check ids: session, backend, store, daemon, pause, tray, gnome_extension, gnome_shell, picker, autostart.
