//! Permission-gated transcript export for the `/export` slash command.
//!
//! This is the "thin permission wrapper" referenced in I014. It evaluates
//! a `write` tool call against [`talos_permission::PermissionEngine`]
//! before touching the filesystem. If the engine returns anything other
//! than [`talos_permission::PermissionDecision::Allow`], the export is
//! refused with a user-facing reason — same surface as the inline
//! permission pipeline used by tools.

#![allow(dead_code)] // Preserved for the planned TUI slash-command rewire of `/export`.

use std::io::Write;
use std::path::Path;

use talos_permission::{PermissionDecision, PermissionEngine};

/// Outcome of an export attempt.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ExportError {
    /// The permission engine refused the write (Deny or Ask).
    PermissionDenied(String),
    /// The filesystem write failed.
    WriteFailed(String),
}

/// Writes `content` to `path` after checking write permission.
///
/// Permission check: a synthetic `write` tool call with `{"path": <path>}`
/// is evaluated against the supplied engine. `Allow` → proceed.
/// `Deny(reason)` / `Ask` → return [`ExportError::PermissionDenied`].
pub(crate) fn export_transcript(
    engine: &PermissionEngine,
    path: &Path,
    content: &str,
) -> Result<(), ExportError> {
    let input = serde_json::json!({ "path": path.to_string_lossy() });
    match engine.evaluate("write", &input) {
        PermissionDecision::Allow => {}
        PermissionDecision::Deny(reason) => {
            return Err(ExportError::PermissionDenied(reason));
        }
        PermissionDecision::Ask => {
            return Err(ExportError::PermissionDenied(
                "write requires interactive approval; use the agent's approval flow or run \
                 in a context where the write tool is allowed"
                    .to_string(),
            ));
        }
    }

    let mut file = std::fs::File::create(path)
        .map_err(|e| ExportError::WriteFailed(format!("create {}: {e}", path.display())))?;
    file.write_all(content.as_bytes())
        .map_err(|e| ExportError::WriteFailed(format!("write {}: {e}", path.display())))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
    use tempfile::tempdir;

    use super::*;

    fn temp_path(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(name);
        (dir, path)
    }

    fn engine_allowing_write() -> PermissionEngine {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new(
            "write",
            None,
            PermissionDecision::Allow,
        ));
        engine
    }

    fn engine_with_rule(rule: PermissionRule) -> PermissionEngine {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(rule);
        engine
    }

    #[test]
    fn export_writes_file_when_allowed() {
        let engine = engine_allowing_write();
        let (_dir, path) = temp_path("transcript.md");
        let content = "Hello, world!";

        export_transcript(&engine, &path, content).expect("allowed export");

        let read_back = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(read_back, content);
    }

    #[test]
    fn export_refuses_when_deny_rule_matches() {
        let engine = engine_with_rule(PermissionRule::new(
            "write",
            None,
            PermissionDecision::Deny("sensitive path".to_string()),
        ));
        let (_dir, path) = temp_path("transcript.md");

        let result = export_transcript(&engine, &path, "secret content");
        assert!(matches!(
            result,
            Err(ExportError::PermissionDenied(reason)) if reason == "sensitive path"
        ));
        assert!(!path.exists(), "file must not be created when denied");
    }

    #[test]
    fn export_refuses_ask_decision_without_creating_file() {
        // Empty engine has no write rule, so the default decision is Ask.
        let engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        let (_dir, path) = temp_path("transcript.md");

        let result = export_transcript(&engine, &path, "content");
        assert!(matches!(result, Err(ExportError::PermissionDenied(_))));
        assert!(!path.exists());
    }

    #[test]
    fn export_reports_filesystem_failure() {
        let engine = engine_allowing_write();
        // Path under a non-existent directory that we cannot create.
        let bad_path = PathBuf::from("/this/does/not/exist/transcript.md");

        let result = export_transcript(&engine, &bad_path, "content");
        assert!(matches!(result, Err(ExportError::WriteFailed(_))));
    }

    #[test]
    fn export_writes_unicode_content() {
        let engine = engine_allowing_write();
        let (_dir, path) = temp_path("unicode.md");
        let content = "中文内容 — Hello, world!";

        export_transcript(&engine, &path, content).expect("unicode export");

        let read_back = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(read_back, content);
    }
}
