//! Protocol v1: types, framing, client and server halves.

pub mod client;
pub mod codec;
pub mod error;
pub mod peercred;
pub mod protocol;
pub mod server;

#[cfg(test)]
mod tests;
