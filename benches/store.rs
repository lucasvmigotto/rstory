//! Store benchmarks (SC-003), plain `std::time`, no framework.
//!
//! Budgets: open + list 500 < 10 ms; one insert <= 20 ms; a batch of N <= 20 ms + N x 1 ms;
//! read-only open + list 500 < 10 ms. A scenario whose median is over budget is flagged; the
//! process exits non-zero only if a p95 is more than 2x its budget (CI headroom, test-strategy
//! section 5). The inserts are fsync-bound: the numbers depend on the disk, so the filesystem
//! under test is printed. Set `BENCH_DIR` to benchmark a specific disk instead of `/tmp`.
//! Fixtures are synthetic; no real clipboard content.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::store::fs::RealFs;
use rstory::store::readonly::ReadOnlyStore;
use rstory::store::{Mutation, Store};
use time::OffsetDateTime;

// 4_102_444_800 is 2100-01-01: far in the future, so reopening a store never expires its
// entries against the wall clock (a future timestamp counts as "now" for expiry).
const BASE: i64 = 4_102_444_800;

struct Stats {
    median: Duration,
    p95: Duration,
}

fn stats(mut samples: Vec<Duration>) -> Stats {
    samples.sort();
    let n = samples.len();
    Stats {
        median: samples[n / 2],
        p95: samples[(n * 95).div_ceil(100) - 1],
    }
}

/// Runs `f` `warmup` times, then `iters` times, timing each call.
fn measure(warmup: usize, iters: usize, mut f: impl FnMut()) -> Stats {
    // Measure the store, not the host's writeback: right after a build (every CI run) the first
    // fsyncs can wait on the compiler's dirty pages, and a batch scenario has only 20 samples.
    let flush = || {
        let _ = std::process::Command::new("sync").status();
    };
    flush();
    for _ in 0..warmup {
        f();
    }
    flush();
    let samples = (0..iters)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed()
        })
        .collect();
    stats(samples)
}

fn payload(counter: u64, size: usize) -> Content {
    let mut bytes = format!("bench payload {counter} ").into_bytes();
    while bytes.len() < size {
        bytes.push(b'a' + u8::try_from(bytes.len() % 26).unwrap_or(0));
    }
    Content::new(bytes, 1 << 20).unwrap()
}

fn at(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(BASE + secs).unwrap()
}

fn caps(max_entries: u32) -> Caps {
    Caps {
        max_entries,
        max_pinned: 10.min(max_entries - 1),
        ..Caps::default()
    }
}

/// A store directory holding `n` entries of 4 KiB each.
fn fixture(root: &Path, name: &str, n: u32) -> PathBuf {
    let dir = root.join(name);
    let mut s = Store::open_exclusive(&dir, caps(n.max(20)), RealFs).unwrap();
    for chunk in (0..u64::from(n)).collect::<Vec<_>>().chunks(100) {
        let batch = chunk
            .iter()
            .map(|&i| Mutation::RecordCopy(payload(i, 4096)))
            .collect();
        s.apply(batch, at(0)).unwrap();
    }
    assert_eq!(s.entries().len(), n as usize);
    dir
}

/// What `ls` does with the handle: touch every row's listing fields.
fn list<I: Iterator<Item = usize>>(lens: I) -> usize {
    lens.sum()
}

fn mount_of(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mounts = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    mounts
        .lines()
        .filter_map(|l| {
            let mut f = l.split_whitespace();
            let (dev, mp, ty) = (f.next()?, f.next()?, f.next()?);
            path.starts_with(mp)
                .then(|| (mp.len(), format!("{dev} on {mp} type {ty}")))
        })
        .max()
        .map_or_else(|| "unknown".to_string(), |(_, s)| s)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

struct Row {
    name: &'static str,
    budget: Duration,
    stats: Stats,
}

fn main() -> ExitCode {
    let root = std::env::var_os("BENCH_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let tmp = tempfile::tempdir_in(&root).expect("bench directory");
    let base = tmp.path();
    println!("filesystem under test: {}", mount_of(base));
    let ten = Duration::from_millis(10);
    let mut rows = Vec::new();

    for n in [20u32, 500] {
        let dir = fixture(base, &format!("open{n}"), n);
        let c = caps(n.max(20));
        let stats = measure(3, 30, || {
            let s = Store::open_exclusive(&dir, c, RealFs).unwrap();
            assert!(list(s.entries().iter().map(|e| e.preview.as_str().len())) > 0);
            assert_eq!(s.entries().len(), n as usize);
        });
        rows.push(Row {
            name: if n == 20 {
                "open_exclusive + list, 20 entries"
            } else {
                "open_exclusive + list, 500 entries"
            },
            budget: ten,
            stats,
        });
    }

    {
        let dir = fixture(base, "ro500", 500);
        let stats = measure(3, 30, || {
            let s = ReadOnlyStore::open_with(&dir, caps(500), RealFs).unwrap();
            assert_eq!(s.entries().len(), 500);
            assert!(list(s.entries().iter().map(|e| e.preview.as_str().len())) > 0);
        });
        rows.push(Row {
            name: "ReadOnlyStore::open + list, 500 entries",
            budget: ten,
            stats,
        });
    }

    {
        let dir = base.join("single");
        let mut s = Store::open_exclusive(&dir, caps(20), RealFs).unwrap();
        let mut counter = 0u64;
        for _ in 0..20 {
            counter += 1;
            s.apply(vec![Mutation::RecordCopy(payload(counter, 4096))], at(0))
                .unwrap();
        }
        // Steady state: the history is full, so each insert also evicts and unlinks one entry.
        let stats = measure(5, 50, || {
            counter += 1;
            s.apply(
                vec![Mutation::RecordCopy(payload(counter, 4096))],
                at(counter as i64),
            )
            .unwrap();
        });
        rows.push(Row {
            name: "apply(RecordCopy 4 KiB), full history",
            budget: Duration::from_millis(20),
            stats,
        });
    }

    {
        let dir = base.join("batch");
        let mut s = Store::open_exclusive(&dir, caps(20), RealFs).unwrap();
        let mut counter = 0u64;
        let stats = measure(3, 20, || {
            let batch = (0..20)
                .map(|_| {
                    counter += 1;
                    Mutation::RecordCopy(payload(counter, 4096))
                })
                .collect();
            s.apply(batch, at(counter as i64)).unwrap();
        });
        rows.push(Row {
            name: "apply(batch of 20 RecordCopy 4 KiB)",
            budget: Duration::from_millis(40),
            stats,
        });
    }

    println!(
        "{:<44} {:>10} {:>10} {:>10}  verdict",
        "scenario", "median ms", "p95 ms", "budget ms"
    );
    let mut failed = false;
    for r in &rows {
        let verdict = if r.stats.p95 > r.budget * 2 {
            failed = true;
            "FAIL (p95 > 2x budget)"
        } else if r.stats.median > r.budget {
            "over budget (within 2x headroom)"
        } else {
            "ok"
        };
        println!(
            "{:<44} {:>10.2} {:>10.2} {:>10.0}  {verdict}",
            r.name,
            ms(r.stats.median),
            ms(r.stats.p95),
            ms(r.budget)
        );
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
