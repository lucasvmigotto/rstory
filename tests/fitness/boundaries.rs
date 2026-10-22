//! F1: module boundaries from architecture section 5.3.
//!
//! Scans every `src/<module>/**/*.rs` for `crate::<other>` references and fails on
//! an edge listed as forbidden ("Must not depend on" column).

use std::fs;
use std::path::{Path, PathBuf};

const ALL: &[&str] = &[
    "domain", "store", "config", "session", "backend", "ipc", "daemon", "tray", "cli", "picker",
    "launch", "doctor", "install", "ui",
];

/// (module, modules it must not depend on).
const FORBIDDEN: &[(&str, &[&str])] = &[
    (
        "domain",
        &[
            "store", "config", "session", "backend", "ipc", "daemon", "tray", "cli", "picker",
            "launch", "doctor", "install", "ui",
        ],
    ),
    (
        "store",
        &["backend", "ipc", "daemon", "tray", "cli", "picker", "ui"],
    ),
    ("session", &["store", "cli", "picker", "tray", "ui"]),
    (
        "backend",
        &["store", "ipc", "daemon", "tray", "cli", "picker", "ui"],
    ),
    ("daemon", &["cli", "picker"]),
    ("ipc", &["store", "backend"]),
    ("tray", &["store", "backend"]),
    ("cli", &["backend", "daemon"]),
    ("picker", &["backend", "daemon"]),
    ("install", &["store", "backend"]),
    (
        "config",
        &[
            "domain", "store", "session", "backend", "ipc", "daemon", "tray", "cli", "picker",
            "launch", "doctor", "install", "ui",
        ],
    ),
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Top-level module names referenced as `crate::<name>` in `src`, ignoring line comments.
fn crate_refs(src: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in src.lines() {
        let line = line.split("//").next().unwrap_or("");
        let mut rest = line;
        while let Some(i) = rest.find("crate::") {
            let before_ok = rest[..i].chars().next_back().is_none_or(|c| !is_ident(c));
            rest = &rest[i + "crate::".len()..];
            if !before_ok {
                continue;
            }
            if let Some(group) = rest.strip_prefix('{') {
                let mut depth = 1;
                let mut at_start = true;
                let mut ident = String::new();
                for c in group.chars() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    if depth == 1 && at_start && is_ident(c) {
                        ident.push(c);
                    } else if !ident.is_empty() {
                        found.push(std::mem::take(&mut ident));
                        at_start = false;
                    }
                    if depth == 1 && c == ',' {
                        at_start = true;
                    }
                }
                if !ident.is_empty() {
                    found.push(ident);
                }
            } else {
                let name: String = rest.chars().take_while(|&c| is_ident(c)).collect();
                if !name.is_empty() {
                    found.push(name);
                }
            }
        }
    }
    found
}

/// Returns `(file, from, to)` for every forbidden edge found in `files` of `module`.
fn violations(module: &str, files: &[(PathBuf, String)]) -> Vec<(PathBuf, String, String)> {
    let Some((_, forbidden)) = FORBIDDEN.iter().find(|(m, _)| *m == module) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (path, src) in files {
        for name in crate_refs(src) {
            if forbidden.contains(&name.as_str()) {
                out.push((path.clone(), module.to_string(), name));
            }
        }
    }
    out
}

#[test]
fn module_boundaries_hold() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    for module in ALL {
        let mut paths = Vec::new();
        rust_files(&root.join(module), &mut paths);
        let files: Vec<_> = paths
            .into_iter()
            .map(|p| {
                let s = fs::read_to_string(&p).unwrap_or_default();
                (p, s)
            })
            .collect();
        bad.extend(violations(module, &files));
    }
    let msg: Vec<String> = bad
        .iter()
        .map(|(f, from, to)| {
            format!(
                "  {}: forbidden edge {from} -> {to} (architecture section 5.3)",
                f.strip_prefix(env!("CARGO_MANIFEST_DIR"))
                    .unwrap_or(f)
                    .display()
            )
        })
        .collect();
    assert!(
        msg.is_empty(),
        "module boundary violations:\n{}",
        msg.join("\n")
    );
}

#[test]
fn every_module_has_a_directory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for m in ALL {
        assert!(
            root.join(m).join("mod.rs").is_file(),
            "missing module src/{m}/mod.rs"
        );
    }
}

#[test]
fn scanner_finds_simple_and_grouped_paths() {
    let src = "use crate::store::Store;\nuse crate::{ipc::Req, domain::{A, B}, config};\n// crate::backend\nlet x = crate::cli::run();";
    let mut refs = crate_refs(src);
    refs.sort();
    assert_eq!(refs, ["cli", "config", "domain", "ipc", "store"]);
}

#[test]
fn detects_a_forbidden_edge() {
    let files = vec![(
        PathBuf::from("src/cli/x.rs"),
        "use crate::backend::Backend;".to_string(),
    )];
    let v = violations("cli", &files);
    assert_eq!(v.len(), 1);
    assert_eq!((v[0].1.as_str(), v[0].2.as_str()), ("cli", "backend"));
    let ok = vec![(
        PathBuf::from("src/cli/x.rs"),
        "use crate::ipc::Client;".to_string(),
    )];
    assert!(violations("cli", &ok).is_empty());
}
