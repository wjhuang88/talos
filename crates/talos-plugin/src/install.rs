//! Offline, explicit Bundle installation with atomic staging.

use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{BundleManifest, CompatibleManifest, ManifestError, parse_compatible_manifest};

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("bundle manifest: {0}")]
    Manifest(#[from] ManifestError),
    #[error("filesystem: {0}")]
    Io(#[from] std::io::Error),
    #[error("bundle source must be a directory")]
    NotDirectory,
    #[error("legacy Plugin manifests require explicit migration before installation")]
    LegacyManifest,
    #[error("bundle artifact is missing: {0}")]
    MissingArtifact(String),
    #[error("bundle artifact digest mismatch")]
    DigestMismatch,
}

/// Validate and atomically install a manually supplied Bundle directory.
/// This function never loads, activates, registers, or grants permissions.
pub fn install_bundle(source: &Path, destination: &Path) -> Result<BundleManifest, InstallError> {
    if !source.is_dir() {
        return Err(InstallError::NotDirectory);
    }
    let source_abs = source.canonicalize()?;
    if destination.exists() && destination.canonicalize()? == source_abs {
        return Err(InstallError::NotDirectory);
    }
    let text = std::fs::read_to_string(source.join("manifest.toml"))?;
    let manifest = match parse_compatible_manifest(&text)? {
        CompatibleManifest::Bundle(bundle) => bundle,
        CompatibleManifest::Legacy(_) => return Err(InstallError::LegacyManifest),
    };
    let artifact = source.join(&manifest.bundle.artifact);
    if !artifact.is_file() {
        return Err(InstallError::MissingArtifact(manifest.bundle.artifact));
    }
    if let Some(expected) = manifest.bundle.digest.as_deref() {
        let bytes = std::fs::read(&artifact)?;
        let actual = format!("sha256:{:x}", Sha256::digest(bytes));
        if actual != expected {
            return Err(InstallError::DigestMismatch);
        }
    }
    let stage = destination.with_extension("staging");
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if stage.exists() {
        std::fs::remove_dir_all(&stage)?;
    }
    copy_tree(source, &stage)?;
    let backup = destination.with_extension("backup");
    if backup.exists() {
        std::fs::remove_dir_all(&backup)?;
    }
    if destination.exists() {
        std::fs::rename(destination, &backup)?;
    }
    std::fs::rename(&stage, destination)?;
    if backup.exists() {
        std::fs::remove_dir_all(backup)?;
    }
    Ok(manifest)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn digest_failure_preserves_existing_destination() {
        let root = std::env::temp_dir().join(format!("talos-i260-{}", std::process::id()));
        let source = root.join("source");
        let destination = root.join("installed");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("sentinel"), "old").unwrap();
        fs::write(source.join("bundle.wasm"), "actual").unwrap();
        fs::write(source.join("manifest.toml"), "schema_version=1\n[bundle]\nname=\"b\"\nversion=\"1.0.0\"\ncarrier=\"wasm\"\nartifact=\"bundle.wasm\"\ndigest=\"sha256:0000000000000000000000000000000000000000000000000000000000000000\"").unwrap();
        assert!(matches!(
            install_bundle(&source, &destination),
            Err(InstallError::DigestMismatch)
        ));
        assert_eq!(
            fs::read_to_string(destination.join("sentinel")).unwrap(),
            "old"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn valid_bundle_installs_and_repeats_deterministically() {
        let root = std::env::temp_dir().join(format!("talos-i260-ok-{}", std::process::id()));
        let source = root.join("source");
        let destination = root.join("installed");
        fs::create_dir_all(source.join("artifacts")).unwrap();
        fs::write(source.join("artifacts/main.wasm"), b"wasm").unwrap();
        fs::write(source.join("manifest.toml"), "schema_version=1\n[bundle]\nname=\"b\"\nversion=\"1.0.0\"\ncarrier=\"wasm\"\nartifact=\"artifacts/main.wasm\"").unwrap();
        let first = install_bundle(&source, &destination).unwrap();
        assert_eq!(first.bundle.name, "b");
        assert_eq!(
            fs::read(destination.join("artifacts/main.wasm")).unwrap(),
            b"wasm"
        );
        let second = install_bundle(&source, &destination).unwrap();
        assert_eq!(second.bundle.version, first.bundle.version);
        let _ = fs::remove_dir_all(root);
    }
}
