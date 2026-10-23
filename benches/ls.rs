//! `rstory ls` budget (SC-001): under 50 ms with 500 entries, through the socket and read-only.
//!
//! Each sample is a whole process run (`exec` included), the way a script would call it. The
//! daemon is the real binary with no display session, so it starts inactive but serves its
//! history. A median over budget is flagged; the process exits non-zero only if a p95 is more
//! than 2x the budget (CI headroom, test-strategy section 5). Fixtures are synthetic.

use std::path::Path;
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::store::fs::RealFs;
use rstory::store::{Mutation, Store};
use time::OffsetDateTime;

const ENTRIES: u32 = 500;
const BUDGET: Duration = Duration::from_millis(50);
// 2100-01-01: far in the future, so reopening the store never expires its entries.
const BASE: i64 = 4_102_444_800;

fn rstory(root: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rstory"));
    cmd.env_clear()
        .env("XDG_RUNTIME_DIR", root.join("run"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("HOME", root.join("home"))
        .env("LANG", "C.UTF-8")
        .env("NO_COLOR", "1")
        .args(args);
    cmd
}

/// 500 entries of 4 KiB in `<root>/state/rstory`, and a config that keeps all of them.
fn fixture(root: &Path) {
    for dir in ["run", "config/rstory", "home"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::set_permissions(
        root.join("run"),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .unwrap();
    std::fs::write(
        root.join("config/rstory/config.toml"),
        format!("max_entries = {ENTRIES}\n"),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("state")).unwrap();
    let caps = Caps {
        max_entries: ENTRIES,
        ..Caps::default()
    };
    let mut store = Store::open_exclusive(&root.join("state/rstory"), caps, RealFs).unwrap();
    let all: Vec<u64> = (0..u64::from(ENTRIES)).collect();
    for chunk in all.chunks(100) {
        let batch = chunk
            .iter()
            .map(|i| {
                let mut bytes = format!("bench payload {i} ").into_bytes();
                bytes.resize(4096, b'a');
                Mutation::RecordCopy(Content::new(bytes, 1 << 20).unwrap())
            })
            .collect();
        let at = OffsetDateTime::from_unix_timestamp(BASE).unwrap();
        store.apply(batch, at).unwrap();
    }
    assert_eq!(store.entries().len(), ENTRIES as usize);
}

fn sample(root: &Path, expect_read_only: bool) -> Duration {
    let t = Instant::now();
    let out = rstory(root, &["ls"]).output().unwrap();
    let took = t.elapsed();
    assert!(out.status.success(), "ls failed");
    assert_eq!(
        out.stdout.iter().filter(|b| **b == b'\n').count(),
        ENTRIES as usize
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).contains("read-only"),
        expect_read_only
    );
    took
}

fn measure(root: &Path, expect_read_only: bool) -> (Duration, Duration) {
    for _ in 0..3 {
        sample(root, expect_read_only);
    }
    let mut all: Vec<Duration> = (0..30).map(|_| sample(root, expect_read_only)).collect();
    all.sort();
    (all[all.len() / 2], all[(all.len() * 95).div_ceil(100) - 1])
}

fn start_daemon(root: &Path) -> Child {
    let mut child = rstory(root, &["daemon"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let up = rstory(root, &["status"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if up {
            return child;
        }
        assert!(Instant::now() < end, "the daemon did not come up");
        assert!(child.try_wait().unwrap().is_none(), "the daemon exited");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn main() -> ExitCode {
    let base = std::env::var_os("BENCH_DIR").map_or_else(std::env::temp_dir, Into::into);
    let tmp = tempfile::tempdir_in(base).expect("bench directory");
    let root = tmp.path();
    fixture(root);

    let read_only = measure(root, true);
    let mut daemon = start_daemon(root);
    let socket = measure(root, false);
    daemon.kill().unwrap();
    daemon.wait().unwrap();

    let mut failed = false;
    for (name, (median, p95)) in [
        ("ls, 500 entries, read-only", read_only),
        ("ls, 500 entries, via the socket", socket),
    ] {
        let flag = if median > BUDGET { "  OVER BUDGET" } else { "" };
        println!(
            "{name:<34} median {:>7.2} ms  p95 {:>7.2} ms  (budget {} ms){flag}",
            ms(median),
            ms(p95),
            BUDGET.as_millis()
        );
        failed |= p95 > BUDGET * 2;
    }
    if failed {
        eprintln!("a p95 is more than 2x its budget");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
