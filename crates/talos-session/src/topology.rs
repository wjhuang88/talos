use sha2::{Digest, Sha256};

/// Compute a filesystem-safe directory name from a workspace root path using SHA-256.
pub(crate) fn workspace_dir_name(workspace_root: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(workspace_root.as_bytes());
    hex::encode(&hasher.finalize()[..8])
}

pub(crate) fn workspace_root_from_dir_name(dir_name: &str) -> String {
    if dir_name.len() == 16 && dir_name.chars().all(|c| c.is_ascii_hexdigit()) {
        dir_name.to_string()
    } else {
        String::new()
    }
}
