//! E2E-K6 Scripting (specs/006 qa.md): `ls --json` validates against the protocol schema,
//! `show <shortid>` piped gives the exact bytes, `clear` without a TTY exits 2 and with
//! `--yes` empties the unpinned history. stdin and stdout are pipes throughout.

use std::path::PathBuf;

use jsonschema::Validator;
use serde_json::{Value, json};

use crate::support::World;

const FIRST: &str = "k6 first\nline two with ünïcode ✓\n";
const SECOND: &str = "k6 second";

fn def(name: &str) -> Validator {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contracts/protocol.schema.json");
    let s: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let wrapped =
        json!({"$schema": s["$schema"], "$defs": s["$defs"], "$ref": format!("#/$defs/{name}")});
    jsonschema::validator_for(&wrapped).unwrap()
}

fn assert_valid(v: &Validator, doc: &Value) {
    let errors: Vec<String> = v.iter_errors(doc).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?} in {doc}");
}

#[test]
fn k6_scripting_json_bytes_and_exit_codes() {
    let w = World::xvfb();
    let _daemon = w.start_daemon();
    w.copy(FIRST);
    w.wait_count(1);
    w.copy(SECOND);
    w.wait_top(SECOND);

    // ls --json | jq: one JSON document, the envelope and every entry valid.
    let r = w.run(&["ls", "--json"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.out().ends_with('\n'));
    let doc = r.json();
    assert_eq!(doc["schema"], 1);
    assert_eq!(doc["read_only"], false);
    let entries = doc["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    let summary = def("EntrySummary");
    for e in entries {
        assert_valid(&summary, e);
    }
    assert_eq!(entries[0]["preview"], SECOND);
    assert_eq!(entries[0]["index"], 0);
    assert_valid(&def("Status"), &w.status());

    // show <shortid> piped: the bytes, exactly, with nothing added.
    let short = entries[1]["short_id"].as_str().unwrap().to_string();
    let shown = w.run(&["show", &short]);
    assert_eq!(shown.code, 0, "{}", shown.stderr);
    assert_eq!(shown.stdout, FIRST.as_bytes());
    assert_eq!(w.run(&["show", "0"]).stdout, SECOND.as_bytes());

    // Exit codes per contract.
    assert_eq!(w.run(&["show", "zzzzzz"]).code, 2, "unknown ref");
    assert_eq!(w.run(&["clear"]).code, 2, "no TTY, no --yes");
    assert_eq!(w.entries().len(), 2, "a refused clear removes nothing");
    let cleared = w.run(&["clear", "--yes"]);
    assert_eq!(cleared.code, 0, "{}", cleared.stderr);
    assert!(w.entries().is_empty());
    let empty = w.json(&["ls", "--json"]);
    assert_eq!(empty["entries"], json!([]));
}

#[test]
fn k6_without_a_daemon_reads_succeed_and_writes_exit_3() {
    let w = World::xvfb();
    let mut d = w.start_daemon();
    w.copy(SECOND);
    w.wait_top(SECOND);
    d.stop();

    let ls = w.run(&["ls", "--json"]);
    assert_eq!(ls.code, 0, "{}", ls.stderr);
    assert_eq!(ls.json()["read_only"], true);
    assert_eq!(w.run(&["show", "0"]).stdout, SECOND.as_bytes());
    assert_eq!(w.run(&["copy", "0"]).code, 3);
    assert_eq!(w.run(&["clear", "--yes"]).code, 3);
}
