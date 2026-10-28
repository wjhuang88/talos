//! Invocation-only native Git command capabilities and conservative policy compatibility.
//!
//! Known paths are evidence, not confinement. An active restriction on an
//! undiscovered transitive effect cannot be satisfied by checking known paths.

use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};

use crate::{PermissionDecision, PermissionDecisionSource, PermissionEngine, PermissionRuleSource};

pub(crate) const CAPABILITY: &str = "git:pull-command";

pub(crate) fn command_tool(capability: &str) -> Option<&'static str> {
    match capability {
        CAPABILITY => Some("git_pull"),
        "git:add-command" => Some("git_add"),
        "git:commit-command" => Some("git_commit"),
        "git:checkout-command" => Some("git_checkout"),
        "git:push-command" => Some("git_push"),
        _ => None,
    }
}

pub(crate) fn requested(facets: &[ToolPermissionFacet]) -> bool {
    facets
        .iter()
        .any(|facet| facet.resource.as_deref().and_then(command_tool).is_some())
}

impl PermissionEngine {
    pub(crate) fn ordinary_transitive_effect_restriction(
        &self,
    ) -> Option<(PermissionDecision, PermissionDecisionSource)> {
        for (rule, metadata) in self.rules.iter().zip(&self.rule_metadata) {
            if rule.nature == Some(ToolNature::Internal)
                || metadata.source != PermissionRuleSource::Explicit
                || rule.decision != PermissionDecision::Ask
            {
                continue;
            }
            if rule.resource.is_some() || rule.path_pattern.is_some() {
                return Some((
                    PermissionDecision::Deny(
                        "Invocation authority cannot enforce an active scoped human checkpoint on transitive effects".into(),
                    ),
                    PermissionDecisionSource::Rule {
                        rule_id: metadata.id,
                        rule_source: metadata.source,
                    },
                ));
            }
        }
        self.transitive_effect_restriction()
    }

    pub(crate) fn native_pull_command_restriction(
        &self,
        tool_name: &str,
        facet: &ToolPermissionFacet,
    ) -> Option<(PermissionDecision, PermissionDecisionSource)> {
        if facet.resource.as_deref().and_then(command_tool) != Some(tool_name)
            || facet.nature != ToolNature::Execute
            || facet.resource_kind != Some(ToolResourceKind::Remote)
        {
            return None;
        }
        if tool_name == "git_pull" {
            self.transitive_effect_restriction()
        } else {
            self.ordinary_transitive_effect_restriction()
        }
    }

    pub(crate) fn transitive_effect_restriction(
        &self,
    ) -> Option<(PermissionDecision, PermissionDecisionSource)> {
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
                            "Invocation authority cannot enforce an active restriction on transitive effects".into(),
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
