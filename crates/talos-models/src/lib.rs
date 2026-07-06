//! Talos model catalog — SQLite-backed model/provider catalog store and resolver.
//!
//! ## Status — Quarantined (non-runtime, 2026-07-06)
//!
//! This crate is **quarantined** as non-runtime historical/foundation code. No
//! production crate in the workspace (`talos-cli`, `talos-tui`, `talos-config`,
//! `talos-agent`, `talos-runtime`, …) depends on `talos-models`, and no CLI/TUI
//! runtime path constructs [`ModelCatalog`] or opens/creates `~/.talos/catalog.db`.
//! Runtime model/provider metadata is sourced exclusively from the packaged
//! offline `crates/talos-config/src/models.toml` (and build-time
//! `BUILD_MODELS=1` refresh), plus user config overlays. See
//! `docs/backlog/active/MC-002-remove-runtime-catalog-db-residuals.md` for the
//! cleanup record and `docs/backlog/active/MC-001-model-catalog-modernization.md`
//! for the 2026-07-05 maintainer decision that superseded the runtime DB path.
//!
//! The crate is intentionally kept (not deleted) to avoid a semver-breaking
//! removal while its parsers were the historical basis for `talos-config`'s
//! build-time refresh. It must not be wired into any CLI/TUI/runtime path, and
//! no future change may reintroduce `~/.talos/catalog.db` creation, import, or
//! seeding. A regression guard test at
//! `crates/talos-cli/tests/no_catalog_db_guard.rs` enforces this invariant from
//! the binary side.
//!
//! ## Historical Description
//!
//! Historically `talos-models` provided a durable catalog store backed by
//! bundled SQLite (ADR-008) with explicit schema versioning and migration,
//! holding provider metadata, model metadata, and pricing data sourced from
//! the built-in TOML dataset and/or models.dev imports.
//!
//! ## Crate Boundary
//!
//! `talos-models` depends on `talos-core` for shared catalog types
//! (`ModelMetadata`, `ProviderInfo`, etc.). It does **not** depend on
//! `talos-config`. (Historically CLI/TUI callers were expected to construct a
//! [`ModelCatalog`] and pass catalog data as plain slices to `talos-config`
//! resolver methods; that wiring was superseded by the 2026-07-05 maintainer
//! decision and is no longer present in any production crate.)

pub mod error;
pub mod import;
pub mod store;

pub use error::CatalogError;
pub use import::{ImportResult, import_models_dev_api, import_models_dev_models};
pub use store::ModelCatalog;

pub use talos_core::model::{
    ModelCapabilities, ModelMetadata, ModelPricing, ModelSource, ProviderInfo, ProviderSource,
};
