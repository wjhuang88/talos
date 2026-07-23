//! `read_image` tool — model-invoked image read (ADR-051 / I154).
//!
//! Only presented to models with `ImageInputCapability::Supported`.
//! The model explicitly calls `read_image({ "path": "..." })`; the
//! tool validates the image through the shared ingestion module and
//! returns a safe summary plus a one-shot provider continuation
//! artifact. The artifact appears in exactly the next provider request.

use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::message::ContentPart;
use talos_core::tool::{
    AgentTool, ToolExecutionAuthorization, ToolExecutionOutput, ToolFamily, ToolNature,
    ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;

use crate::file_tools::{FileSnapshotRegistry, FileToolError, resolve_authorized_path};
use crate::image_validation;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadImageInput {
    pub path: String,
}

pub struct ReadImageTool {
    workspace_root: PathBuf,
    #[allow(dead_code)]
    snapshots: Option<FileSnapshotRegistry>,
}

impl ReadImageTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            snapshots: None,
        }
    }

    pub fn with_snapshot_registry(
        workspace_root: PathBuf,
        snapshots: FileSnapshotRegistry,
    ) -> Self {
        Self {
            workspace_root,
            snapshots: Some(snapshots),
        }
    }
}

#[async_trait]
impl AgentTool for ReadImageTool {
    fn name(&self) -> &str {
        "read_image"
    }

    fn description(&self) -> &str {
        "Read a local image file (PNG/JPEG/GIF/WebP) and attach it to your next response. \
         Requires exact path permission. Only available for vision-capable models."
    }

    fn parameters(&self) -> Value {
        tool_parameters!(ReadImageInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let _parsed: ReadImageInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {e}")),
        };
        ToolResult::error(
            "read_image requires authorized execution; the path was not read. \
             Use the tool through a permission-aware execution path."
                .to_string(),
        )
    }

