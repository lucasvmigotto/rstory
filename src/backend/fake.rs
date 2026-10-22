//! Scriptable test double for [`Backend`]: tests push events and read results, and inspect what
//! was served.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedSender;

use super::{Backend, BackendEvent, EventStream, ReadError, ServeError};

#[derive(Default)]
struct State {
    reads: VecDeque<Result<Vec<u8>, ReadError>>,
    read_limits: Vec<usize>,
    serve_result: Option<ServeError>,
    served: Vec<Arc<[u8]>>,
    stops: usize,
}

/// The backend half, handed to code under test.
pub struct FakeBackend {
    events: EventStream,
    state: Arc<Mutex<State>>,
}

/// The test half: script events and results, inspect calls.
#[derive(Clone)]
pub struct FakeHandle {
    tx: UnboundedSender<BackendEvent>,
    state: Arc<Mutex<State>>,
}

impl FakeBackend {
    /// A backend and its control handle.
    #[must_use]
    pub fn new() -> (Self, FakeHandle) {
        let (tx, events) = EventStream::channel();
        let state = Arc::new(Mutex::new(State::default()));
        (
            Self {
                events,
                state: Arc::clone(&state),
            },
            FakeHandle { tx, state },
        )
    }
}

impl FakeHandle {
    /// Queue an event for `events()`.
    pub fn emit(&self, event: BackendEvent) {
        let _ = self.tx.send(event);
    }

    /// Queue the result of the next `read_text` call. With nothing queued it fails `NoText`.
    pub fn push_read(&self, result: Result<Vec<u8>, ReadError>) {
        self.state.lock().unwrap().reads.push_back(result);
    }

    /// Make every following `serve` fail with `error`; `None` restores success.
    pub fn fail_serve(&self, error: Option<ServeError>) {
        self.state.lock().unwrap().serve_result = error;
    }

    /// Values passed to successful `serve` calls, oldest first.
    #[must_use]
    pub fn served(&self) -> Vec<Arc<[u8]>> {
        self.state.lock().unwrap().served.clone()
    }

    /// Limits passed to `read_text`, oldest first.
    #[must_use]
    pub fn read_limits(&self) -> Vec<usize> {
        self.state.lock().unwrap().read_limits.clone()
    }

    /// How many times `stop_serving` was called.
    #[must_use]
    pub fn stop_calls(&self) -> usize {
        self.state.lock().unwrap().stops
    }
}

impl Backend for FakeBackend {
    fn events(&mut self) -> &mut EventStream {
        &mut self.events
    }

    async fn read_text(&mut self, limit: usize) -> Result<Vec<u8>, ReadError> {
        let mut s = self.state.lock().unwrap();
        s.read_limits.push(limit);
        match s.reads.pop_front() {
            Some(Ok(v)) if v.len() > limit => Err(ReadError::TooLarge),
            Some(r) => r,
            None => Err(ReadError::NoText),
        }
    }

    async fn serve(&mut self, text: Arc<[u8]>) -> Result<(), ServeError> {
        let mut s = self.state.lock().unwrap();
        if let Some(e) = s.serve_result {
            return Err(e);
        }
        s.served.push(text);
        Ok(())
    }

    fn stop_serving(&mut self) {
        self.state.lock().unwrap().stops += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changed(types: &[&str], self_owned: bool) -> BackendEvent {
        BackendEvent::SelectionChanged {
            types: types.iter().map(|t| (*t).to_string()).collect(),
            self_owned,
        }
    }

    #[tokio::test]
    async fn events_arrive_in_order() {
        let (mut b, h) = FakeBackend::new();
        h.emit(changed(&["UTF8_STRING"], false));
        h.emit(BackendEvent::SelectionCleared);
        h.emit(BackendEvent::Disconnected);
        assert_eq!(
            b.events().next().await,
            Some(changed(&["UTF8_STRING"], false))
        );
        assert_eq!(
            b.events().next().await,
            Some(BackendEvent::SelectionCleared)
        );
        assert_eq!(b.events().next().await, Some(BackendEvent::Disconnected));
        assert_eq!(b.events().try_next(), None);
    }

    #[tokio::test]
    async fn stream_ends_when_the_handle_is_gone() {
        let (mut b, h) = FakeBackend::new();
        drop(h);
        assert_eq!(b.events().next().await, None);
    }

    #[tokio::test]
    async fn reads_follow_the_script_and_the_limit() {
        let (mut b, h) = FakeBackend::new();
        h.push_read(Ok(b"abc".to_vec()));
        h.push_read(Ok(b"abcdef".to_vec()));
        h.push_read(Err(ReadError::ConversionRefused));
        assert_eq!(b.read_text(10).await, Ok(b"abc".to_vec()));
        assert_eq!(b.read_text(5).await, Err(ReadError::TooLarge));
        assert_eq!(b.read_text(5).await, Err(ReadError::ConversionRefused));
        assert_eq!(b.read_text(5).await, Err(ReadError::NoText));
        assert_eq!(h.read_limits(), vec![10, 5, 5, 5]);
    }

    #[tokio::test]
    async fn serves_are_recorded_and_can_fail() {
        let (mut b, h) = FakeBackend::new();
        b.serve(Arc::from(&b"one"[..])).await.unwrap();
        h.fail_serve(Some(ServeError::OwnershipFailed));
        assert_eq!(
            b.serve(Arc::from(&b"two"[..])).await,
            Err(ServeError::OwnershipFailed)
        );
        b.stop_serving();
        assert_eq!(h.served().len(), 1);
        assert_eq!(&*h.served()[0], b"one");
        assert_eq!(h.stop_calls(), 1);
    }

    #[test]
    fn errors_carry_no_content() {
        assert_eq!(
            ReadError::TooLarge.to_string(),
            "selection is larger than the limit"
        );
    }
}
