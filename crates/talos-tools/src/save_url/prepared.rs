//! Read/network admission followed by complete-image, recoverable write admission.
use super::*;
use talos_core::tool::{
    PreparedFileMutationDescription, PreparedFileMutationEvidence, PreparedNormalStage,
    PreparedNormalToolInvocation, PreparedPlanIdentity, ToolExecutionOutput, ToolProvenance,
};
use talos_permission::{AdmittedInvocation, PermissionRequest};

pub(super) fn prepare(
    tool: &SaveUrlTool,
    input: &Value,
    provenance: ToolProvenance,
) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
    if input.get("retain_backup").and_then(Value::as_bool) != Some(true)
        || input.get("recovery_id").is_some()
    {
        return Ok(None);
    }
    let parsed: SaveUrlInput = serde_json::from_value(input.clone()).map_err(|e| e.to_string())?;
    if parsed.destination.is_empty() {
        return Err("destination must not be empty".into());
    }
    let profile = tool
        .permission_profile(input)
        .into_iter()
        .filter(|facet| facet.nature != ToolNature::Write)
        .collect();
    Ok(Some(Box::new(PreparedSave {
        tool: SaveUrlTool {
            client: tool.client.clone(),
            max_body_bytes: tool.max_body_bytes,
            recovery: tool.recovery.clone(),
        },
        parsed,
        input: input.clone(),
        provenance,
        profile,
        identity: PreparedPlanIdentity::new(),
        images: None,
        mutation: None,
    })))
}

struct Images {
    path: PathBuf,
    before: Vec<u8>,
    after: Vec<u8>,
}

struct PreparedSave {
    tool: SaveUrlTool,
    parsed: SaveUrlInput,
    input: Value,
    provenance: ToolProvenance,
    profile: Vec<ToolPermissionFacet>,
    identity: PreparedPlanIdentity,
    images: Option<Images>,
    mutation: Option<PreparedFileMutationEvidence>,
}

impl PreparedSave {
    async fn advance(
        mut self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> Result<PreparedNormalStage, String> {
        let proof = admission
            .downcast::<AdmittedInvocation>()
            .map_err(|_| "missing exact save admission")?;
        let request = PermissionRequest::new(
            "save_url",
            self.provenance.clone(),
            &self.profile,
            &self.input,
        );
        let authorizations = proof
            .consume_for_prepared(&request, &self.identity)
            .map_err(|_| "save admission does not match prepared plan")?;
        let path = admitted_path(&authorizations, ToolNature::Read)?;
        if let Some(images) = &self.images {
            let write_path = admitted_path(&authorizations, ToolNature::Write)?;
            if path != images.path || write_path != images.path {
                return Err("destination identity changed after download".into());
            }
            let id = self
                .tool
                .recovery
                .replace(&path, &images.before, &images.after, None)
                .map_err(|e| e.to_string())?;
            return Ok(PreparedNormalStage::Complete(ToolExecutionOutput::success(
                format!(
                    "saved {} bytes to {}; recovery_id: {id}",
                    images.after.len(),
                    self.parsed.destination,
                ),
            )));
        }
        let before = crate::file_tools::read_bounded_regular_file(&path)
            .map_err(|e| e.to_string())?
            .0;
        let after = self
            .tool
            .download(
                &self.parsed.url,
                self.tool
                    .max_body_bytes
                    .min(talos_core::tool::MAX_FILE_MUTATION_REVIEW_BYTES),
            )
            .await?;
        self.identity = PreparedPlanIdentity::new();
        // Binary images retain human approval rather than being truncated or recoded.
        if let (Ok(before), Ok(after)) = (std::str::from_utf8(&before), std::str::from_utf8(&after))
        {
            self.mutation = Some(
                PreparedFileMutationEvidence::new(
                    self.identity.clone(),
                    PreparedFileMutationDescription {
                        path: path.clone(),
                        before: before.into(),
                        after: Some(after.into()),
                    },
                )
                .map_err(|e| e.to_string())?,
            );
        }
        self.images = Some(Images {
            path,
            before,
            after,
        });
        self.profile = [ToolNature::Read, ToolNature::Write]
            .into_iter()
            .map(|nature| {
                ToolPermissionFacet::with_resource(
                    nature,
                    &self.parsed.destination,
                    ToolResourceKind::Path,
                )
            })
            .collect();
        Ok(PreparedNormalStage::Continue(self))
    }
}

// The proof covers exactly one path per nature. Use its normalized target;
// never reinterpret relative input against ambient process cwd.
fn admitted_path(
    authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    nature: ToolNature,
) -> Result<PathBuf, String> {
    let mut paths = authorizations.iter().filter(|authorization| {
        let path = authorization.normalized_path();
        path.is_absolute()
            && path.to_str().is_some_and(|resource| {
                authorization.authorizes_path("save_url", nature, path, resource)
            })
    });
    let path = paths
        .next()
        .ok_or("missing exact save path authorization")?
        .normalized_path();
    if paths.next().is_some() || path.parent().is_none() {
        return Err("ambiguous save path authorization".into());
    }
    Ok(path.to_path_buf())
}

#[async_trait]
impl PreparedNormalToolInvocation for PreparedSave {
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
        match self.advance(admission).await {
            Ok(stage) => stage,
            Err(error) => PreparedNormalStage::Complete(ToolExecutionOutput::error(error)),
        }
    }
    async fn execute(self: Box<Self>, _: Box<dyn std::any::Any + Send>) -> ToolExecutionOutput {
        ToolExecutionOutput::error("recoverable save requires staged admission")
    }
}

#[cfg(test)]
#[path = "prepared_tests.rs"]
mod tests;
