//! Talos JSON-RPC server over stdio.
//!
//! This crate provides a JSON-RPC 2.0 server using newline-delimited JSON
//! (NDJSON) framing over stdio. The MVP executes requests sequentially; it does
//! not run method handlers concurrently.

pub mod cancel;
pub mod error;
pub mod framing;
pub mod methods;
pub mod protocol;
pub mod runtime;
pub mod server;

pub use runtime::{Runtime, RuntimeError};
pub use server::RpcServer;
