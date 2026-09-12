//! Talos plugin — lifecycle hooks, built-in hook handlers, and plugin manifest parser.

pub mod builtin;
pub mod error;
pub mod event;
pub mod handler;
pub mod install;
#[cfg(feature = "wasm")]
pub mod lifecycle;
pub mod manifest;
pub mod registry;
#[cfg(feature = "wasm")]
pub mod wasm;

pub use builtin::LoggingHandler;
pub use error::HookError;
pub use event::{
    ALL_HOOK_EVENT_KINDS, BudgetKind, HookEvent, HookEventKind, ToolObservation, TurnEndReason,
    TurnId, TurnStatus,
};
pub use handler::{HookContext, HookHandler, HookResult};
pub use install::{InstallError, install_bundle};
pub use manifest::{
    BundleManifest, BundleMetadata, CompatibleManifest, LanguageProviderDeclaration, ManifestError,
    MigrationOptions, PluginHook, PluginManifest, PluginMetadata, PluginSkill, PluginTool,
    migrate_legacy_manifest, parse_compatible_manifest,
};
pub use registry::{HookOutcome, HookRegistration, HookRegistry};
#[cfg(feature = "wasm")]
pub use wasm::WasmLanguageProvider;
