//! End-to-end journeys (docs/product/test-strategy.md §4, specs/*/qa.md): the real `rstory`
//! binary in headless sessions inside the tools container, with temporary XDG directories and
//! synthetic clipboard text. Run by `make test-e2e`. Nothing here touches the host session.

#![allow(dead_code)]

// The shared launchers (Xvfb, headless sway, headless mutter) and test clients.
#[path = "../support/mod.rs"]
mod session;
mod support;

#[path = "E2E-K1.rs"]
mod k1;
#[path = "E2E-K2.rs"]
mod k2;
#[path = "E2E-K3.rs"]
mod k3;
#[path = "E2E-K3b.rs"]
mod k3b;
#[path = "E2E-K4.rs"]
mod k4;
#[path = "E2E-K6.rs"]
mod k6;
#[path = "E2E-K7.rs"]
mod k7;
#[path = "launch_table.rs"]
mod launch_table;
#[path = "E2E-PEER.rs"]
mod peer;