    async fn execute_authorized_with_output(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> ToolExecutionOutput {
        let parsed: ReadImageInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => {
                return ToolExecutionOutput::error(format!("Invalid input: {e}"));
            }
        };

        let resolved = match resolve_authorized_path(
            &self.workspace_root,
            &parsed.path,
            "read_image",
            ToolNature::Read,
            authorizations,
        ) {
            Ok(p) => p,
            Err(FileToolError::PathEscape(_)) => {
                return ToolExecutionOutput::error(
                    "Permission denied: path is outside the workspace and no matching authorization was provided.".to_string(),
                );
            }
            Err(e) => {
                return ToolExecutionOutput::error(format!("Path resolution failed: {e}"));
            }
        };

        match image_validation::create_image_content_part(&resolved, 0, 0) {
            Ok(content_part) => {
                let summary = match &content_part {
                    ContentPart::Image {
                        path,
                        mime,
                        byte_count,
                        ..
                    } => {
                        let basename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("(unknown)");
                        format!(
                            "[Image read: `{basename}` ({byte_count} bytes, {mime}); attached to next provider request]"
                        )
                    }
                    _ => String::new(),
                };
                ToolExecutionOutput {
                    result: ToolResult::success(summary),
                    next_provider_parts: vec![content_part],
                }
            }
            Err(e) => ToolExecutionOutput::error(format!("Image validation failed: {e}")),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::File
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        match input.get("path").and_then(Value::as_str) {
            Some(path) => vec![
                ToolPermissionFacet::with_resource(ToolNature::Read, path, ToolResourceKind::Path)
                    .with_description("image read"),
            ],
            None => vec![ToolPermissionFacet::new(ToolNature::Read)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{ToolAuthorizationScope, ToolExecutionAuthorization};

    fn workspace_root() -> PathBuf {
        PathBuf::from(".")
    }

    #[tokio::test]
    async fn execute_returns_error_without_authorization() {
        let tool = ReadImageTool::new(workspace_root());
        let result = tool.execute(serde_json::json!({"path": "test.png"})).await;
        assert!(result.is_error);
        assert!(result.content.contains("requires authorized execution"));
        // Path must NOT appear in the error message
        assert!(!result.content.contains("test.png"));
    }

    #[tokio::test]
    async fn execute_authorized_returns_image_part_for_valid_png() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.png");
        std::fs::write(&img_path, MINIMAL_PNG).unwrap();

        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let auth = vec![
            ToolExecutionAuthorization::for_path(
                "read_image",
                ToolNature::Read,
                dir.path(),
                "test.png",
                ToolAuthorizationScope::Once,
            )
            .unwrap(),
        ];
        let output = tool
            .execute_authorized_with_output(
                serde_json::json!({"path": img_path.to_string_lossy()}),
                &auth,
            )
            .await;

        assert!(!output.result.is_error, "{}", output.result.content);
        assert_eq!(output.next_provider_parts.len(), 1);
        match &output.next_provider_parts[0] {
            ContentPart::Image {
                mime, byte_count, ..
            } => {
                assert_eq!(*mime, "image/png");
                assert_eq!(*byte_count, MINIMAL_PNG.len() as u64);
            }
            _ => panic!("expected Image content part"),
        }
        assert!(output.result.content.contains("test.png"));
        assert!(output.result.content.contains("image/png"));
    }

    #[tokio::test]
    async fn execute_authorized_rejects_path_escape() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let output = tool
            .execute_authorized_with_output(serde_json::json!({"path": "/etc/passwd"}), &[])
            .await;

        assert!(output.result.is_error);
        assert!(output.result.content.contains("Permission denied"));
        // Raw path must NOT appear in the error message
        assert!(!output.result.content.contains("/etc/passwd"));
        assert!(output.next_provider_parts.is_empty());
    }

    #[tokio::test]
    async fn execute_authorized_rejects_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let output = tool
            .execute_authorized_with_output(serde_json::json!({"path": "no_such_file.png"}), &[])
            .await;

        assert!(output.result.is_error);
        assert!(output.next_provider_parts.is_empty());
    }

    #[tokio::test]
    async fn execute_authorized_rejects_directory() {
        let dir = tempfile::tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let output = tool
            .execute_authorized_with_output(serde_json::json!({"path": "subdir"}), &[])
            .await;

        assert!(output.result.is_error);
        assert!(output.next_provider_parts.is_empty());
    }

    #[test]
    fn tool_metadata() {
        let tool = ReadImageTool::new(workspace_root());
        assert_eq!(tool.name(), "read_image");
        assert!(tool.is_read_only());
        assert_eq!(tool.nature(), ToolNature::Read);
        assert_eq!(tool.family(), ToolFamily::File);
        assert_eq!(tool.summary_fields(), &["path"]);
    }

    #[test]
    fn permission_profile_returns_path_facet() {
        let tool = ReadImageTool::new(workspace_root());
        let profile = tool.permission_profile(&serde_json::json!({"path": "img.png"}));
        assert_eq!(profile.len(), 1);
        assert_eq!(profile[0].nature, ToolNature::Read);
        assert_eq!(profile[0].resource.as_deref(), Some("img.png"));
        assert_eq!(profile[0].resource_kind, Some(ToolResourceKind::Path));
    }

    #[test]
    fn permission_profile_without_path_returns_bare_read_facet() {
        let tool = ReadImageTool::new(workspace_root());
        let profile = tool.permission_profile(&serde_json::json!({}));
        assert_eq!(profile.len(), 1);
        assert_eq!(profile[0].nature, ToolNature::Read);
        assert!(profile[0].resource.is_none());
    }

    #[tokio::test]
    async fn execute_with_output_default_delegates_to_execute() {
        let tool = ReadImageTool::new(workspace_root());
        let output = tool
            .execute_with_output(serde_json::json!({"path": "x.png"}))
            .await;
        assert!(output.result.is_error);
        assert!(output.next_provider_parts.is_empty());
    }

    #[tokio::test]
    async fn execute_authorized_rejects_text_file_with_png_extension() {
        let dir = tempfile::tempdir().unwrap();
        let fake_png = dir.path().join("fake.png");
        std::fs::write(&fake_png, b"not a png file").unwrap();

        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let auth = vec![
            ToolExecutionAuthorization::for_path(
                "read_image",
                ToolNature::Read,
                dir.path(),
                "fake.png",
                ToolAuthorizationScope::Once,
            )
            .unwrap(),
        ];
        let output = tool
            .execute_authorized_with_output(
                serde_json::json!({"path": fake_png.to_string_lossy()}),
                &auth,
            )
            .await;

        assert!(output.result.is_error, "non-image file must be rejected");
        assert!(output.next_provider_parts.is_empty());
    }

    #[tokio::test]
    async fn execute_authorized_rejects_attach_image_authorization() {
        let workspace = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let img_path = external.path().join("test.png");
        std::fs::write(&img_path, MINIMAL_PNG).unwrap();
        let canonical = img_path.canonicalize().unwrap();

        let tool = ReadImageTool::new(workspace.path().to_path_buf());
        let attach_auth = vec![
            ToolExecutionAuthorization::for_path(
                "attach_image",
                ToolNature::Read,
                external.path(),
                "test.png",
                ToolAuthorizationScope::Once,
            )
            .unwrap(),
        ];
        let output = tool
            .execute_authorized_with_output(
                serde_json::json!({"path": canonical.to_string_lossy()}),
                &attach_auth,
            )
            .await;

        assert!(
            output.result.is_error,
            "attach_image authorization must not authorize read_image"
        );
        assert!(output.next_provider_parts.is_empty());
    }

    #[tokio::test]
    async fn execute_authorized_rejects_read_authorization() {
        let workspace = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let img_path = external.path().join("test.png");
        std::fs::write(&img_path, MINIMAL_PNG).unwrap();
        let canonical = img_path.canonicalize().unwrap();

        let tool = ReadImageTool::new(workspace.path().to_path_buf());
        let read_auth = vec![
            ToolExecutionAuthorization::for_path(
                "read",
                ToolNature::Read,
                external.path(),
                "test.png",
                ToolAuthorizationScope::Once,
            )
            .unwrap(),
        ];
        let output = tool
            .execute_authorized_with_output(
                serde_json::json!({"path": canonical.to_string_lossy()}),
                &read_auth,
            )
            .await;

        assert!(
            output.result.is_error,
            "read authorization must not authorize read_image"
        );
        assert!(output.next_provider_parts.is_empty());
    }

    #[tokio::test]
    async fn execute_authorized_rejects_fifo() {
        let dir = tempfile::tempdir().unwrap();
        let fifo_path = dir.path().join("pipe.png");
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            std::os::unix::fs::symlink("/dev/null", dir.path().join("pipe.png")).ok();
        }
        let tool = ReadImageTool::new(dir.path().to_path_buf());
        let output = tool
            .execute_authorized_with_output(serde_json::json!({"path": "pipe.png"}), &[])
            .await;
        assert!(output.result.is_error);
        assert!(output.next_provider_parts.is_empty());
    }

    /// Minimal valid 1×1 white PNG (67 bytes).
    const MINIMAL_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // signature
        0x00, 0x00, 0x00, 0x0d, // IHDR length
        0x49, 0x48, 0x44, 0x52, // "IHDR"
        0x00, 0x00, 0x00, 0x01, // width=1
        0x00, 0x00, 0x00, 0x01, // height=1
        0x08, 0x02, 0x00, 0x00, 0x00, // bitdepth=8, colortype=RGB
        0x90, 0x77, 0x53, 0xde, // CRC
        0x00, 0x00, 0x00, 0x0c, // IDAT length
        0x49, 0x44, 0x41, 0x54, // "IDAT"
        0x78, 0x9c, 0x63, 0xf8, 0xff, 0xff, 0x3f, 0x00, 0x05, 0xfe, 0x02, 0xfe, 0xa3, 0x35, 0x81,
        0x84, // compressed data + CRC
        0x00, 0x00, 0x00, 0x00, // IEND length
        0x49, 0x45, 0x4e, 0x44, // "IEND"
        0xae, 0x42, 0x60, 0x82, // IEND CRC
    ];
}
