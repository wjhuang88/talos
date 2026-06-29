//! Talos JSON-RPC server over stdio.
//!
//! This crate provides a JSON-RPC 2.0 server using newline-delimited JSON
//! (NDJSON) framing over stdio. The MVP executes requests sequentially; it does
//! not run method handlers concurrently.
//!
//! The pre-1.0 support boundary is local transport only:
//!
//! - stdio NDJSON framing and protocol types are reusable;
//! - request handlers run behind the Talos runtime abstraction supplied by the embedding process;
//! - this crate does not provide TCP, HTTP, authentication, authorization, or remote session
//!   exposure;
//! - cancellation is scoped to in-process request state;
//! - remote/control-plane semantics require a separate ADR before becoming stable.
//!
//! Treat this crate as a local integration protocol foundation, not as a public remote server.

pub mod cancel;
pub mod error;
pub mod framing;
pub mod methods;
pub mod protocol;
pub mod runtime;
pub mod server;

pub use runtime::{Runtime, RuntimeError};
pub use server::RpcServer;
