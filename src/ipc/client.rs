//! The blocking protocol client used by the CLI.

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use serde_json::{Value, json};

    use super::*;
    use crate::ipc::error::ErrorCode;

    /// A server that answers each request line with `reply(request)`; `None` stays silent.
    fn serve(
        reply: impl Fn(&Value) -> Option<String> + Send + 'static,
    ) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        let listener = UnixListener::bind(&path).unwrap();
        thread::spawn(move || {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut out = stream.try_clone().unwrap();
            for line in BufReader::new(stream).lines() {
                let Ok(line) = line else { return };
                let req: Value = serde_json::from_str(&line).unwrap();
                if let Some(text) = reply(&req) {
                    out.write_all(text.as_bytes()).unwrap();
                    out.write_all(b"\n").unwrap();
                }
            }
        });
        (dir, path)
    }

    fn ok(req: &Value, result: Value) -> Option<String> {
        Some(json!({"v":1,"id":req["id"],"ok":true,"result":result}).to_string())
    }

    #[test]
    fn a_missing_socket_means_the_daemon_is_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let err = Client::connect(&dir.path().join("none.sock")).unwrap_err();
        assert!(matches!(err, ClientError::NotRunning), "{err:?}");
    }

    #[test]
    fn a_stale_socket_file_means_the_daemon_is_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        drop(UnixListener::bind(&path).unwrap());
        let err = Client::connect(&path).unwrap_err();
        assert!(matches!(err, ClientError::NotRunning), "{err:?}");
    }

    #[test]
    fn a_call_sends_the_envelope_and_returns_the_result() {
        let (_d, path) = serve(|req| {
            assert_eq!(req["v"], 1);
            assert_eq!(req["op"], "show");
            assert_eq!(req["args"], json!({"ref": "1"}));
            ok(req, json!({"content": "x"}))
        });
        let mut c = Client::connect(&path).unwrap();
        assert_eq!(
            c.call("show", json!({"ref": "1"})).unwrap(),
            json!({"content": "x"})
        );
    }

    #[test]
    fn ids_count_up_and_events_are_skipped() {
        let (_d, path) = serve(|req| {
            let id = req["id"].as_u64().unwrap();
            // an event first, then the answer on the same write
            Some(format!(
                "{}\n{}",
                json!({"v":1,"event":"history_changed","data":{"entries":1,"top":null}}),
                json!({"v":1,"id":id,"ok":true,"result":{"n":id}})
            ))
        });
        let mut c = Client::connect(&path).unwrap();
        assert_eq!(c.call("status", json!({})).unwrap()["n"], 1);
        assert_eq!(c.call("status", json!({})).unwrap()["n"], 2);
    }

    #[test]
    fn a_remote_error_keeps_its_code_and_message() {
        let (_d, path) = serve(|req| {
            Some(json!({"v":1,"id":req["id"],"ok":false,"error":{"code":"pin_limit","message":"pin limit reached (10)"}}).to_string())
        });
        let mut c = Client::connect(&path).unwrap();
        match c.call("pin", json!({"ref": "0"})).unwrap_err() {
            ClientError::Remote(e) => {
                assert_eq!(e.code, ErrorCode::PinLimit);
                assert_eq!(e.message, "pin limit reached (10)");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn silence_times_out() {
        let (_d, path) = serve(|_| None);
        let mut c = Client::connect(&path).unwrap();
        c.set_timeout(Some(Duration::from_millis(50)));
        let err = c.call("status", json!({})).unwrap_err();
        assert!(matches!(err, ClientError::Timeout), "{err:?}");
    }

    #[test]
    fn garbage_is_a_protocol_error() {
        let (_d, path) = serve(|_| Some("not json".to_string()));
        let mut c = Client::connect(&path).unwrap();
        let err = c.call("status", json!({})).unwrap_err();
        assert!(matches!(err, ClientError::Protocol(_)), "{err:?}");
    }

    #[test]
    fn a_closed_connection_is_a_protocol_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        let l = UnixListener::bind(&path).unwrap();
        thread::spawn(move || drop(l.accept()));
        let mut c = Client::connect(&path).unwrap();
        let err = c.call("status", json!({})).unwrap_err();
        assert!(matches!(err, ClientError::Protocol(_)), "{err:?}");
    }

    #[test]
    fn restore_waits_longer_than_other_calls() {
        assert_eq!(timeout_for("list"), Duration::from_secs(2));
        assert_eq!(timeout_for("restore"), Duration::from_secs(5));
    }
}
