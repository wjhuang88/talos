//! Image file I/O boundary for provider adapters (ADR-050 / R5 + P1-B).
//!
//! When a `Message::Multimodal` part carries an image, the provider
//! adapter has to read the file bytes just before encoding them into
//! the wire format. The stored path was canonicalized at grant time
//! (see `talos_cli::image_validation::validate_image_path`), but the
//! filesystem can change between grant and read — a classic TOCTOU
//! surface for symlink-swap attacks and same-path replacement.
//!
//! This module enforces two complementary invariants:
//!
//! 1. **Path stability** (R5): the canonical path observed at read
//!    time must byte-for-byte match the canonical path stored on the
//!    `ContentPart::Image`. Any drift (symlink retarget, rename,
//!    replacement at a different inode) causes the read to be omitted.
//!
//! 2. **Content stability** (P1-B): the SHA-256 digest of the bytes
//!    read at request time must match the digest captured at grant
//!    time. If an attacker atomically replaces the file at the SAME
//!    canonical path, the path check alone passes, but the digest
//!    check catches the substitution and the part is omitted. The
//!    all-zero `ContentDigest::default()` sentinel means "verification
//!    intentionally skipped" and is only used by test fixtures.
//!
//! On any path drift, digest mismatch, fs error, or panic, the caller
//! MUST drop the corresponding content part from the provider request.

use std::path::Path;

use sha2::{Digest, Sha256};

use talos_core::message::ContentDigest;

/// Outcome of a TOCTOU-guarded image read.
#[derive(Debug)]
pub(crate) enum ImageRead {
    /// File contents were read and both the canonical path and the
    /// content digest matched the grant-time snapshot.
    Bytes(Vec<u8>),
    /// Canonical path drifted, content digest mismatched, or the read
    /// panicked/errored. The caller MUST omit this part from the
    /// provider request.
    Omit,
}

impl ImageRead {
    /// Returns the byte vector only when the read succeeded and both
    /// the canonical path and content digest matched. Returns `None`
    /// for [`ImageRead::Omit`].
    pub(crate) fn into_bytes(self) -> Option<Vec<u8>> {
        match self {
            ImageRead::Bytes(b) => Some(b),
            ImageRead::Omit => None,
        }
    }
}

/// Read image bytes for a path that was canonicalized at grant time.
///
/// Re-canonicalizes the path and compares it byte-for-byte against the
/// stored canonical path. Then reads the file and recomputes the
/// SHA-256 digest, comparing against the stored digest. On any mismatch,
/// fs error, or panic, returns [`ImageRead::Omit`] and emits a
/// `tracing::warn!`. The caller must drop the corresponding content
/// part from the provider request.
///
/// The `expected_digest` parameter carries the digest captured at grant
/// time. `ContentDigest::default()` (all-zero) is the test-fixture
/// sentinel: when supplied, digest verification is SKIPPED. Production
/// paths must always supply a non-default digest computed by
/// `talos_cli::image_validation::compute_content_digest`.
pub(crate) fn read_image_with_toctou_guard(
    stored_canonical: &Path,
    expected_digest: &ContentDigest,
) -> ImageRead {
    let re_canon = match std::panic::catch_unwind(|| stored_canonical.canonicalize()) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            tracing::warn!(
                path = %stored_canonical.display(),
                error = %e,
                "image_io: canonicalize failed at read time; dropping attachment"
            );
            return ImageRead::Omit;
        }
        Err(_) => {
            tracing::warn!(
                path = %stored_canonical.display(),
                "image_io: canonicalize panicked at read time; dropping attachment"
            );
            return ImageRead::Omit;
        }
    };

    if re_canon != stored_canonical {
        tracing::warn!(
            stored = %stored_canonical.display(),
            observed = %re_canon.display(),
            "image_io: TOCTOU mismatch — canonical path drifted between grant and read; dropping attachment"
        );
        return ImageRead::Omit;
    }

    let bytes = match std::panic::catch_unwind(|| std::fs::read(stored_canonical)) {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(e)) => {
            tracing::warn!(
                path = %stored_canonical.display(),
                error = %e,
                "image_io: fs::read failed; dropping attachment"
            );
            return ImageRead::Omit;
        }
        Err(_) => {
            tracing::warn!(
                path = %stored_canonical.display(),
                "image_io: fs::read panicked; dropping attachment"
            );
            return ImageRead::Omit;
        }
    };

    // P1-B: verify content digest. Skip only when caller supplied the
    // all-zero test sentinel.
    if !is_zero_sentinel(expected_digest) {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let observed: [u8; 32] = hasher.finalize().into();
        if observed != *expected_digest.as_bytes() {
            tracing::warn!(
                path = %stored_canonical.display(),
                expected = %expected_digest,
                "image_io: content digest mismatch — file replaced at same canonical path; dropping attachment"
            );
            return ImageRead::Omit;
        }
    }

    ImageRead::Bytes(bytes)
}

