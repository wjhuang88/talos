//! Explicit, bounded on-demand Bundle resolution.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{
    BundleManifest, CompatibleManifest, InstallError, install_bundle_with_guard,
    parse_compatible_manifest,
};

/// User consent required before resolving an optional Bundle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionConsent {
    /// The user explicitly approved this exact request.
    Granted,
    /// No resolution is allowed.
    Denied,
}

/// Stable identity of the capability being resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionRequest {
    /// Capability identity requested by the consumer.
    pub capability: String,
    /// Exact Bundle name expected from the source.
    pub bundle_name: String,
    /// Exact Bundle version expected from the source.
    pub bundle_version: String,
}

/// Bounded controls applied before any Bundle files are copied.
#[derive(Clone, Copy, Debug)]
pub struct ResolutionLimits {
    /// Maximum time allowed for this resolution operation.
    pub timeout: Duration,
}

impl Default for ResolutionLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }
}

/// Explicit resolution outcome. Installation does not activate executable content.
#[derive(Debug)]
pub struct ResolutionResult {
    /// Validated manifest installed into the inactive destination.
    pub manifest: BundleManifest,
    /// Destination that remains inactive until normal lifecycle admission.
    pub destination: PathBuf,
}

/// Errors are deliberately fail-closed and retain the reason for audit output.
#[derive(Debug, thiserror::Error)]
pub enum ResolutionError {
    /// Consent was not granted for the exact request.
    #[error("explicit consent is required")]
    ConsentDenied,
    /// Request identity did not match the verified manifest.
    #[error("resolved Bundle identity does not match the request")]
    IdentityMismatch,
    /// The operation exceeded its bound or was cancelled.
    #[error("resolution cancelled or timed out")]
    Cancelled,
    /// Bundle verification or installation failed.
    #[error("Bundle installation failed: {0}")]
    Install(#[from] InstallError),
}

/// Resolve a manually supplied/offline Bundle only after exact user consent.
///
/// This function intentionally accepts a local source path: network acquisition is an outer
/// policy owned by a future connector and cannot be implied by this resolver. The Bundle is
/// validated and copied, but never activated or registered here.
pub fn resolve_verified_bundle(
    request: &ResolutionRequest,
    source: &Path,
    destination: &Path,
    consent: ResolutionConsent,
    limits: ResolutionLimits,
    cancelled: impl Fn() -> bool,
) -> Result<ResolutionResult, ResolutionError> {
    if consent != ResolutionConsent::Granted || cancelled() {
        return Err(if consent == ResolutionConsent::Granted {
            ResolutionError::Cancelled
        } else {
            ResolutionError::ConsentDenied
        });
    }
    let started = Instant::now();
    let manifest_text =
        std::fs::read_to_string(source.join("manifest.toml")).map_err(InstallError::Io)?;
    let expected =
        match parse_compatible_manifest(&manifest_text).map_err(InstallError::Manifest)? {
            CompatibleManifest::Bundle(bundle) => bundle,
            CompatibleManifest::Legacy(_) => {
                return Err(ResolutionError::Install(InstallError::LegacyManifest));
            }
        };
    let provider_matches = expected
        .language_provider
        .as_ref()
        .map(|provider| format!("language.{}", provider.language) == request.capability)
        .unwrap_or(false);
    if expected.bundle.name != request.bundle_name
        || expected.bundle.version != request.bundle_version
        || !provider_matches
    {
        return Err(ResolutionError::IdentityMismatch);
    }
    let staging = destination.with_extension("resolution-staging");
    let manifest = install_bundle_with_guard(source, &staging, || {
        cancelled() || started.elapsed() > limits.timeout
    })
    .map_err(|error| match error {
        InstallError::Io(io) if io.kind() == std::io::ErrorKind::Interrupted => {
            ResolutionError::Cancelled
        }
        InstallError::Cancelled => ResolutionError::Cancelled,
        other => ResolutionError::Install(other),
    });
    let manifest = match manifest {
        Ok(manifest) => manifest,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    if started.elapsed() > limits.timeout || cancelled() {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(ResolutionError::Cancelled);
    }
    debug_assert_eq!(manifest.bundle.name, request.bundle_name);
    let backup = destination.with_extension("resolution-backup");
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(InstallError::Io)?;
    }
    if destination.exists() {
        std::fs::rename(destination, &backup).map_err(InstallError::Io)?;
    }
    if let Err(error) = std::fs::rename(&staging, destination) {
        if backup.exists() && !destination.exists() {
            let _ = std::fs::rename(&backup, destination);
        }
        let _ = std::fs::remove_dir_all(&staging);
        return Err(ResolutionError::Install(InstallError::Io(error)));
    }
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(InstallError::Io)?;
    }
    Ok(ResolutionResult {
        manifest,
        destination: destination.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(root: &Path) -> PathBuf {
        let source = root.join("source");
        fs::create_dir_all(&source).expect("test fixture operation");
        fs::write(source.join("provider.wat"), "(module)").expect("test fixture operation");
        fs::write(
            source.join("manifest.toml"),
            "schema_version=1\n[bundle]\nname=\"demo\"\nversion=\"1.0.0\"\ncarrier=\"wasm\"\nartifact=\"provider.wat\"\n[language_provider]\nlanguage=\"python\"\nartifact=\"provider.wat\"",
        )
        .expect("test fixture operation");
        source
    }

    #[test]
    fn denied_consent_does_not_write() {
        let root = std::env::temp_dir().join(format!("talos-resolution-{}", std::process::id()));
        let source = fixture(&root);
        let destination = root.join("installed");
        let request = ResolutionRequest {
            capability: "language.python".into(),
            bundle_name: "demo".into(),
            bundle_version: "1.0.0".into(),
        };
        assert!(matches!(
            resolve_verified_bundle(
                &request,
                &source,
                &destination,
                ResolutionConsent::Denied,
                ResolutionLimits::default(),
                || false
            ),
            Err(ResolutionError::ConsentDenied)
        ));
        assert!(!destination.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn identity_mismatch_rolls_back_inactive_install() {
        let root = std::env::temp_dir().join(format!("talos-resolution-id-{}", std::process::id()));
        let source = fixture(&root);
        let destination = root.join("installed");
        let request = ResolutionRequest {
            capability: "language.python".into(),
            bundle_name: "other".into(),
            bundle_version: "1.0.0".into(),
        };
        assert!(matches!(
            resolve_verified_bundle(
                &request,
                &source,
                &destination,
                ResolutionConsent::Granted,
                ResolutionLimits::default(),
                || false
            ),
            Err(ResolutionError::IdentityMismatch)
        ));
        assert!(!destination.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cancelled_resolution_preserves_existing_install() {
        let root =
            std::env::temp_dir().join(format!("talos-resolution-cancel-{}", std::process::id()));
        let source = fixture(&root);
        let destination = root.join("installed");
        fs::create_dir_all(&destination).expect("test fixture operation");
        fs::write(destination.join("sentinel"), "old").expect("test fixture operation");
        let request = ResolutionRequest {
            capability: "language.python".into(),
            bundle_name: "demo".into(),
            bundle_version: "1.0.0".into(),
        };
        assert!(matches!(
            resolve_verified_bundle(
                &request,
                &source,
                &destination,
                ResolutionConsent::Granted,
                ResolutionLimits::default(),
                || true
            ),
            Err(ResolutionError::Cancelled)
        ));
        assert_eq!(
            fs::read_to_string(destination.join("sentinel")).expect("test fixture operation"),
            "old"
        );
        let _ = fs::remove_dir_all(root);
    }
}
