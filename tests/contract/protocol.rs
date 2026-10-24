//! T002/T003: every fixture validates, the examples are the contract's, and what the code
//! produces validates too.

use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::Validator;
use rstory::ipc::error::{ErrorCode, ProtoError};
use rstory::ipc::protocol::{
    Command, Counters, EntrySummary, Event, State, StateKind, Status, parse_request, response_err,
    response_ok,
};
use serde_json::{Value, json};
use time::macros::datetime;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn schema() -> Value {
    let text = fs::read_to_string(root().join("contracts/protocol.schema.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// A validator for the whole schema (`None`) or for one of its definitions.
fn validator(def: Option<&str>) -> Validator {
    let mut s = schema();
    if let Some(name) = def {
        let defs = s["$defs"].clone();
        s = json!({"$schema": s["$schema"], "$defs": defs, "$ref": format!("#/$defs/{name}")});
    }
    jsonschema::validator_for(&s).unwrap()
}

fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect()
}

fn fixture_files(dir: &str) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(root().join("tests/fixtures/protocol").join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    files
}

#[test]
fn examples_are_the_contracts_examples_verbatim() {
    let md = fs::read_to_string(root().join("contracts/protocol.md")).unwrap();
    let block = md
        .split("```jsonl\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("the contract has a jsonl examples block");
    let fixture =
        fs::read_to_string(root().join("tests/fixtures/protocol/examples.jsonl")).unwrap();
    assert_eq!(fixture, block);
    assert_eq!(
        fixture.lines().count(),
        26,
        "one request and one answer per op, plus events"
    );
}

#[test]
fn every_example_validates() {
    let v = validator(None);
    for line in lines(&root().join("tests/fixtures/protocol/examples.jsonl")) {
        let value: Value = serde_json::from_str(&line).unwrap();
        assert!(v.is_valid(&value), "example does not validate: {line}");
    }
}

#[test]
fn example_results_validate_against_their_definitions() {
    let status = validator(Some("Status"));
    let state = validator(Some("State"));
    let entry = validator(Some("EntrySummary"));
    let mut checked = 0;
    for line in lines(&root().join("tests/fixtures/protocol/examples.jsonl")) {
        let value: Value = serde_json::from_str(&line).unwrap();
        let Some(result) = value.get("result") else {
            continue;
        };
        if result.get("protocol").is_some() {
            assert!(status.is_valid(result), "{line}");
            checked += 1;
        }
        if let Some(s) = result.get("state").filter(|s| s.is_object()) {
            assert!(state.is_valid(s), "{line}");
            checked += 1;
        }
        for e in result["entries"].as_array().into_iter().flatten() {
            assert!(entry.is_valid(e), "{line}");
            checked += 1;
        }
        if let Some(e) = result.get("entry") {
            assert!(entry.is_valid(e), "{line}");
            checked += 1;
        }
    }
    assert!(checked >= 7, "checked {checked}");
}

#[test]
fn valid_fixtures_validate_and_cover_every_op_and_event() {
    let v = validator(None);
    let names: Vec<String> = fixture_files("valid")
        .iter()
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    for want in [
        "list",
        "show",
        "restore",
        "pin",
        "unpin",
        "rm",
        "clear",
        "pause",
        "resume",
        "status",
        "open_picker",
        "subscribe",
        "history_changed",
        "pause_changed",
    ] {
        assert!(
            names.iter().any(|n| n == want),
            "no valid fixture for {want}"
        );
    }
    for path in fixture_files("valid") {
        for line in lines(&path) {
            let value: Value = serde_json::from_str(&line).unwrap();
            assert!(v.is_valid(&value), "{}: {line}", path.display());
        }
    }
}

#[test]
fn invalid_fixtures_do_not_validate() {
    let v = validator(None);
    let files = fixture_files("invalid");
    assert!(files.len() >= 10);
    for path in files {
        for line in lines(&path) {
            let value: Value = serde_json::from_str(&line).unwrap();
            assert!(
                !v.is_valid(&value),
                "{} should be invalid: {line}",
                path.display()
            );
        }
    }
}

#[test]
fn invalid_results_do_not_validate_against_their_definitions() {
    for (dir, def) in [
        ("invalid_status", "Status"),
        ("invalid_entry", "EntrySummary"),
    ] {
        let v = validator(Some(def));
        let files = fixture_files(dir);
        assert!(files.len() >= 2, "{dir}");
        for path in files {
            for line in lines(&path) {
                let value: Value = serde_json::from_str(&line).unwrap();
                assert!(!v.is_valid(&value), "{} should be invalid", path.display());
            }
        }
    }
}

#[test]
fn request_fixtures_parse_into_commands() {
    let mut parsed = Vec::new();
    for path in fixture_files("valid") {
        for line in lines(&path) {
            let value: Value = serde_json::from_str(&line).unwrap();
            if value.get("op").is_some() {
                let req = parse_request(line.as_bytes())
                    .unwrap_or_else(|e| panic!("{}: {line}: {e:?}", path.display()));
                assert_eq!(Some(req.id), value["id"].as_u64());
                parsed.push(req.command);
            }
        }
    }
    assert_eq!(parsed.len(), 12, "one request per op");
    assert!(
        parsed
            .iter()
            .any(|c| matches!(c, Command::Pause { duration: Some(d) } if d.as_secs() == 50))
    );
    assert!(
        parsed
            .iter()
            .any(|c| matches!(c, Command::List { limit: Some(2) }))
    );
}

#[test]
fn bad_requests_get_bad_request_with_their_id() {
    let v = validator(None);
    let mut n = 0;
    for line in lines(&root().join("tests/fixtures/protocol/server/bad_requests.jsonl")) {
        let value: Value = serde_json::from_str(&line).unwrap();
        let err = parse_request(line.as_bytes()).expect_err(&line);
        assert_eq!(err.error.code, ErrorCode::BadRequest, "{line}");
        assert_eq!(Some(err.id), value["id"].as_u64(), "{line}");
        assert!(v.is_valid(&response_err(err.id, &err.error)), "{line}");
        n += 1;
    }
    assert_eq!(n, 12);
}

#[test]
fn malformed_and_unversioned_requests_are_refused_without_a_panic() {
    let e = parse_request(b"{not json").unwrap_err();
    assert_eq!((e.id, e.error.code), (0, ErrorCode::BadRequest));
    let e = parse_request(b"[1]").unwrap_err();
    assert_eq!(e.error.code, ErrorCode::BadRequest);
    let e = parse_request(br#"{"v":2,"id":9,"op":"list"}"#).unwrap_err();
    assert_eq!((e.id, e.error.code), (9, ErrorCode::UnsupportedVersion));
    let e = parse_request(br#"{"id":9,"op":"list"}"#).unwrap_err();
    assert_eq!(e.error.code, ErrorCode::BadRequest);
    // Unknown fields are ignored; missing args mean no args.
    let ok = parse_request(br#"{"v":1,"id":1,"op":"status","extra":true}"#).unwrap();
    assert_eq!(ok.command, Command::Status);
}

fn sample_state(kind: StateKind) -> State {
    State {
        state: kind,
        paused_until: (kind == StateKind::Paused).then_some(datetime!(2026-10-21 14:06:00 UTC)),
        reason: (kind == StateKind::Inactive).then(|| "no_session".to_string()),
    }
}

#[test]
fn what_the_code_serializes_validates() {
    let root_v = validator(None);
    let entry_v = validator(Some("EntrySummary"));
    let state_v = validator(Some("State"));
    let status_v = validator(Some("Status"));

    let entry = EntrySummary {
        id: "01JBD8Z0R5S7K3M2N1P0Q9T8V7".to_string(),
        short_id: "01JB".to_string(),
        index: 0,
        size: 38,
        created: datetime!(2026-10-21 14:02:11 UTC),
        last_copied: datetime!(2026-10-21 14:02:11 UTC),
        pinned: false,
        preview: "synthetic".to_string(),
    };
    let entry_json = serde_json::to_value(&entry).unwrap();
    assert!(entry_v.is_valid(&entry_json));

    for kind in [StateKind::Recording, StateKind::Paused, StateKind::Inactive] {
        let state = sample_state(kind);
        let state_json = serde_json::to_value(&state).unwrap();
        assert!(state_v.is_valid(&state_json), "{state_json}");
        assert!(root_v.is_valid(&response_ok(1, json!({"state": state_json}))));
        assert!(root_v.is_valid(&Event::pause_changed(&state).to_value()));
        let status = Status {
            state,
            backend: "x11".to_string(),
            mode: Some("native".to_string()),
            entries: 3,
            pinned: 1,
            max_entries: 20,
            max_pinned: 10,
            counters: Counters::default(),
            tray: "disabled".to_string(),
            picker_available: false,
            version: "0.1.0".to_string(),
            protocol: 1,
        };
        let status_json = serde_json::to_value(&status).unwrap();
        assert!(status_v.is_valid(&status_json), "{status_json}");
        assert!(root_v.is_valid(&response_ok(2, status_json)));
    }
    assert!(root_v.is_valid(&Event::history_changed(0, None).to_value()));
    assert!(root_v.is_valid(&Event::history_changed(2, Some(entry.id.clone())).to_value()));
    assert!(root_v.is_valid(&response_ok(3, json!({"entries": [entry_json]}))));
}

#[test]
fn every_error_code_validates_in_a_response() {
    let v = validator(None);
    for code in [
        ErrorCode::BadRequest,
        ErrorCode::TooLarge,
        ErrorCode::UnsupportedVersion,
        ErrorCode::Internal,
        ErrorCode::NotFound,
        ErrorCode::Ambiguous,
        ErrorCode::PinLimit,
        ErrorCode::Inactive,
        ErrorCode::BackendError,
        ErrorCode::NoTerminal,
        ErrorCode::PickerCommandFailed,
    ] {
        let e = ProtoError::new(code, "message");
        assert!(v.is_valid(&response_err(4, &e)), "{code}");
    }
}
