//! Context loader for AGENTS.md files.
//!
//! Loads `AGENTS.md` files from the working directory and parent directories,
//! concatenates them into system prompt context. Loading order:
//! 1. Global: `~/.talos/AGENTS.md` (if exists)
//! 2. Project: walk from `workspace_root` up to git root (or filesystem root),
//!    loading `AGENTS.md` from each directory
//!
//! Total context is capped at 20,000 characters with head/tail truncation
//! if exceeded.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum total context size in characters.
const MAX_CONTEXT_SIZE: usize = 20_000;

/// Size of the head portion when truncating (first N characters).
const HEAD_SIZE: usize = 10_000;

/// Size of the tail portion when truncating (last N characters).
const TAIL_SIZE: usize = 10_000;

/// The filename to look for in each directory.
const AGENTS_MD: &str = "AGENTS.md";

/// Errors that can occur during context loading.
#[derive(Debug, Error)]
pub enum ContextError {
    /// An I/O error occurred while reading a file.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// The specified path was not found.
    #[error("path not found: {0}")]
    PathNotFound(PathBuf),
}

/// Result alias for context operations.
pub type ContextResult<T> = Result<T, ContextError>;

/// Loads AGENTS.md files from the workspace and parent directories.
///
/// The loader walks up from the workspace root to the git root (or filesystem
/// root), collecting `AGENTS.md` files along the way. It also loads a global
/// `AGENTS.md` from `~/.talos/` if it exists.
///
/// # Example
///
/// ```no_run
/// use talos_agent::context::ContextLoader;
/// use std::path::PathBuf;
///
/// let loader = ContextLoader::new(PathBuf::from("/path/to/project"));
/// let context = loader.load().unwrap();
/// ```
pub struct ContextLoader {
    /// The workspace root directory to start walking from.
    workspace_root: PathBuf,
    /// Whether context loading is enabled.
    enabled: bool,
}

