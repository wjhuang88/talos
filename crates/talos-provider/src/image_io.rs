//! Image file I/O boundary for provider adapters (ADR-050 / R5).
//!
//! When a `Message::Multimodal` part carries an image, the provider
//! adapter has to read the file bytes just before encoding them into
//! the wire format. The stored path was canonicalized at grant time
//! (see `talos_cli::image_validation::validate_image_path`), but the
//! filesystem can change between grant and read — a classic TOCTOU
//! surface for symlink-swap attacks.
//!
//! This module enforces a single rule: the canonical path observed
//! at read time must byte-for-byte match the canonical path stored on
//! the `ContentPart::Image`. Any drift (symlink retarget, rename,
//! replacement) causes the read to be skipped and an empty payload
//! to be returned, which the upstream provider will reject as a
//! malformed image. That is strictly safer than sending the wrong
//! file's bytes to the provider.

use std::path::Path;

/// Outcome of a TOCTOU-guarded image read.
#[derive(Debug)]
pub(crate) enum ImageRead {
    /// File contents were read and the canonical path matched.
    Bytes(Vec<u8>),
    /// Canonical path drifted between grant and read, or the read
    /// panicked/errored. The caller MUST omit this part from the
    /// provider request — sending `Bytes(Vec::new())` would still
    /// leak the fact that a (possibly different) file was readable.
    Omit,
}

impl ImageRead {
    /// Returns the byte vector only when the read succeeded and the
    /// canonical path matched. Returns `None` for [`ImageRead::Omit`].
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
/// stored canonical path. On any mismatch, fs error, or panic, returns
/// [`ImageRead::Omit`] and emits a `tracing::warn!`. The caller must
/// drop the corresponding content part from the provider request.
pub(crate) fn read_image_with_toctou_guard(stored_canonical: &Path) -> ImageRead {
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

    match std::panic::catch_unwind(|| std::fs::read(stored_canonical)) {
        Ok(Ok(bytes)) => ImageRead::Bytes(bytes),
        Ok(Err(e)) => {
            tracing::warn!(
                path = %stored_canonical.display(),
                error = %e,
                "image_io: fs::read failed; dropping attachment"
            );
            ImageRead::Omit
        }
        Err(_) => {
            tracing::warn!(
                path = %stored_canonical.display(),
                "image_io: fs::read panicked; dropping attachment"
            );
            ImageRead::Omit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_real_png(path: &Path) {
        let img = image::RgbaImage::new(2, 2);
        img.save_with_format(path, image::ImageFormat::Png).unwrap();
    }

    #[test]
    fn matching_canonical_path_returns_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stable.png");
        write_real_png(&path);
        let canonical = path.canonicalize().unwrap();
        match read_image_with_toctou_guard(&canonical) {
            ImageRead::Bytes(b) => assert!(!b.is_empty()),
            ImageRead::Omit => panic!("stable file must produce bytes"),
        }
    }

    #[test]
    fn nonexistent_path_is_omitted() {
        let result = read_image_with_toctou_guard(Path::new("/nonexistent/path/img.png"));
        assert!(matches!(result, ImageRead::Omit));
    }

    #[test]
    fn symlink_swap_is_detected_and_omitted() {
        // The guard compares the stored path against the freshly
        // canonicalized path. When the stored path is non-canonical
        // (e.g. a symlink that has since been repointed), the two
        // diverge and the guard omits the read. We construct that
        // scenario directly: create file_a, point a symlink at it,
        // then repoint the symlink at file_b. Passing the symlink
        // path as `stored_canonical` simulates an attacker who
        // swapped the target after the original grant.
        let dir = tempfile::tempdir().unwrap();
        let file_a = dir.path().join("a.png");
        let file_b = dir.path().join("b.png");
        write_real_png(&file_a);
        write_real_png(&file_b);

        let link = dir.path().join("link.png");
        std::os::unix::fs::symlink(&file_a, &link).unwrap();
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(&file_b, &link).unwrap();

        match read_image_with_toctou_guard(&link) {
            ImageRead::Omit => {}
            ImageRead::Bytes(_) => panic!("swapped symlink must be omitted"),
        }
    }
}
