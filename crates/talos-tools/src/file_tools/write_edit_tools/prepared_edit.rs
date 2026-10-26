//! Read-admitted capture and one-use execution of recoverable string edits.

use super::*;
use talos_core::tool::{
    PreparedFileMutationDescription, PreparedFileMutationEvidence, PreparedNormalStage,
    PreparedNormalToolInvocation, PreparedPlanIdentity, ToolExecutionOutput, ToolProvenance,
};
use talos_permission::{AdmittedInvocation, PermissionRequest};

pub(super) fn prepare(
    tool: &EditTool,
    input: &Value,
    provenance: ToolProvenance,
) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
    if input.get("retain_backup").and_then(Value::as_bool) != Some(true)
        || input.get("snapshot_id").is_some()
        || input.get("recovery_id").is_some()
    {
        return Ok(None);
    }
    let edit: EditInput = serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
    Ok(Some(Box::new(PreparedEdit {
        root: tool.workspace_root.clone(),
        snapshots: tool.snapshots.clone(),
        recovery: tool.recovery.clone(),
        input: input.clone(),
        profile: vec![ToolPermissionFacet::with_resource(
            ToolNature::Read,
            &edit.path,
            ToolResourceKind::Path,
        )],
        edit,
        provenance,
        identity: PreparedPlanIdentity::new(),
        mutation: None,
    })))
}

struct PreparedEdit {
    root: PathBuf,
    snapshots: Option<FileSnapshotRegistry>,
    recovery: std::sync::Arc<super::super::recovery::FileRecoveryStore>,
    input: Value,
    edit: EditInput,
    provenance: ToolProvenance,
    identity: PreparedPlanIdentity,
    profile: Vec<ToolPermissionFacet>,
    mutation: Option<PreparedFileMutationEvidence>,
}

impl PreparedEdit {
    fn advance(
        mut self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> Result<PreparedNormalStage, String> {
        let proof = admission
            .downcast::<AdmittedInvocation>()
            .map_err(|_| "missing exact edit admission".to_owned())?;
        let request =
            PermissionRequest::new("edit", self.provenance.clone(), &self.profile, &self.input);
        let authorizations = proof
            .consume_for_prepared(&request, &self.identity)
            .map_err(|_| "edit admission does not match prepared plan".to_owned())?;
        let path = resolve_authorized_path(
            &self.root,
            &self.edit.path,
            "edit",
            ToolNature::Read,
            &authorizations,
        )
        .map_err(|e| e.to_string())?;
        if let Some(mutation) = &self.mutation {
            let description = mutation.description();
            let write_path = resolve_authorized_path(
                &self.root,
                &self.edit.path,
                "edit",
                ToolNature::Write,
                &authorizations,
            )
            .map_err(|e| e.to_string())?;
            if path != description.path || write_path != path {
                return Err(FileToolError::PathIdentityChanged.to_string());
            }
            let after = description.after.as_ref().ok_or("missing edit result")?;
            let lock = self
                .snapshots
                .as_ref()
                .map(|r| r.path_lock(&path))
                .transpose()
                .map_err(|e| e.to_string())?;
            let id = self
                .recovery
                .replace(
                    &path,
                    description.before.as_bytes(),
                    after.as_bytes(),
                    lock.as_deref(),
                )
                .map_err(|e| e.to_string())?;
            let receipt = super::recovery_receipt(id);
            if let Some(registry) = &self.snapshots {
                registry.invalidate_path(&path).map_err(|e| {
                    format!("file edited{receipt}; snapshot invalidation failed: {e}")
                })?;
            }
            return Ok(PreparedNormalStage::Complete(ToolExecutionOutput::success(
                format!(
                    "edited {}{receipt}\ndiff:\n{}",
                    self.edit.path,
                    super::bounded_replacement_diff(&self.edit.old_string, &self.edit.new_string),
                ),
            )));
        }
        let before = String::from_utf8(
            super::super::recovery::read_bounded_regular_file(&path)
                .map_err(|e| e.to_string())?
                .0,
        )
        .map_err(|_| "edit requires a text file")?;
        let start = before
            .find(&self.edit.old_string)
            .ok_or("edit string not found")?;
        let updated_len = before
            .len()
            .checked_sub(self.edit.old_string.len())
            .and_then(|len| len.checked_add(self.edit.new_string.len()))
            .filter(|len| *len <= talos_core::tool::MAX_FILE_MUTATION_REVIEW_BYTES)
            .ok_or("edit result exceeds complete review image limit")?;
        let mut after = String::with_capacity(updated_len);
        after.push_str(&before[..start]);
        after.push_str(&self.edit.new_string);
        after.push_str(&before[start + self.edit.old_string.len()..]);
        self.identity = PreparedPlanIdentity::new();
        self.mutation = Some(
            PreparedFileMutationEvidence::new(
                self.identity.clone(),
                PreparedFileMutationDescription {
                    path,
                    before,
                    after: Some(after),
                },
            )
            .map_err(|e| e.to_string())?,
        );
        self.profile = [ToolNature::Read, ToolNature::Write]
            .into_iter()
            .map(|nature| {
                ToolPermissionFacet::with_resource(nature, &self.edit.path, ToolResourceKind::Path)
            })
            .collect();
        Ok(PreparedNormalStage::Continue(self))
    }
}

#[async_trait]
impl PreparedNormalToolInvocation for PreparedEdit {
    fn plan_identity(&self) -> Option<&PreparedPlanIdentity> {
        Some(&self.identity)
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.profile
    }
    fn prepared_file_mutation_evidence(&self) -> Option<PreparedFileMutationEvidence> {
        self.mutation.clone()
    }
    async fn execute_stage(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> PreparedNormalStage {
        match self.advance(admission) {
            Ok(stage) => stage,
            Err(error) => PreparedNormalStage::Complete(ToolExecutionOutput::error(error)),
        }
    }
    async fn execute(self: Box<Self>, _: Box<dyn std::any::Any + Send>) -> ToolExecutionOutput {
        ToolExecutionOutput::error("recoverable edit requires staged admission")
    }
}

#[cfg(test)]
#[path = "prepared_edit_tests.rs"]
mod tests;
