//! Preserve std::process current-directory spelling at the native launch boundary.
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Component, Path, PathBuf};

pub(super) fn encode(path: &Path) -> Result<Vec<u16>, String> {
    let original: Vec<u16> = path.as_os_str().encode_wide().collect();
    let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
    let unc: Vec<u16> = r"\\?\UNC\".encode_utf16().collect();
    let candidate = if original.starts_with(&unc) {
        Some([vec![92, 92], original[unc.len()..].to_vec()].concat())
    } else if original.starts_with(&prefix) {
        Some(original[prefix.len()..].to_vec())
    } else {
        None
    };
    let mut selected = original;
    if let Some(candidate) = candidate {
        let candidate_path = PathBuf::from(OsString::from_wide(&candidate));
        // Like Rust's make_dirp, remove the verbatim prefix only when Win32's
        // full-path normalization leaves the remaining spelling byte-for-byte
        // unchanged. Compare OsStr, not Path components (which normalize dots).
        let absolute = std::panic::catch_unwind(|| std::path::absolute(&candidate_path))
            .map_err(|_| "Windows current-directory normalization panicked".to_owned())?;
        let still_verbatim = matches!(candidate_path.components().next(),
            Some(Component::Prefix(prefix)) if prefix.kind().is_verbatim());
        if !still_verbatim
            && candidate_path.is_absolute()
            && absolute.is_ok_and(|absolute| absolute.as_os_str() == candidate_path.as_os_str())
        {
            selected = candidate;
        }
    }
    if selected.contains(&0) {
        return Err("Windows current directory contains NUL".into());
    }
    selected.push(0);
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_drive_directory_retains_target_without_verbatim_prefix() {
        let root = tempfile::tempdir().expect("workspace");
        let canonical = root.path().canonicalize().expect("canonical workspace");
        let encoded = encode(&canonical).expect("encoded cwd");
        let cwd = PathBuf::from(OsString::from_wide(&encoded[..encoded.len() - 1]));
        assert!(!encoded.starts_with(&r"\\?\".encode_utf16().collect::<Vec<_>>()));
        assert_eq!(cwd.canonicalize().expect("same target"), canonical);
    }

    #[test]
    fn normalization_sensitive_verbatim_directory_is_preserved() {
        for path in [
            r"\\?\C:\workspace\..\other",
            r"\\?\C:\workspace\trailing.",
            r"\\?\\\?\C:\workspace",
        ] {
            let mut expected: Vec<u16> = path.encode_utf16().collect();
            expected.push(0);
            assert_eq!(encode(Path::new(path)).expect("encode"), expected);
        }
    }
}
