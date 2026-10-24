//! SC-001 (F8): hinted content never reaches the store, logs or memory past the decision.
//! Models the daemon's per-change flow with a recording reader and a recording store.

use proptest::prelude::*;
use rstory::domain::filter::{CopyFilter, FilterDecision, PreRead, SENSITIVE_HINTS};

const CAP: u64 = 64;

#[derive(Debug, Clone)]
struct Job {
    types: Vec<String>,
    paused: bool,
    content: Vec<u8>,
}

fn job() -> impl Strategy<Value = Job> {
    let ty = prop_oneof![
        Just("UTF8_STRING".to_string()),
        Just("text/plain;charset=utf-8".to_string()),
        Just("TARGETS".to_string()),
        Just(SENSITIVE_HINTS[0].to_string()),
        "[a-zA-Z/-]{1,12}",
    ];
    (
        prop::collection::vec(ty, 0..6),
        any::<bool>(),
        prop::collection::vec(any::<u8>(), 0..160),
    )
        .prop_map(|(types, paused, content)| Job {
            types,
            paused,
            content,
        })
}

fn hinted(j: &Job) -> bool {
    j.types
        .iter()
        .any(|t| SENSITIVE_HINTS.contains(&t.as_str()))
}

/// 10 000 cases unless `PROPTEST_CASES` says otherwise (1 000 on PRs, 100 000 nightly).
fn config() -> ProptestConfig {
    let cases = std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000);
    ProptestConfig {
        cases,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn hinted_content_is_never_read_stored_or_logged(jobs in prop::collection::vec(job(), 0..12)) {
        let filter = CopyFilter::new(CAP);
        let mut reads = 0usize;
        let mut stored: Vec<Vec<u8>> = Vec::new();
        let mut log = String::new();
        let mut expected_reads = 0usize;
        for j in &jobs {
            let pre = filter.decide(&j.types, j.paused);
            if !j.paused && !hinted(j) {
                expected_reads += 1;
            }
            let decision = match pre {
                PreRead::Skip(d) => d,
                PreRead::Read { limit } => {
                    reads += 1;
                    let n = j.content.len().min(limit);
                    filter.after_read(j.content[..n].to_vec())
                }
            };
            log.push_str(&format!("{decision:?}\n"));
            if hinted(j) && !j.paused {
                prop_assert_eq!(&decision, &FilterDecision::SkipSensitive);
            }
            if let FilterDecision::Record(b) = decision {
                prop_assert!(!hinted(j) && !j.paused);
                prop_assert!(!b.is_empty() && b.len() as u64 <= CAP);
                stored.push(b);
            }
        }
        prop_assert_eq!(reads, expected_reads);
        // No hinted job's bytes were stored; the log shows sizes only.
        for j in jobs.iter().filter(|j| hinted(j) && !j.content.is_empty()) {
            prop_assert!(!stored.iter().any(|s| s == &j.content));
        }
        // Decisions print as variant names and sizes: no stored text can appear in the log.
        for s in &stored {
            if let Ok(text) = std::str::from_utf8(s) {
                prop_assert!(text.len() < 3 || !log.contains(text));
            }
        }
    }
}
