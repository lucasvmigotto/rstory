//! Picker budgets (F6): first frame p95 under 100 ms with 20 and 500 entries (SC-001), and
//! keystroke to redraw p95 under 16 ms with 500 entries (SC-002).
//!
//! A first-frame sample is everything `rstory pick` does before its first draw, in process:
//! open the source (the daemon socket, or the store read-only), list, build the state, fetch
//! the selection's content, and render 80x24 into a buffer. Process start and the terminal's
//! own paint are not included (no real terminal is opened). A median over budget is flagged;
//! the run fails only when a p95 is more than 2x its budget (test-strategy section 5).
//! Fixtures are synthetic.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rstory::config::Caps;
use rstory::domain::entry::Content;
use rstory::picker::app::step;
use rstory::picker::data::{DaemonSource, ReadOnlySource, Source};
use rstory::picker::state::{Key, Msg, PickerState, update};
use rstory::picker::view;
use rstory::store::fs::RealFs;
use rstory::store::{Mutation, Store};
use rstory::ui::glyphs::Glyphs;
use rstory::ui::theme::{Background, Theme, Tier};
use rstory::ui::widgets::Ctx;
use time::OffsetDateTime;

const FIRST_FRAME: Duration = Duration::from_millis(100);
const KEYSTROKE: Duration = Duration::from_millis(16);
const SIZE: (u16, u16) = (80, 24);
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
        .args(args);
    cmd
}

/// `n` entries of about 1 KiB under `<root>/state/rstory`, and a config that keeps them.
fn fixture(root: &Path, n: u32) {
    for dir in ["run", "config/rstory", "home", "state"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::set_permissions(
        root.join("run"),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .unwrap();
    std::fs::write(
        root.join("config/rstory/config.toml"),
        format!("max_entries = {}\n", n.max(20)),
    )
    .unwrap();
    let caps = Caps {
        max_entries: n.max(20),
        ..Caps::default()
    };
    let mut store = Store::open_exclusive(&root.join("state/rstory"), caps, RealFs).unwrap();
    let all: Vec<u64> = (0..u64::from(n)).collect();
    for chunk in all.chunks(100) {
        let batch = chunk
            .iter()
            .map(|i| {
                let mut bytes = format!("synthetic bench entry {i} git rebase ").into_bytes();
                bytes.resize(1024, b'x');
                Mutation::RecordCopy(Content::new(bytes, 1 << 20).unwrap())
            })
            .collect();
        store
            .apply(batch, OffsetDateTime::from_unix_timestamp(BASE).unwrap())
            .unwrap();
    }
    assert_eq!(store.entries().len(), n as usize);
}

fn render(state: &PickerState) -> Buffer {
    let theme = Theme::new(Tier::Ansi16, Background::Dark);
    let glyphs = Glyphs::new(false);
    let area = Rect::new(0, 0, SIZE.0, SIZE.1);
    let mut buf = Buffer::empty(area);
    view::render(state, area, &mut buf, Ctx::new(&theme, &glyphs));
    buf
}

/// One first frame from `open`, checked for `n` rows.
fn first_frame(open: &dyn Fn() -> Box<dyn Source>, n: usize) -> Duration {
    let t = Instant::now();
    let mut source = open();
    let snap = source.list().unwrap();
    let mut state = PickerState::new(snap, source.read_only(), SIZE);
    step(
        &mut state,
        &mut *source,
        Msg::Resize {
            width: SIZE.0,
            height: SIZE.1,
        },
    );
    let buf = render(&state);
    let took = t.elapsed();
    assert_eq!(state.entries().len(), n);
    assert!(state.selected_content().is_some(), "content fetched");
    assert!(buf.area.width == SIZE.0);
    took
}

fn stats(mut all: Vec<Duration>) -> (Duration, Duration) {
    all.sort();
    (all[all.len() / 2], all[(all.len() * 95).div_ceil(100) - 1])
}

fn measure(open: &dyn Fn() -> Box<dyn Source>, n: usize) -> (Duration, Duration) {
    for _ in 0..3 {
        first_frame(open, n);
    }
    stats((0..30).map(|_| first_frame(open, n)).collect())
}

/// Typing a query and erasing it again, one redraw per key, over the read-only listing.
fn keystrokes(dir: PathBuf) -> (Duration, Duration) {
    let mut source = ReadOnlySource::new(dir);
    let snap = source.list().unwrap();
    let mut state = PickerState::new(snap, true, SIZE);
    let mut all = Vec::new();
    for round in 0..10 {
        let keys = "rebase 4"
            .chars()
            .map(Key::Char)
            .chain(std::iter::repeat_n(Key::Backspace, 8));
        for k in keys {
            let t = Instant::now();
            update(&mut state, Msg::Key(k));
            render(&state);
            if round > 0 {
                all.push(t.elapsed());
            }
        }
    }
    stats(all)
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
    let mut rows: Vec<(String, (Duration, Duration), Duration)> = Vec::new();
    for n in [20u32, 500] {
        let tmp = tempfile::tempdir_in(&base).expect("bench directory");
        let root = tmp.path().to_path_buf();
        fixture(&root, n);
        let store = root.join("state/rstory");

        let dir = store.clone();
        let ro = measure(
            &move || Box::new(ReadOnlySource::new(dir.clone())),
            n as usize,
        );
        rows.push((
            format!("first frame, {n} entries, read-only"),
            ro,
            FIRST_FRAME,
        ));
        if n == 500 {
            rows.push((
                "keystroke to redraw, 500 entries".into(),
                keystrokes(store.clone()),
                KEYSTROKE,
            ));
        }

        let mut daemon = start_daemon(&root);
        let socket = root.join("run/rstory/control.sock");
        let d = measure(
            &move || Box::new(DaemonSource::connect(&socket).unwrap()),
            n as usize,
        );
        rows.push((format!("first frame, {n} entries, socket"), d, FIRST_FRAME));
        daemon.kill().unwrap();
        daemon.wait().unwrap();
    }

    let mut failed = false;
    for (name, (median, p95), budget) in rows {
        let flag = if median > budget { "  OVER BUDGET" } else { "" };
        println!(
            "{name:<40} median {:>7.2} ms  p95 {:>7.2} ms  (budget {} ms){flag}",
            ms(median),
            ms(p95),
            budget.as_millis()
        );
        failed |= p95 > budget * 2;
    }
    if failed {
        eprintln!("a p95 is more than 2x its budget");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
