# Data model: Session detection & X11 capture (incl. XWayland bridge)

## Backend (trait)

```rust
trait Backend {
    fn events(&mut self) -> impl Stream<Item = BackendEvent>;
    async fn read_text(&mut self, limit: usize) -> Result<Vec<u8>, ReadError>; // TooLarge | ConversionRefused | Timeout | Disconnected
    async fn serve(&mut self, text: Arc<[u8]>) -> Result<(), ServeError>;      // resolves when ownership confirmed
    fn stop_serving(&mut self);
}
enum BackendEvent { SelectionChanged { types: Vec<String>, self_owned: bool }, SelectionCleared, Disconnected }
```

## BackendSelection

`{ backend: Wayland | X11 | None, mode: Native | XwaylandBridge | None, reason: Reason }`

| Input (session, data-control global, DISPLAY, override) | Result |
|---|---|
| override = x11/wayland | that backend, reason `override` |
| x11, –, set | X11/Native |
| wayland, ext or zwlr, – | Wayland/Native (spec 003) |
| wayland, none, set | X11/XwaylandBridge, reason `no_data_control` |
| wayland, none, unset | None, reason `no_data_control_no_xwayland` |
| none (no session) | None, reason `no_session` |
