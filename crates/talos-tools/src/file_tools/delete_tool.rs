use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolExecutionAuthorization, ToolFamily, ToolNature, ToolPermissionFacet,
    ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;

use super::{
    DeleteError, FileSnapshotRegistry, FileToolError, resolve_authorized_path,
    resolve_workspace_path,
};

mod prepared_delete;

/// Input parameters for the [`DeleteTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteInput {
    /// Path to the file or directory to delete.
    pub path: String,
}

/// A tool that deletes files or directories within the workspace.
pub struct DeleteTool {
    workspace_root: PathBuf,
    snapshots: Option<FileSnapshotRegistry>,
    recovery: std::sync::Arc<super::recovery::FileRecoveryStore>,
}

impl DeleteTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            snapshots: None,
            recovery: Default::default(),
        }
    }

    /// Creates a delete tool that invalidates shared file snapshots.
    #[must_use]
    pub fn with_snapshot_registry(
        workspace_root: PathBuf,
        snapshots: FileSnapshotRegistry,
    ) -> Self {
        Self {
            workspace_root,
            snapshots: Some(snapshots),
            recovery: Default::default(),
        }
    }

    async fn execute_inner(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<String, FileToolError> {
        let recovery_id = input
            .get("recovery_id")
            .map(|value| {
                value
                    .as_str()
                    .filter(|id| !id.is_empty() && id.len() <= 64)
                    .map(str::to_owned)
                    .ok_or_else(|| FileToolError::InvalidInput("invalid recovery id".into()))
            })
            .transpose()?;
        let retain_backup = match input.get("retain_backup") {
            None => false,
            Some(Value::Bool(value)) => *value,
            _ => {
                return Err(FileToolError::InvalidInput(
                    "retain_backup must be a boolean".into(),
                ));
            }
        };
        if recovery_id.is_some() && input.get("retain_backup").is_some() {
            return Err(FileToolError::InvalidInput(
                "recovery cannot be combined with retain_backup".into(),
            ));
        }
        let delete_input: DeleteInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_authorized_path(
            &self.workspace_root,
            &delete_input.path,
            "delete",
            ToolNature::Write,
            authorizations,
        )?;

        if retain_backup {
            let readable = resolve_authorized_path(
                &self.workspace_root,
                &delete_input.path,
                "delete",
                ToolNature::Read,
                authorizations,
            )?;
            if readable != path {
                return Err(FileToolError::PathIdentityChanged);
            }
        }
        let path_lock = self
            .snapshots
            .as_ref()
            .map(|registry| registry.path_lock(&path))
            .transpose()?;
        if let Some(id) = recovery_id {
            self.recovery.restore(&path, &id, path_lock.as_deref())?;
            if let Some(registry) = &self.snapshots {
                registry.invalidate_path(&path).map_err(|error| FileToolError::AtomicWriteFailed(
                    format!("file restored and recovery id consumed; snapshot invalidation failed: {error}")
                ))?;
            }
            return Ok(format!(
                "restored {}; recovery id consumed",
                delete_input.path
            ));
        }
        if !path.exists() {
            return Err(FileToolError::FileNotFound(delete_input.path));
        }

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .map_err(FileToolError::Io)?;

        if path == canonical_root {
            return Err(FileToolError::InvalidInput(
                DeleteError::WorkspaceRoot.to_string(),
            ));
        }

        let receipt = if retain_backup {
            Some(self.recovery.remove(&path, path_lock.as_deref())?)
        } else {
            None
        };
        if receipt.is_none() {
            if path.is_dir() {
                tokio::fs::remove_dir_all(&path).await?;
            } else {
                tokio::fs::remove_file(&path).await?;
            }
        }
        if let Some(registry) = &self.snapshots {
            registry.invalidate_path(&path).map_err(|error| {
                FileToolError::AtomicWriteFailed(format!(
                    "path deleted; recovery_id: {receipt:?}; snapshot invalidation failed: {error}"
                ))
            })?;
        }

        let display = path
            .strip_prefix(&canonical_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        Ok(match receipt {
            Some(id) => format!(
                "deleted {display}; recovery_id: {id}; restore with this delete instance using path and recovery_id within 15 minutes. Backup is memory-only; restores bytes and basic permissions, not ownership, ACLs or timestamps."
            ),
            None => format!("deleted {display}"),
        })
    }
}

#[async_trait]
impl AgentTool for DeleteTool {
    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: talos_core::tool::ToolProvenance,
    ) -> Result<Option<Box<dyn talos_core::tool::PreparedNormalToolInvocation>>, String> {
        prepared_delete::prepare(self, input, provenance)
    }

    fn name(&self) -> &str {
        "delete"
    }

    fn description(&self) -> &str {
        "Delete a file or directory (cannot delete workspace root)"
    }

    fn parameters(&self) -> Value {
        let mut schema = tool_parameters!(DeleteInput);
        schema["properties"]["retain_backup"] = serde_json::json!({
            "type": "boolean", "description": "Retain an in-memory regular-file backup for 15 minutes; maximum 2 MiB/file, 16 MiB and 64 receipts per tool instance. Directories unsupported."
        });
        schema["properties"]["recovery_id"] = serde_json::json!({
            "type": "string", "minLength": 1, "maxLength": 64,
            "description": "Restore a receipt from this delete instance using path and recovery_id only. Refuses to overwrite an existing path."
        });
        schema
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::File
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input, &[]).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    async fn execute_authorized(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> ToolResult {
        match self.execute_inner(input, authorizations).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let Some(path) = input.get("path").and_then(Value::as_str) else {
            return vec![ToolPermissionFacet::new(ToolNature::Write)];
        };

        let description = match resolve_workspace_path(&self.workspace_root, path) {
            Ok(resolved) if resolved.is_dir() => "directory deletion",
            Ok(_) => "file deletion",
            Err(_) => "delete path",
        };

        let mut facets = vec![
            ToolPermissionFacet::with_resource(ToolNature::Write, path, ToolResourceKind::Path)
                .with_description(description),
        ];
        if input.get("retain_backup").and_then(Value::as_bool) == Some(true) {
            facets.push(
                ToolPermissionFacet::with_resource(ToolNature::Read, path, ToolResourceKind::Path)
                    .with_description("read original bytes for deletion backup"),
            );
        }
        facets
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}
