//! Native Pull scope discovery without fetch scheduling or integration prediction.

use super::*;

impl AddPlan {
    // Native preparation gathers scope only; it does not fetch, predict new
    // commits, or choose an integration command. Sender routing stays on the
    // legacy path until recursive configuration admission is complete.
    pub(super) async fn discover_native_pull_scope(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        self.require_pull_parent_metadata(authorizations)?;
        self.pull_hook_evidence = self.capture_native_pull_hooks(authorizations).await?;
        let mut inventory = self.pull_inventory.take().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull parent inventory missing".into())
        })?;
        let parent_ready = self.advance_pull_parent_inventory(&mut inventory, authorizations)?;
        self.pull_inventory = Some(inventory);
        if !parent_ready {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        if self.native_pull_modules.is_none() {
            self.native_pull_modules =
                Some(super::super::pull_inventory::Inventory::stored_modules(
                    &self.repository.common_dir,
                    self.pull_reference_budget.clone(),
                )?);
        }
        let modules = self.native_pull_modules.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Native Pull module inventory missing".into())
        })?;
        // The parent metadata capability is not a grant to read child configs.
        // Request every module-name frontier before inspecting its identities.
        let admitted = |path: &Path| {
            authorizations.iter().any(|token| {
                token.authorizes_path(
                    "git_pull",
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            })
        };
        if modules.iter_paths().all(&admitted) && !modules.complete() {
            modules.advance(admitted)?;
        }
        for path in modules.iter_paths() {
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
            }) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "native Pull stored-module identity frontier; config contents deferred",
                )?);
            }
        }
        if !modules.complete() {
            self.phase = Phase::PullNativePreparation;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        // These roots are candidates, not yet validated repositories. Exact
        // config/metadata admission must precede child configuration discovery.
        for root in modules.stored_module_roots() {
            for leaf in ["HEAD", "commondir", "config", "config.worktree"] {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &root.join(leaf),
                    "native Pull stored child fixed configuration/layout frontier",
                )?);
            }
        }
        let (remote, destinations) = self.resolve_native_pull_destination().await?;
        self.pull_remote = Some(remote);
        self.pull_destinations = destinations;
        self.pull_transport_supported = true;
        self.facets.extend(self.native_pull_parent_facets()?);
        self.phase = Phase::PullNativeMutation;
        self.identity = PreparedPlanIdentity::new();
        self.revalidate()?;
        Ok(self)
    }
}