fn is_zero_sentinel(digest: &ContentDigest) -> bool {
    digest.as_bytes().iter().all(|b| *b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_real_png(path: &Path) {
        let img = image::RgbaImage::new(2, 2);
        img.save_with_format(path, image::ImageFormat::Png).unwrap();
    }

    fn digest_of(path: &Path) -> ContentDigest {
        let bytes = std::fs::read(path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let raw: [u8; 32] = hasher.finalize().into();
        ContentDigest::from_raw(raw)
    }

    #[test]
    fn matching_canonical_path_and_digest_returns_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stable.png");
        write_real_png(&path);
        let canonical = path.canonicalize().unwrap();
        let digest = digest_of(&canonical);
        match read_image_with_toctou_guard(&canonical, &digest) {
            ImageRead::Bytes(b) => assert!(!b.is_empty()),
            ImageRead::Omit => panic!("stable file with matching digest must produce bytes"),
        }
    }

    #[test]
    fn nonexistent_path_is_omitted() {
        let result = read_image_with_toctou_guard(
            Path::new("/nonexistent/path/img.png"),
            &ContentDigest::default(),
        );
        assert!(matches!(result, ImageRead::Omit));
    }

    #[test]
    fn symlink_swap_is_detected_and_omitted() {
        // The guard compares the stored path against the freshly
        // canonicalized path. When the stored path is non-canonical
        // (e.g. a symlink that has since been repointed), the two
        // diverge and the guard omits the read.
        let dir = tempfile::tempdir().unwrap();
        let file_a = dir.path().join("a.png");
        let file_b = dir.path().join("b.png");
        write_real_png(&file_a);
        write_real_png(&file_b);

        let link = dir.path().join("link.png");
        std::os::unix::fs::symlink(&file_a, &link).unwrap();
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&file_b, &link).unwrap();

        match read_image_with_toctou_guard(&link, &ContentDigest::default()) {
            ImageRead::Omit => {}
            ImageRead::Bytes(_) => panic!("swapped symlink must be omitted"),
        }
    }

    /// P1-B regression: atomic replacement at the SAME canonical path
    /// must be detected via digest mismatch. This is the specific
    /// attack that the R5 path-only guard missed. The stored path is
    /// fully canonical (matches what production ContentPart::Image
    /// carries), the file is replaced in place, and the guard must
    /// still omit the read.
    #[test]
    fn same_path_replacement_detected_via_digest_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("attack.png");
        write_real_png(&path);
        let canonical = path.canonicalize().unwrap();
        let original_digest = digest_of(&canonical);

        // Atomically replace the file at the same canonical path with
        // different content. A path-only check would still pass.
        let replacement = dir.path().join("replacement.png");
        let other = image::RgbaImage::new(8, 8);
        other
            .save_with_format(&replacement, image::ImageFormat::Png)
            .unwrap();
        std::fs::rename(&replacement, &canonical).unwrap();

        match read_image_with_toctou_guard(&canonical, &original_digest) {
            ImageRead::Omit => {}
            ImageRead::Bytes(_) => {
                panic!("same-path replacement must be detected via digest mismatch")
            }
        }
    }

    /// P1-B positive: after replacement, computing the digest of the
    /// NEW content and passing that as the expected digest must allow
    /// the read. This proves the guard is checking content, not
    /// blindly rejecting.
    #[test]
    fn same_path_with_updated_digest_passes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rotate.png");
        write_real_png(&path);
        let canonical = path.canonicalize().unwrap();

        let replacement = dir.path().join("r2.png");
        let other = image::RgbaImage::new(8, 8);
        other
            .save_with_format(&replacement, image::ImageFormat::Png)
            .unwrap();
        std::fs::rename(&replacement, &canonical).unwrap();

        let new_digest = digest_of(&canonical);
        match read_image_with_toctou_guard(&canonical, &new_digest) {
            ImageRead::Bytes(_) => {}
            ImageRead::Omit => panic!("matching digest after replacement must produce bytes"),
        }
    }

    /// All-zero sentinel skips digest verification. This is the
    /// test-fixture escape hatch documented on ContentDigest::default.
    #[test]
    fn zero_digest_sentinel_skips_verification() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sentinel.png");
        write_real_png(&path);
        let canonical = path.canonicalize().unwrap();
        match read_image_with_toctou_guard(&canonical, &ContentDigest::default()) {
            ImageRead::Bytes(_) => {}
            ImageRead::Omit => panic!("zero sentinel must skip digest verification"),
        }
    }
}
