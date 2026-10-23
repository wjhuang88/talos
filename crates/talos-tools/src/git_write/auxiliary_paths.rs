//! Identity-bound local Git auxiliary file paths.
//!
//! The named resolver capability permits only bounded component metadata and
//! link-target inspection. File bodies require fresh exact Path Read tokens.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use talos_core::tool::{ToolExecutionAuthorization, ToolNature};

use super::{GitToolError, pull_resolution, snapshot};

#[derive(Debug, PartialEq, Eq)]
struct Source {
    spelling: PathBuf,
    resolution: pull_resolution::Resolution,
    leaf: Option<snapshot::RegularEvidence>,
}

/// Frozen operation source set, resolution chain and admitted leaf snapshots.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct AuxiliaryPaths {
    tool: String,
    workspace: PathBuf,
    anchor: PathBuf,
    sources: Vec<Source>,
    budget: pull_resolution::Budget,
}

impl AuxiliaryPaths {
    /// Resolve only paths frozen from the admitted Git configuration and
    /// selected operation. Does not inspect any file contents.
    pub(super) fn resolve(
        tool: &str,
        workspace: &Path,
        anchor: &Path,
        paths: impl IntoIterator<Item = PathBuf>,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<Self, GitToolError> {
        let capability = authorizations
            .iter()
            .any(|token| token.authorizes_git_auxiliary_path_resolution(tool));
        if !capability
            || !authorizations.iter().any(|token| {
                token.authorizes_path(
                    tool,
                    ToolNature::Read,
                    workspace,
                    anchor.to_str().unwrap_or_default(),
                )
            })
        {
            return Err(invalid());
        }
        let mut budget = pull_resolution::Budget::default();
        let mut sources = Vec::new();
        for spelling in paths.into_iter().collect::<BTreeSet<_>>() {
            if !spelling.is_absolute() {
                return Err(invalid());
            }
            let resolution = pull_resolution::Resolution::capture_file(&spelling, &mut budget)?;
            sources.push(Source {
                spelling,
                resolution,
                leaf: None,
            });
        }
        Ok(Self {
            tool: tool.to_owned(),
            workspace: workspace.to_owned(),
            anchor: anchor.to_owned(),
            sources,
            budget,
        })
    }

    /// Add configuration/receiver sources revealed by a later admitted stage
    /// without resetting the invocation allowance or replacing prior evidence.
    pub(super) fn extend(
        &mut self,
        tool: &str,
        paths: impl IntoIterator<Item = PathBuf>,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        if tool != self.tool
            || !authorizations
                .iter()
                .any(|token| token.authorizes_git_auxiliary_path_resolution(tool))
            || !self.authorizes_anchor(authorizations)
        {
            return Err(invalid());
        }
        self.revalidate_resolutions_inner()?;
        let existing: BTreeSet<_> = self
            .sources
            .iter()
            .map(|source| source.spelling.clone())
            .collect();
        for spelling in paths.into_iter().collect::<BTreeSet<_>>() {
            if existing.contains(&spelling) {
                continue;
            }
            if !spelling.is_absolute() {
                return Err(invalid());
            }
            let resolution =
                pull_resolution::Resolution::capture_file(&spelling, &mut self.budget)?;
            self.sources.push(Source {
                spelling,
                resolution,
                leaf: None,
            });
        }
        self.sources
            .sort_by(|left, right| left.spelling.cmp(&right.spelling));
        self.revalidate_resolutions_inner()
    }

    pub(super) fn targets(&self) -> impl Iterator<Item = &Path> {
        self.sources
            .iter()
            .map(|source| source.resolution.resolved())
    }

    #[cfg(test)]
    pub(super) fn related_paths(&self) -> impl Iterator<Item = &Path> {
        self.sources
            .iter()
            .flat_map(|source| source.resolution.related_paths())
    }

    pub(super) fn related_paths_for(&self, spelling: &Path) -> Option<Vec<PathBuf>> {
        let source = self
            .sources
            .iter()
            .find(|source| source.spelling == spelling)?;
        Some(
            source
                .resolution
                .related_paths()
                .map(Path::to_path_buf)
                .collect(),
        )
    }

    pub(super) fn add_read_frontier(
        &self,
        facets: &mut Vec<talos_core::tool::ToolPermissionFacet>,
    ) -> Result<(), GitToolError> {
        for target in self.targets() {
            if !facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == target.to_str()
            }) {
                facets.push(super::prepared_add::path_facet(
                    ToolNature::Read,
                    target,
                    "resolved Git auxiliary target; content Read requires fresh approval",
                )?);
            }
        }
        Ok(())
    }

    pub(super) fn revalidate_resolutions(
        &self,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        if !authorizations
            .iter()
            .any(|token| token.authorizes_git_auxiliary_path_resolution(&self.tool))
            || !self.authorizes_anchor(authorizations)
        {
            return Err(invalid());
        }
        self.revalidate_resolutions_inner()
    }

    pub(super) fn original_leaf_is_link(&self, spelling: &Path) -> bool {
        self.sources
            .iter()
            .find(|source| source.spelling == spelling)
            .is_some_and(|source| source.resolution.original_leaf_is_link())
    }

    pub(super) fn leaf_evidence(&self, spelling: &Path) -> Option<&snapshot::RegularEvidence> {
        self.sources
            .iter()
            .find(|source| source.spelling == spelling)?
            .leaf
            .as_ref()
    }

    /// Consume the new exact target Read frontier and retain bytes/leaf
    /// identity only after every target token is present.
    pub(super) fn capture_admitted(
        &mut self,
        authorizations: &[ToolExecutionAuthorization],
        hooks: &[PathBuf],
    ) -> Result<(), GitToolError> {
        self.capture_with_budget(authorizations, hooks, None, &[])
    }

    /// Native Pull shares its invocation evidence allowance across child hooks.
    pub(super) fn capture_native_pull(
        &mut self,
        authorizations: &[ToolExecutionAuthorization],
        hooks: &[PathBuf],
        shared: &super::pull_inventory::ReferenceBudget,
        roots: &[PathBuf],
    ) -> Result<(), GitToolError> {
        if self.tool != "git_pull" {
            return Err(invalid());
        }
        self.capture_with_budget(authorizations, hooks, Some(shared), roots)
    }

    fn capture_with_budget(
        &mut self,
        authorizations: &[ToolExecutionAuthorization],
        hooks: &[PathBuf],
        shared: Option<&super::pull_inventory::ReferenceBudget>,
        roots: &[PathBuf],
    ) -> Result<(), GitToolError> {
        if !authorizations
            .iter()
            .any(|token| token.authorizes_git_auxiliary_path_resolution(&self.tool))
            || !self.authorizes_anchor(authorizations)
        {
            return Err(invalid());
        }
        self.revalidate_resolutions(authorizations)?;
        if let Some(hook) = hooks.iter().find(|hook| self.original_leaf_is_link(hook)) {
            return Err(unsupported_hook(hook));
        }
        self.require_target_reads(authorizations, roots)?;
        for source in &mut self.sources {
            if let Some(evidence) = &source.leaf {
                evidence
                    .revalidate(source.resolution.resolved())
                    .map_err(|_| invalid())?;
                continue;
            }
            let identity_handles = if cfg!(windows) { 3 } else { 1 };
            self.budget.charge(identity_handles, 0)?;
            let mut allowance = snapshot::Budget {
                entries: self.budget.entries,
                bytes: self.budget.bytes,
            };
            let evidence = if let Some(shared) = shared {
                shared.capture_with_budget(source.resolution.resolved(), &mut allowance)
            } else {
                snapshot::RegularEvidence::capture(source.resolution.resolved(), &mut allowance)
                    .map_err(|_| invalid())
            };
            let entries = self.budget.entries - allowance.entries;
            let bytes = self.budget.bytes - allowance.bytes;
            self.budget.charge(entries, bytes)?;
            source.leaf = Some(evidence?);
        }
        for hook in hooks {
            let evidence = self.leaf_evidence(hook).ok_or_else(invalid)?;
            if let snapshot::Entry::File(permissions, _) = &evidence.entry {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if permissions.mode() & 0o111 != 0 {
                        return Err(unsupported_hook(hook));
                    }
                }
                #[cfg(not(unix))]
                {
                    // Git for Windows can execute an existing hook without
                    // Unix executable bits. Preserve the existing refusal.
                    let _ = permissions;
                    return Err(unsupported_hook(hook));
                }
            }
        }
        self.revalidate_with_roots(authorizations, roots)
    }

    /// Recheck the frozen component chain and admitted regular-file leaf
    /// evidence without discovering a new path or reading another source.
    pub(super) fn revalidate(
        &self,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        self.revalidate_with_roots(authorizations, &[])
    }

    pub(super) fn revalidate_native_pull(
        &self,
        authorizations: &[ToolExecutionAuthorization],
        roots: &[PathBuf],
    ) -> Result<(), GitToolError> {
        self.revalidate_with_roots(authorizations, roots)
    }

    fn revalidate_with_roots(
        &self,
        authorizations: &[ToolExecutionAuthorization],
        roots: &[PathBuf],
    ) -> Result<(), GitToolError> {
        if !authorizations
            .iter()
            .any(|token| token.authorizes_git_auxiliary_path_resolution(&self.tool))
        {
            return Err(invalid());
        }
        if !self.authorizes_anchor(authorizations) {
            return Err(invalid());
        }
        self.revalidate_resolutions(authorizations)?;
        self.require_target_reads(authorizations, roots)?;
        for source in &self.sources {
            let target = source.resolution.resolved();
            let evidence = source.leaf.as_ref().ok_or_else(invalid)?;
            evidence.revalidate(target).map_err(|_| invalid())?;
        }
        Ok(())
    }

    fn require_target_reads(
        &self,
        authorizations: &[ToolExecutionAuthorization],
        roots: &[PathBuf],
    ) -> Result<(), GitToolError> {
        // Check the entire frontier before re-reading any retained file body.
        for target in self.targets() {
            if !self.native_pull_target_read(target, authorizations, roots)
                && !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        &self.tool,
                        ToolNature::Read,
                        &self.workspace,
                        target.to_str().unwrap_or_default(),
                    )
                })
            {
                return Err(invalid());
            }
        }
        Ok(())
    }

    /// Only frozen, unredirected sources may reuse caller-revalidated parent or
    /// module roots. A Path token alone never grants descendant content reads.
    pub(super) fn native_pull_target_read(
        &self,
        target: &Path,
        authorizations: &[ToolExecutionAuthorization],
        roots: &[PathBuf],
    ) -> bool {
        self.tool == "git_pull"
            && authorizations
                .iter()
                .any(|token| token.authorizes_git_pull_bounded_discovery())
            && self.authorizes_anchor(authorizations)
            && roots.iter().any(|root| {
                target.starts_with(root)
                    && (root.starts_with(&self.anchor)
                        || authorizations.iter().any(|token| {
                            token.authorizes_path(
                                &self.tool,
                                ToolNature::Read,
                                &self.workspace,
                                root.to_str().unwrap_or_default(),
                            )
                        }))
            })
            && self.sources.iter().any(|source| {
                source.resolution.resolved() == target
                    && source.resolution.is_unredirected(&source.spelling)
            })
    }

    fn authorizes_anchor(&self, authorizations: &[ToolExecutionAuthorization]) -> bool {
        authorizations.iter().any(|token| {
            token.authorizes_path(
                &self.tool,
                ToolNature::Read,
                &self.workspace,
                self.anchor.to_str().unwrap_or_default(),
            )
        })
    }

    fn revalidate_resolutions_inner(&self) -> Result<(), GitToolError> {
        for source in &self.sources {
            source.resolution.revalidate()?;
        }
        Ok(())
    }
}

fn invalid() -> GitToolError {
    GitToolError::InvalidInput(
        "Git auxiliary path identity changed, lacks exact Read admission, or exceeded its limit"
            .into(),
    )
}

fn unsupported_hook(path: &Path) -> GitToolError {
    GitToolError::InvalidInput(format!(
        "Configured Git {} hook execution is not supported by this plan",
        path.file_name().unwrap_or_default().to_string_lossy()
    ))
}

#[cfg(test)]
#[path = "auxiliary_paths_tests.rs"]
mod tests;
