//! SC-002: every `--json` output of the CLI validates against the protocol schema's
//! definitions (`EntrySummary`, `Status`) plus the CLI envelope of `contracts/cli.md`.

#[path = "../support/mod.rs"]
mod support;

use std::fs;
use std::path::PathBuf;

use jsonschema::Validator;
use serde_json::{Value, json};
use support::daemon::{Dirs, Opts, Rig, copy};

fn def(name: &str) -> Validator {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contracts/protocol.schema.json");
    let s: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let wrapped =
        json!({"$schema": s["$schema"], "$defs": s["$defs"], "$ref": format!("#/$defs/{name}")});
    jsonschema::validator_for(&wrapped).unwrap()
}

fn assert_valid(v: &Validator, doc: &Value) {
    let errors: Vec<String> = v.iter_errors(doc).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:?} in {doc}");
}

fn json_of(out: &str) -> Value {
    assert!(out.ends_with('\n'), "one document and a newline");
    serde_json::from_str(out).unwrap()
}

fn assert_entry_list(doc: &Value, read_only: bool, n: usize) {
    assert_eq!(doc["schema"], 1);
    assert_eq!(doc["read_only"], read_only);
    let entries = doc["entries"].as_array().unwrap();
    assert_eq!(entries.len(), n);
    let summary = def("EntrySummary");
    for e in entries {
        assert_valid(&summary, e);
    }
    let keys: Vec<&String> = doc.as_object().unwrap().keys().collect();
    assert_eq!(keys.len(), 3, "no undocumented keys: {keys:?}");
}

#[tokio::test]
async fn ls_show_and_status_json_validate() {
    let dirs = Dirs::new();
    let rig = Rig::start(&dirs, Opts::default()).await;
    let mut c = rig.client().await;
    c.subscribe().await;
    for t in ["alpha", "beta"] {
        copy(&rig.fake, t);
        c.event("history_changed").await;
    }
    let ls = json_of(&dirs.rstory(&["ls", "--json"]).await.out());
    assert_entry_list(&ls, false, 2);

    let show = json_of(&dirs.rstory(&["show", "0", "--json"]).await.out());
    assert_eq!(show["schema"], 1);
    assert_valid(&def("EntrySummary"), &show["entry"]);
    assert_eq!(show["content"], "beta");
    assert_eq!(show.as_object().unwrap().len(), 3);

    let status = json_of(&dirs.rstory(&["status", "--json"]).await.out());
    assert_eq!(status["schema"], 1);
    let mut body = status.clone();
    body.as_object_mut().unwrap().remove("schema");
    assert_valid(&def("Status"), &body);
    drop(c);
    rig.stop().await;

    // the same listing read straight from the store
    let ro = json_of(&dirs.rstory(&["ls", "--json"]).await.out());
    assert_entry_list(&ro, true, 2);
    let show = json_of(&dirs.rstory(&["show", "1", "--json"]).await.out());
    assert_valid(&def("EntrySummary"), &show["entry"]);
    assert_eq!(show["content"], "alpha");
}