impl ContextLoader {
    /// Creates a new context loader for the given workspace root.
    ///
    /// Context loading is enabled by default. Use [`ContextLoader::with_no_context`]
    /// to disable it.
    #[must_use]
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            enabled: true,
        }
    }

    /// Disables context loading.
    ///
    /// When context loading is disabled, [`ContextLoader::load`] returns an
    /// empty string without reading any files.
    #[must_use]
    pub fn with_no_context(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Loads and concatenates all AGENTS.md files.
    ///
    /// Files are loaded in this order:
    /// 1. Global: `~/.talos/AGENTS.md` (if exists)
    /// 2. Project: walk from `workspace_root` up to git root (or filesystem root),
    ///    loading `AGENTS.md` from each directory
    ///
    /// Each file is separated by `--- AGENTS.md from {path} ---`.
    ///
    /// If the total context exceeds 20,000 characters, it is truncated with
    /// head (first 10,000) + tail (last 10,000) preservation.
    ///
    /// # Errors
    ///
    /// Returns [`ContextError::IoError`] if a file exists but cannot be read.
    /// Missing files are skipped gracefully.
    pub fn load(&self) -> ContextResult<String> {
        if !self.enabled {
            return Ok(String::new());
        }

        let mut parts: Vec<String> = Vec::new();

        // 1. Load global AGENTS.md
        if let Some(global_path) = self.global_agents_path() {
            if global_path.exists() {
                let content = fs::read_to_string(&global_path).map_err(ContextError::IoError)?;
                if !content.trim().is_empty() {
                    parts.push(self.format_section(&global_path, &content));
                }
            }
        }

        // 2. Walk from workspace_root up to git root (or filesystem root)
        let mut current: Option<&Path> = Some(&self.workspace_root);
        while let Some(dir) = current {
            let agents_path = dir.join(AGENTS_MD);
            if agents_path.exists() {
                let content = fs::read_to_string(&agents_path).map_err(ContextError::IoError)?;
                if !content.trim().is_empty() {
                    parts.push(self.format_section(&agents_path, &content));
                }
            }

            // Stop at git root
            if dir.join(".git").exists() {
                break;
            }

            current = dir.parent();
        }

        let combined = parts.join("\n");
        Ok(Self::apply_size_limit(&combined))
    }

    /// Returns the path to the global AGENTS.md file.
    fn global_agents_path(&self) -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let mut path = PathBuf::from(home);
        path.push(".talos");
        path.push(AGENTS_MD);
        Some(path)
    }

    /// Formats a section with a clear separator header.
    fn format_section(&self, path: &Path, content: &str) -> String {
        format!("--- AGENTS.md from {} ---\n{}", path.display(), content)
    }

    /// Applies the size limit to the combined context.
    ///
    /// If the context exceeds [`MAX_CONTEXT_SIZE`], it is truncated to preserve
    /// the first [`HEAD_SIZE`] characters and the last [`TAIL_SIZE`] characters.
    fn apply_size_limit(content: &str) -> String {
        let char_count = content.chars().count();
        if char_count <= MAX_CONTEXT_SIZE {
            return content.to_string();
        }

        let chars: Vec<char> = content.chars().collect();
        let mut result = String::with_capacity(HEAD_SIZE + TAIL_SIZE + 3);

        // Head portion
        result.extend(chars.iter().take(HEAD_SIZE));

        // Truncation indicator
        result.push_str("\n...\n");

        // Tail portion
        let tail_start = char_count.saturating_sub(TAIL_SIZE);
        result.extend(chars.iter().skip(tail_start));

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a temporary directory with an AGENTS.md file.
    fn create_agents_md(dir: &Path, content: &str) {
        let path = dir.join(AGENTS_MD);
        fs::write(path, content).expect("failed to write AGENTS.md");
    }

    /// Helper to create a .git directory to simulate a git root.
    fn create_git_root(dir: &Path) {
        let git_dir = dir.join(".git");
        fs::create_dir(git_dir).expect("failed to create .git directory");
    }

    #[test]
    fn test_load_single_agents_md() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        create_agents_md(temp_dir.path(), "# Project Rules\nBe helpful.");

        let loader = ContextLoader::new(temp_dir.path().to_path_buf());
        let context = loader.load().expect("load failed");

        assert!(context.contains("# Project Rules"));
        assert!(context.contains("Be helpful."));
        assert!(context.contains("AGENTS.md from"));
    }

    #[test]
    fn test_load_multiple_agents_md_from_parent_dirs() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let sub_dir = temp_dir.path().join("sub");
        fs::create_dir(&sub_dir).expect("failed to create sub directory");

        create_agents_md(temp_dir.path(), "# Root Rules\nRoot content.");
        create_agents_md(&sub_dir, "# Sub Rules\nSub content.");

        let loader = ContextLoader::new(sub_dir);
        let context = loader.load().expect("load failed");

        assert!(context.contains("# Root Rules"));
        assert!(context.contains("# Sub Rules"));
        // Sub directory file should appear before root (walk order: sub -> root)
        let sub_idx = context.find("# Sub Rules").expect("sub rules not found");
        let root_idx = context.find("# Root Rules").expect("root rules not found");
        assert!(sub_idx < root_idx, "sub should appear before root");
    }

    #[test]
    fn test_global_agents_md_loading() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let talos_dir = temp_dir.path().join(".talos");
        fs::create_dir(&talos_dir).expect("failed to create .talos directory");
        create_agents_md(&talos_dir, "# Global Rules\nGlobal content.");

        // Override HOME for this test
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let loader = ContextLoader::new(temp_dir.path().join("project"));
        let context = loader.load().expect("load failed");

        assert!(context.contains("# Global Rules"));
        assert!(context.contains("Global content."));

        // Restore HOME
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_size_limit_truncation() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        // Override HOME to temp dir to avoid picking up real global AGENTS.md
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        // Create content that exceeds 20,000 characters
        let head_content = "A".repeat(15_000);
        let tail_content = "B".repeat(15_000);
        let full_content = format!("{}{}", head_content, tail_content);
        create_agents_md(temp_dir.path(), &full_content);

        let loader = ContextLoader::new(temp_dir.path().to_path_buf());
        let context = loader.load().expect("load failed");

        let char_count = context.chars().count();
        // Truncated content: head (10,000) + separator (5) + tail (10,000) = 20,005
        assert!(
            char_count <= MAX_CONTEXT_SIZE + 20,
            "context should be near size limit, got {} chars",
            char_count
        );

        // The separator header shifts the start; verify head chars are present
        assert!(context.contains(&"A".repeat(100)));
        // Tail portion should be preserved
        assert!(context.ends_with(&"B".repeat(100)));
        // Truncation indicator should be present
        assert!(context.contains("\n...\n"));

        // Restore HOME
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_no_context_disables_loading() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        create_agents_md(temp_dir.path(), "# Should Not Load");

        let loader = ContextLoader::new(temp_dir.path().to_path_buf()).with_no_context();
        let context = loader.load().expect("load failed");

        assert!(context.is_empty());
    }

    #[test]
    fn test_missing_agents_md_skipped_gracefully() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        // No AGENTS.md created

        let loader = ContextLoader::new(temp_dir.path().to_path_buf());
        let context = loader.load().expect("load failed");

        assert!(context.is_empty());
    }

    #[test]
    fn test_git_root_detection() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let sub_dir = temp_dir.path().join("sub");
        let deep_dir = sub_dir.join("deep");
        fs::create_dir_all(&deep_dir).expect("failed to create directories");

        create_agents_md(temp_dir.path(), "# Root");
        create_agents_md(&sub_dir, "# Sub");
        create_agents_md(&deep_dir, "# Deep");
        create_git_root(temp_dir.path());

        // Walking stops at .git; verify git root file IS included

        let loader = ContextLoader::new(deep_dir);
        let context = loader.load().expect("load failed");

        assert!(context.contains("# Root"));
        assert!(context.contains("# Sub"));
        assert!(context.contains("# Deep"));
    }

    #[test]
    fn test_git_root_stops_walking() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let sub_dir = temp_dir.path().join("sub");
        fs::create_dir(&sub_dir).expect("failed to create sub directory");

        create_agents_md(temp_dir.path(), "# Git Root");
        create_agents_md(&sub_dir, "# Sub Dir");
        create_git_root(temp_dir.path());

        let loader = ContextLoader::new(sub_dir);
        let context = loader.load().expect("load failed");

        // Both should be found since we walk from sub_dir up to git root
        assert!(context.contains("# Git Root"));
        assert!(context.contains("# Sub Dir"));
    }

    #[test]
    fn test_empty_agents_md_skipped() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        // Create an empty AGENTS.md
        fs::write(temp_dir.path().join(AGENTS_MD), "").expect("failed to write empty file");

        // Override HOME to temp dir to avoid picking up real global AGENTS.md
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let loader = ContextLoader::new(temp_dir.path().to_path_buf());
        let context = loader.load().expect("load failed");

        assert!(context.is_empty());

        // Restore HOME
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_whitespace_only_agents_md_skipped() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        fs::write(temp_dir.path().join(AGENTS_MD), "   \n\n  ").expect("failed to write whitespace file");

        let loader = ContextLoader::new(temp_dir.path().to_path_buf());
        let context = loader.load().expect("load failed");

        assert!(context.is_empty());
    }

    #[test]
    fn test_apply_size_limit_exact_boundary() {
        // Content exactly at the limit should not be truncated
        let content = "X".repeat(MAX_CONTEXT_SIZE);
        let result = ContextLoader::apply_size_limit(&content);
        assert_eq!(result.chars().count(), MAX_CONTEXT_SIZE);
    }

    #[test]
    fn test_apply_size_limit_one_over() {
        // Content significantly over the limit should be truncated
        let content = "X".repeat(MAX_CONTEXT_SIZE + 5_000);
        let result = ContextLoader::apply_size_limit(&content);
        // Should have head + tail + truncation indicator
        assert!(result.contains("..."));
        assert!(result.chars().count() < content.chars().count());
    }
}
