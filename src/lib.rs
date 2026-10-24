//! rstory: a small, private clipboard history for Linux.
//!
//! One crate, one multi-call binary; module boundaries follow
//! `docs/product/architecture.md` section 5.3 and are enforced by
//! `tests/fitness/boundaries.rs`.

#![forbid(unsafe_code)]

pub mod backend;
pub mod cli;
pub mod config;
pub mod daemon;
pub mod doctor;
pub mod domain;
pub mod install;
pub mod ipc;
pub mod launch;
pub mod picker;
pub mod session;
pub mod store;
pub mod tray;
pub mod ui;
