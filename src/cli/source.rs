//! Where `ls` and `show` read from: the daemon when it answers, else the store opened read-only
//! (BR14). Writes never come through here: they need the daemon (Constitution IV).
//!
//! The read-only handle is dropped before this module returns, so its shared lock never blocks
//! a daemon that starts meanwhile.

use serde_json::{Value, json};

use super::env::Env;
use super::error::CliError;
use super::{EntryRef, ExitCode};
use crate::ipc::client::{Client, ClientError};
use crate::ipc::protocol::EntrySummary;
use crate::store::StoreError;
use crate::store::readonly::ReadOnlyStore;

/// The listing, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// Newest first.
    pub entries: Vec<EntrySummary>,
    /// True when the store was read directly because no daemon answered.
    pub read_only: bool,
}

/// One entry with its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// The entry.
    pub entry: EntrySummary,
    /// The content as stored (through the daemon: as UTF-8, lossily).
    pub content: Vec<u8>,
    /// True when read directly from the store.
    pub read_only: bool,
}

/// Connects to the daemon. `None` means there is none: no socket, or no runtime directory
/// where one could live.
fn try_connect(env: &Env) -> Result<Option<Client>, CliError> {
    let Ok(path) = env.socket_path() else {
        return Ok(None);
    };
    match Client::connect(&path) {
        Ok(c) => Ok(Some(c)),
        Err(ClientError::NotRunning) => Ok(None),
        Err(e) => Err(CliError::from_client(e, None)),
    }
}

/// Connects to the daemon for a command that cannot work without it.
///
/// # Errors
/// Exit 3 when it is not running; exit 5 when there is no runtime directory.
pub fn require_daemon(env: &Env) -> Result<Client, CliError> {
    let path = env.socket_path()?;
    Client::connect(&path).map_err(|e| CliError::from_client(e, None))
}

fn open_store(env: &Env) -> Result<ReadOnlyStore<crate::store::fs::RealFs>, CliError> {
    let dir = env.store_dir()?;
    ReadOnlyStore::open(&dir).map_err(|e| match e {
        StoreError::DaemonRunning => CliError::environment(
            "The daemon holds the store but its socket does not answer. Run `rstory doctor` for details.",
        ),
        other => CliError::environment(format!("Cannot read the store: {other}")),
    })
}

/// The entries as clients see them, from a read-only store.
#[must_use]
pub fn summaries(store: &ReadOnlyStore<crate::store::fs::RealFs>) -> Vec<EntrySummary> {
    store
        .entries()
        .iter()
        .zip(store.short_ids())
        .enumerate()
        .map(|(i, (e, short))| EntrySummary {
            id: e.id.to_string(),
            short_id: short.to_string(),
            index: u32::try_from(i).unwrap_or(u32::MAX),
            size: e.size,
            created: e.created,
            last_copied: e.last_copied,
            pinned: e.pinned,
            preview: e.preview.as_str().to_string(),
        })
        .collect()
}

/// The history, at most `limit` entries.
///
/// # Errors
/// Exit 5 when neither the daemon nor the store can be read; exit 1 on a bad answer.
pub fn list(env: &Env, limit: Option<u32>) -> Result<Listing, CliError> {
    if let Some(mut client) = try_connect(env)? {
        let args = limit.map_or_else(|| json!({}), |n| json!({"limit": n}));
        let result = client
            .call("list", args)
            .map_err(|e| CliError::from_client(e, None))?;
        return Ok(Listing {
            entries: parse_entries(&result)?,
            read_only: false,
        });
    }
    let store = open_store(env)?;
    let mut entries = summaries(&store);
    drop(store);
    if let Some(n) = limit {
        entries.truncate(n as usize);
    }
    Ok(Listing {
        entries,
        read_only: true,
    })
}

/// One entry and its content, by index or short id.
///
/// # Errors
/// Exit 2 when `reference` matches nothing or several entries; otherwise as [`list`].
pub fn show(env: &Env, reference: &str) -> Result<Shown, CliError> {
    if let Some(mut client) = try_connect(env)? {
        let result = client
            .call("show", json!({"ref": reference}))
            .map_err(|e| CliError::from_client(e, Some(reference)))?;
        let entry = result
            .get("entry")
            .and_then(|v| serde_json::from_value::<EntrySummary>(v.clone()).ok())
            .ok_or_else(bad_answer)?;
        let content = result
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(bad_answer)?
            .as_bytes()
            .to_vec();
        return Ok(Shown {
            entry,
            content,
            read_only: false,
        });
    }
    let parsed: EntryRef = reference
        .parse()
        .map_err(|_| CliError::usage(format!("{reference} is not an index or a short id.")))?;
    let store = open_store(env)?;
    let all = summaries(&store);
    let id = match store.resolve(&parsed) {
        Ok(e) => e.id,
        Err(crate::domain::history::ResolveError::NotFound) => {
            return Err(CliError::not_found(reference));
        }
        Err(crate::domain::history::ResolveError::Ambiguous(ids)) => {
            let names: Vec<String> = all
                .iter()
                .filter(|s| ids.iter().any(|i| i.to_string() == s.id))
                .map(|s| s.short_id.clone())
                .collect();
            return Err(CliError::ambiguous(reference, &names));
        }
    };
    let content = store
        .content(&id)
        .map_err(|e| CliError::environment(format!("Cannot read the entry: {e}")))?;
    drop(store);
    let entry = all
        .into_iter()
        .find(|s| s.id == id.to_string())
        .ok_or_else(bad_answer)?;
    Ok(Shown {
        entry,
        content,
        read_only: true,
    })
}

fn parse_entries(result: &Value) -> Result<Vec<EntrySummary>, CliError> {
    result
        .get("entries")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or_else(bad_answer)
}

fn bad_answer() -> CliError {
    CliError::new(
        ExitCode::Unexpected,
        "The daemon sent an answer rstory does not understand.",
    )
}
