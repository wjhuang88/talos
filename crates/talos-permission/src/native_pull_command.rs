//! Invocation-only native Pull authority and conservative policy compatibility.
//!
//! Known paths are evidence, not confinement. An active restriction on an
//! undiscovered transitive effect cannot be satisfied by checking known paths.

use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};

use crate::{PermissionDecision, PermissionDecisionSource, PermissionEngine, PermissionRuleSource};

pub(crate) const CAPABILITY: &str = "git:pull-command";

pub(crate) fn requested(facets: &[ToolPermissionFacet]) -> bool {
    facets
        .iter()
        .any(|facet| facet.resource.as_deref().map(str::trim) == Some(CAPABILITY))
}

impl PermissionEngine {
    pub(crate) fn native_pull_command_restriction(
        &self,
        tool_name: &str,
        facet: &ToolPermissionFacet,
    ) -> Option<(PermissionDecision, PermissionDecisionSource)> {
        if tool_name != "git_pull"
            || facet.nature != ToolNature::Execute
            || facet.resource_kind != Some(ToolResourceKind::Remote)
            || facet.resource.as_deref() != Some(CAPABILITY)
        {
            return None;
        }
        let mut human_checkpoint = None;
        for (rule, metadata) in self.rules.iter().zip(&self.rule_metadata) {
            // Helpers are arbitrary programs. Without OS confinement, do not
            // infer disjointness from a resource glob or a legacy tool name.
            // Internal rules govern Talos plumbing, not child-process effects.
            if rule.nature == Some(ToolNature::Internal) {
                continue;
            }
            let source = PermissionDecisionSource::Rule {
                rule_id: metadata.id,
                rule_source: metadata.source,
            };
            match &rule.decision {
                PermissionDecision::Deny(_) => {
                    return Some((
                        PermissionDecision::Deny(
                            "Native Pull command authority cannot enforce an active restriction on transitive effects".into(),
                        ),
                        source,
                    ));
                }
                PermissionDecision::Ask if metadata.source == PermissionRuleSource::Explicit => {
                    human_checkpoint.get_or_insert((PermissionDecision::Ask, source));
                }
                PermissionDecision::Allow | PermissionDecision::Ask => {}
            }
        }
        human_checkpoint
    }
}

#[cfg(test)]
#[path = "native_pull_command_tests.rs"]
mod tests;
