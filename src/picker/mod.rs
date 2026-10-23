//! Terminal picker UI: search, preview, actions (spec 007, ADR-0007).
//!
//! Layout: `state` is the pure core (no I/O); `term` owns the terminal lifecycle. Allowed
//! dependencies (architecture 5.3, fitness F1): `ipc` (client), `store` (read-only API),
//! `config`, `domain` and the design system in `ui`; never `backend` or `daemon`.

pub mod state;
pub mod term;
pub mod app;
pub mod data;
pub mod view;

#[cfg(test)]
mod tests;
