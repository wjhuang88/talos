//! Read-admitted capture followed by exact-plan recoverable deletion.
use super::*;
use talos_core::tool::{
    PreparedFileMutationDescription, PreparedFileMutationEvidence, PreparedNormalStage,
    PreparedNormalToolInvocation, PreparedPlanIdentity, ToolExecutionOutput, ToolProvenance,
};
use talos_permission::{AdmittedInvocation, PermissionRequest};

pub(super) fn prepare(
    tool: &DeleteTool,
    input: &Value,
    provenance: ToolProvenance,
) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
    if input.get("retain_backup").and_then(Value::as_bool) != Some(true)
        || input.get("recovery_id").is_some()
    {
        return Ok(None);
    }
    let delete: DeleteInput = serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
    Ok(Some(Box::new(PreparedDelete {
        root: tool.workspace_root.clone(),
        snapshots: tool.snapshots.clone(),
        recovery: tool.recovery.clone(),
        profile: vec![ToolPermissionFacet::with_resource(
            ToolNature::Read,
            &delete.path,
            ToolResourceKind::Path,
        )],
        path: delete.path,
        input: input.clone(),
        provenance,
        identity: PreparedPlanIdentity::new(),
        mutation: None,
        captured: None,
    })))
}

struct PreparedDelete {
    root: PathBuf,
    snapshots: Option<FileSnapshotRegistry>,
    recovery: std::sync::Arc<super::super::recovery::FileRecoveryStore>,
    path: String,
    input: Value,
    provenance: ToolProvenance,
    identity: PreparedPlanIdentity,
    profile: Vec<ToolPermissionFacet>,
    mutation: Option<PreparedFileMutationEvidence>,
    captured: Option<(PathBuf, Vec<u8>)>,
}

impl PreparedDelete {
    fn advance(
        mut self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> Result<PreparedNormalStage, String> {
        let proof = admission
            .downcast::<AdmittedInvocation>()
            .map_err(|_| "missing exact delete admission")?;
        let request = PermissionRequest::new(
            "delete",
            self.provenance.clone(),
            &self.profile,
            &self.input,
        );
        let authorizations = proof
            .consume_for_prepared(&request, &self.identity)
            .map_err(|_| "delete admission does not match prepared plan")?;
        let path = resolve_authorized_path(
            &self.root,
            &self.path,
            "delete",
            ToolNature::Read,
            &authorizations,
        )
        .map_err(|e| e.to_string())?;
        if let Some((captured_path, before)) = &self.captured {
            let writable = resolve_authorized_path(
                &self.root,
                &self.path,
                "delete",
                ToolNature::Write,
                &authorizations,
            )
            .map_err(|e| e.to_string())?;
            if &path != captured_path || writable != path {
                return Err(FileToolError::PathIdentityChanged.to_string());
            }
            let lock = self
                .snapshots
                .as_ref()
                .map(|r| r.path_lock(&path))
                .transpose()
                .map_err(|e| e.to_string())?;
            let id = self
                .recovery
                .remove_captured(&path, Some(before), lock.as_deref())
                .map_err(|e| e.to_string())?;
            let receipt = format!(
                "recovery_id: {id}; restore with this delete instance using path and recovery_id within 15 minutes. Backup is memory-only; restores bytes and basic permissions, not ownership, ACLs or timestamps."
            );
            if let Some(registry) = &self.snapshots {
                registry.invalidate_path(&path).map_err(|e| {
                    format!("file deleted; {receipt}; snapshot invalidation failed: {e}")
                })?;
            }
            return Ok(PreparedNormalStage::Complete(ToolExecutionOutput::success(
                format!("deleted {}; {receipt}", self.path),
            )));
        }
        let before = super::super::recovery::read_bounded_regular_file(&path)
            .map_err(|e| e.to_string())?
            .0;
        self.identity = PreparedPlanIdentity::new();
        self.mutation = std::str::from_utf8(&before)
            .ok()
            .filter(|text| !text.contains('\0'))
            .map(|text| {
                PreparedFileMutationEvidence::new(
                    self.identity.clone(),
                    PreparedFileMutationDescription {
                        path: path.clone(),
                        before: text.to_owned(),
                        after: None,
                    },
                )
            })
            .transpose()
            .map_err(|e| e.to_string())?;
        self.captured = Some((path, before));
        self.profile = [ToolNature::Read, ToolNature::Write]
            .into_iter()
            .map(|nature| {
                ToolPermissionFacet::with_resource(nature, &self.path, ToolResourceKind::Path)
            })
            .collect();
        Ok(PreparedNormalStage::Continue(self))
    }
}

#[async_trait]
impl PreparedNormalToolInvocation for PreparedDelete {
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
        ToolExecutionOutput::error("recoverable delete requires staged admission")
    }
}

#[cfg(test)]
#[path = "prepared_delete_tests.rs"]
mod tests;
