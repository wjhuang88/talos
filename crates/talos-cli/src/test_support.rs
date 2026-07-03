//! Shared test-only synchronization for `HOME`-mutating tests.
//!
//! `HOME` is process-wide global state (`std::env::set_var`/`remove_var`
//! affect the whole process, not just the calling thread). `cargo test`
//! executes tests in parallel threads within the same process, so any two
//! tests that redirect `HOME` to an isolated temp directory (e.g. around
//! `Config::save()`/`Config::load()`) must serialize through the *same*
//! mutex instance. A private per-module mutex only prevents races among
//! tests within that one module — it does nothing against a different
//! module's tests that also mutate `HOME` under their own private lock.
//! All `HOME`-mutating tests in this crate must lock this shared mutex.

#[cfg(test)]
pub(crate) static HOME_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
