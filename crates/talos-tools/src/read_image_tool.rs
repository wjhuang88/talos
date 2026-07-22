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
    AgentTool, ToolExecutionAuthorization, ToolExecutionOutput, ToolFamily, ToolNature, ToolResult,
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
        let parsed: ReadImageInput = match serde_json::from_value(input) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(format!("Invalid input: {e}")),
        };
        ToolResult::error(format!(
            "read_image requires authorized execution. Path: {}",
            parsed.path
        ))
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
                return ToolExecutionOutput::error(format!(
                    "Permission denied: path '{}' is outside the workspace and no matching authorization was provided.",
                    parsed.path
                ));
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
}
