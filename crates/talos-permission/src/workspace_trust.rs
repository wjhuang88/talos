//! Workspace trust store (ADR-038).
//!
//! Manages persistent trust decisions for Git repository roots.
//! Trust is granted by explicit user approval and stored in
//! `~/.talos/trusted_workspaces.toml`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Persistent store of trusted workspace paths.
///
/// Trust decisions are keyed by canonical path. The store reads from and writes
/// to `~/.talos/trusted_workspaces.toml`.
pub struct WorkspaceTrustStore {
    trusted: Mutex<HashSet<String>>,
    config_path: PathBuf,
}

impl WorkspaceTrustStore {
    pub fn new(talos_root: &Path) -> Self {
        let config_path = talos_root.join("trusted_workspaces.toml");
        let store = Self {
            trusted: Mutex::new(HashSet::new()),
            config_path,
        };
        let _ = store.load();
        store
    }

    fn load(&self) -> Result<(), std::io::Error> {
        if !self.config_path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&self.config_path)?;
        let mut trusted = self.trusted.lock().expect("trust lock poisoned");
        trusted.clear();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(path) = line.strip_prefix("workspace = ") {
                let path = path.trim_matches('"');
                if !path.is_empty() {
                    trusted.insert(path.to_string());
                }
            }
        }
        Ok(())
    }

    fn save(&self) -> Result<(), std::io::Error> {
        let trusted = self.trusted.lock().expect("trust lock poisoned");
        let mut content = String::from("# Trusted workspace paths (ADR-038).\n\n");
        for path in trusted.iter() {
            content.push_str(&format!("workspace = \"{path}\"\n"));
        }
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn is_trusted(&self, workspace_root: &Path) -> bool {
        let canonical = canonicalize_path(workspace_root);
        let trusted = self.trusted.lock().expect("trust lock poisoned");
        trusted.contains(&canonical)
    }

    pub fn grant_trust(&self, workspace_root: &Path) -> Result<(), std::io::Error> {
        let canonical = canonicalize_path(workspace_root);
        {
            let mut trusted = self.trusted.lock().expect("trust lock poisoned");
            trusted.insert(canonical);
        }
        self.save()
    }

    pub fn revoke_trust(&self, workspace_root: &Path) -> Result<(), std::io::Error> {
        let canonical = canonicalize_path(workspace_root);
        {
            let mut trusted = self.trusted.lock().expect("trust lock poisoned");
            trusted.remove(&canonical);
        }
        self.save()
    }

    pub fn trusted_count(&self) -> usize {
        self.trusted.lock().expect("trust lock poisoned").len()
    }
}

fn canonicalize_path(path: &Path) -> String {
    std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

/// Check if a Git repository exists at the given workspace root.
pub fn is_git_workspace(workspace_root: &Path) -> bool {
    workspace_root.join(".git").exists()
}

/// Check if a target path is within the repo root boundary.
/// Uses canonical path comparison to prevent symlink/`..` escapes.
pub fn is_within_repo(repo_root: &Path, target: &Path) -> bool {
    let repo_canonical = match std::fs::canonicalize(repo_root) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let target_canonical = match std::fs::canonicalize(target) {
        Ok(p) => p,
        Err(_) => {
            let parent = target.parent();
            match parent.and_then(|p| std::fs::canonicalize(p).ok()) {
                Some(p) => p.join(target.file_name().unwrap_or_default()),
                None => return false,
            }
        }
    };
    target_canonical.starts_with(&repo_canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    fn test_directory(label: &str) -> PathBuf {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("{label}-{}-{sequence}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("create isolated test directory");
        directory
    }

    #[test]
    fn trust_store_grant_and_check() {
        let dir = test_directory("trust_test_grant");

        let store = WorkspaceTrustStore::new(&dir);
        let ws = dir.join("my-project");
        std::fs::create_dir_all(&ws).unwrap();

        assert!(!store.is_trusted(&ws));
        store.grant_trust(&ws).unwrap();
        assert!(store.is_trusted(&ws));
        assert_eq!(store.trusted_count(), 1);

        store.revoke_trust(&ws).unwrap();
        assert!(!store.is_trusted(&ws));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn trust_store_persists_across_instances() {
        let dir = test_directory("trust_test_persist");

        let ws = dir.join("workspace-a");
        std::fs::create_dir_all(&ws).unwrap();

        {
            let store = WorkspaceTrustStore::new(&dir);
            store.grant_trust(&ws).unwrap();
        }

        {
            let store = WorkspaceTrustStore::new(&dir);
            assert!(
                store.is_trusted(&ws),
                "trust should persist across instances"
            );
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn is_git_workspace_detects_git_dir() {
        let dir = test_directory("trust_test_git");

        assert!(!is_git_workspace(&dir));

        std::fs::create_dir_all(dir.join(".git")).unwrap();
        assert!(is_git_workspace(&dir));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn is_within_repo_boundary_check() {
        let dir = test_directory("trust_test_boundary");
        std::fs::create_dir_all(dir.join("subdir")).unwrap();

        let inner = dir.join("subdir").join("file.txt");
        std::fs::write(&inner, "test").unwrap();

        assert!(is_within_repo(&dir, &inner));
        let outside = std::env::temp_dir().join("other-trust-test");
        assert!(!is_within_repo(&dir, &outside));

        std::fs::remove_dir_all(&dir).ok();
    }
}
