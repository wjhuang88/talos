//! Open-handle file-version evidence. Windows OS ABI boundary: ADR-096.
use crate::history_page::HistoryPageError;

#[cfg(unix)]
pub(crate) type FileStamp = std::fs::Metadata;
#[cfg(windows)]
#[derive(Clone)]
pub(crate) struct FileStamp {
    // All constructors receive the restrictive history_page::open_source handle.
    // Cloning a stamp must retain the lease, not merely copy sampled metadata.
    _lease: std::sync::Arc<std::fs::File>,
    volume: u64,
    identity: [u8; 16],
    size: u64,
    created: i64,
    written: i64,
    changed: i64,
}
#[cfg(windows)]
impl PartialEq for FileStamp {
    fn eq(&self, other: &Self) -> bool {
        self.volume == other.volume
            && self.identity == other.identity
            && self.size == other.size
            && self.created == other.created
            && self.written == other.written
            && self.changed == other.changed
    }
}
#[cfg(windows)]
impl Eq for FileStamp {}

#[cfg(not(any(unix, windows)))]
#[derive(Clone)]
pub(crate) struct FileStamp;

pub(crate) fn from_file(file: &std::fs::File) -> Result<FileStamp, HistoryPageError> {
    #[cfg(unix)]
    {
        Ok(file.metadata()?)
    }
    #[cfg(windows)]
    {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| windows_stamp(file)))
            .unwrap_or(Err(HistoryPageError::UnsupportedSource))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = file;
        Err(HistoryPageError::UnsupportedSource)
    }
}

#[cfg(windows)]
fn windows_stamp(file: &std::fs::File) -> Result<FileStamp, HistoryPageError> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_BASIC_INFO, FILE_ID_INFO, FileBasicInfo, FileIdInfo, GetFileInformationByHandleEx,
    };
    let mut identity = FILE_ID_INFO::default();
    let mut basic = FILE_BASIC_INFO::default();
    // SAFETY (ADR-096): borrowed live handle; matching information class and
    // repr(C) initialized output with exact size; pointers live through the call.
    let identity_ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileIdInfo,
            (&mut identity as *mut FILE_ID_INFO).cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    };
    if identity_ok == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    // SAFETY (ADR-096): same lifetime/size invariants, now FILE_BASIC_INFO class.
    let basic_ok = unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileBasicInfo,
            (&mut basic as *mut FILE_BASIC_INFO).cast(),
            std::mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    };
    if basic_ok == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(FileStamp {
        _lease: std::sync::Arc::new(file.try_clone()?),
        volume: identity.VolumeSerialNumber,
        identity: identity.FileId.Identifier,
        size: file.metadata()?.len(),
        created: basic.CreationTime,
        written: basic.LastWriteTime,
        changed: basic.ChangeTime,
    })
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn retained_stamp_excludes_writers_and_delete_until_last_clone_drops() {
        let dir = tempfile::tempdir().expect("temp directory");
        let path = dir.path().join("history.jsonl");
        std::fs::write(&path, b"original").expect("seed source");
        let file = crate::history_page::open_source(&path).expect("read lease");
        let stamp = from_file(&file).expect("stamp");
        let cloned = stamp.clone();
        drop(file);
        drop(stamp);
        assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err());
        assert!(std::fs::rename(&path, dir.path().join("moved")).is_err());
        let reader = crate::history_page::open_source(&path).expect("compatible reader");
        drop(reader);
        drop(cloned);
        std::fs::write(&path, b"appended after release").expect("released writer");
        std::fs::rename(&path, dir.path().join("moved")).expect("released rename");
    }

    #[test]
    fn existing_writer_prevents_recovery_lease() {
        let dir = tempfile::tempdir().expect("temp directory");
        let path = dir.path().join("history.jsonl");
        std::fs::write(&path, b"original").expect("seed source");
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .expect("writer");
        assert!(crate::history_page::open_source(&path).is_err());
        drop(writer);
        assert!(crate::history_page::open_source(&path).is_ok());
    }
}

/// Private source-version representation for host-owned scan checkpoints.
/// Equality is only version evidence; loading this value does not authorize reads.
#[cfg(unix)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredFileVersion {
    device: u64,
    inode: u64,
    size: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
}

#[cfg(unix)]
impl From<&FileStamp> for StoredFileVersion {
    fn from(stamp: &FileStamp) -> Self {
        use std::os::unix::fs::MetadataExt;
        Self {
            device: stamp.dev(),
            inode: stamp.ino(),
            size: stamp.len(),
            modified: stamp.mtime(),
            modified_ns: stamp.mtime_nsec(),
            changed: stamp.ctime(),
            changed_ns: stamp.ctime_nsec(),
        }
    }
}

#[cfg(all(test, unix))]
mod stored_version_tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_identity_and_rejects_equal_length_replacement() {
        let dir = tempfile::tempdir().expect("root");
        let path = dir.path().join("source");
        std::fs::write(&path, b"original").expect("source");
        let file = crate::history_page::open_source(&path).expect("open");
        let original = StoredFileVersion::from(&from_file(&file).expect("stamp"));
        let encoded = serde_json::to_vec(&original).expect("encode");
        let decoded: StoredFileVersion = serde_json::from_slice(&encoded).expect("decode");
        assert_eq!(original, decoded);
        // Reads may change access time, which is deliberately not version authority.
        assert_eq!(std::fs::read(&path).expect("read"), b"original");
        assert_eq!(
            decoded,
            StoredFileVersion::from(&from_file(&file).expect("read stamp"))
        );
        let replacement = dir.path().join("replacement");
        std::fs::write(&replacement, b"original").expect("replacement");
        std::fs::rename(&replacement, &path).expect("replace");
        let fresh = crate::history_page::open_source(&path).expect("new handle");
        assert_ne!(
            decoded,
            StoredFileVersion::from(&from_file(&fresh).expect("new stamp"))
        );
    }
}
