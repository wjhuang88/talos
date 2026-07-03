//! Talos model catalog — SQLite-backed model/provider catalog store and resolver.
//!
//! Provides a durable catalog store backed by bundled SQLite (ADR-008) with
//! explicit schema versioning and migration. The store holds provider
//! metadata, model metadata, and pricing data sourced from the built-in TOML
//! dataset and/or models.dev imports.
//!
//! ## Crate Boundary
//!
//! `talos-models` depends on `talos-core` for shared catalog types
//! (`ModelMetadata`, `ProviderInfo`, etc.). It does **not** depend on
//! `talos-config`; CLI/TUI/runtime callers construct a [`ModelCatalog`] and
//! pass catalog data as plain slices to `talos-config` resolver methods.
//! This keeps `talos-config` free of any implicit SQLite dependency.

pub mod error;
pub mod import;
pub mod store;

pub use error::CatalogError;
pub use import::{ImportResult, import_models_dev_api, import_models_dev_models};
pub use store::ModelCatalog;

pub use talos_core::model::{
    ModelCapabilities, ModelMetadata, ModelPricing, ModelSource, ProviderInfo, ProviderSource,
};
