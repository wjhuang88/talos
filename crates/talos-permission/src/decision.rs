//! Structured permission requests and redaction-safe decision reports.

use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};

use crate::PermissionDecision;

/// Runtime mode recorded with a permission evaluation.
///
/// I189 treats this as diagnostic context only. Approval routing and
/// mode-specific resolution remain composition-root responsibilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// A read-only composition root.
    ReadOnly,
    /// An interactive composition root.
    Interactive,
    /// A composition root without a human approval surface.
    Headless,
    /// An interactive trusted-workspace composition root.
    TrustedWorkspace,
    /// Model-assisted evaluation mode governed by ADR-064.
    Auto,
}

/// Whether a composition root can ask a human to resolve an `Ask` decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InteractionCapability {
    /// A human approval surface is available.
    Available,
    /// No human approval surface is available.
    Unavailable,
}

/// Non-authoritative execution context for one permission evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionContext {
    mode: PermissionMode,
    interaction: InteractionCapability,
}

impl PermissionContext {
    /// Creates an explicit evaluation context.
    #[must_use]
    pub const fn new(mode: PermissionMode, interaction: InteractionCapability) -> Self {
        Self { mode, interaction }
    }

    /// Returns the recorded runtime mode.
    #[must_use]
    pub const fn mode(&self) -> PermissionMode {
        self.mode
    }

    /// Returns the recorded interaction capability.
    #[must_use]
    pub const fn interaction(&self) -> InteractionCapability {
        self.interaction
    }

    /// Returns the context used by legacy evaluation entrypoints.
    #[must_use]
    pub const fn compatibility() -> Self {
        Self::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        )
    }
}

impl Default for PermissionContext {
    fn default() -> Self {
        Self::compatibility()
    }
}

/// A structured permission request.
///
/// Raw input, facets and provenance are evaluator-only data. The request is
/// intentionally not serializable, and its `Debug` implementation is redacted.
pub struct PermissionRequest<'a> {
    tool_name: &'a str,
    provenance: ToolProvenance,
    facets: &'a [ToolPermissionFacet],
    input: &'a Value,
}

impl<'a> PermissionRequest<'a> {
    /// Creates a structured request from composition-root data.
    #[must_use]
    pub fn new(
        tool_name: &'a str,
        provenance: ToolProvenance,
        facets: &'a [ToolPermissionFacet],
        input: &'a Value,
    ) -> Self {
        Self {
            tool_name,
            provenance,
            facets,
            input,
        }
    }

    /// Creates the native-tool request used by compatibility entrypoints.
    #[must_use]
    pub fn native(tool_name: &'a str, facets: &'a [ToolPermissionFacet], input: &'a Value) -> Self {
        Self::new(tool_name, ToolProvenance::Native, facets, input)
    }

    /// Returns the rule-matching tool name.
    #[must_use]
    pub const fn tool_name(&self) -> &str {
        self.tool_name
    }

    /// Returns the supplied tool provenance.
    #[must_use]
    pub const fn provenance(&self) -> &ToolProvenance {
        &self.provenance
    }

    /// Returns the declared facets.
    #[must_use]
    pub const fn facets(&self) -> &[ToolPermissionFacet] {
        self.facets
    }

    pub(crate) const fn input(&self) -> &Value {
        self.input
    }
}

impl fmt::Debug for PermissionRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PermissionRequest")
            .field("tool_name", &"<redacted>")
            .field("tool_source", &PermissionToolSource::from(&self.provenance))
            .field("facet_count", &self.facets.len())
            .field("input", &"<redacted>")
            .finish()
    }
}

/// Coarse, redaction-safe tool provenance class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionToolSource {
    /// A native in-process tool.
    Native,
    /// A tool supplied by a remote MCP server.
    McpRemote,
    /// A tool supplied by a plugin package.
    Plugin,
}

impl From<&ToolProvenance> for PermissionToolSource {
    fn from(value: &ToolProvenance) -> Self {
        match value {
            ToolProvenance::Native => Self::Native,
            ToolProvenance::McpRemote { .. } => Self::McpRemote,
            ToolProvenance::Plugin { .. } => Self::Plugin,
        }
    }
}

/// Stable decision class without a free-form denial message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionOutcome {
    /// The operation may proceed.
    Allow,
    /// A human decision is required.
    Ask,
    /// The operation is denied.
    Deny,
}

impl From<&PermissionDecision> for PermissionOutcome {
    fn from(value: &PermissionDecision) -> Self {
        match value {
            PermissionDecision::Allow => Self::Allow,
            PermissionDecision::Ask => Self::Ask,
            PermissionDecision::Deny(_) => Self::Deny,
        }
    }
}

/// Opaque identifier for one engine-local rule entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct PermissionRuleId(pub(crate) u64);

impl PermissionRuleId {
    /// Returns the opaque engine-local numeric value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Provenance assigned by the rule insertion path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionRuleSource {
    /// A built-in default rule installed by `PermissionEngine::new`.
    Default,
    /// A rule loaded from serialized configuration.
    Configured,
    /// A session/runtime grant inserted through `add_runtime_allow_rule`.
    RuntimeGrant,
    /// A rule supplied directly through the Rust API.
    Explicit,
}

/// Whether a concrete resource was available to the evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionResourceState {
    /// Current behavior does not require a resource for this facet.
    NotRequired,
    /// A concrete resource was supplied or extracted.
    Present,
    /// A consequential facet had no usable concrete resource.
    MissingOrInvalid,
}

/// Closed, redaction-safe explanation for one facet outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionReason {
    /// A matched rule explicitly allowed the facet.
    RuleAllow,
    /// A matched rule requires human approval.
    RuleAsk,
    /// A matched rule explicitly denied the facet.
    RuleDeny,
    /// Trusted-workspace handling allowed a repo-contained write.
    TrustedWorkspaceWrite,
    /// A concrete path was outside the configured workspace boundary.
    ExternalPathRequiresApproval,
    /// A consequential facet had no usable concrete resource.
    MissingOrInvalidResource,
    /// Nature fallback allowed a read or internal facet.
    DefaultAllow,
    /// Nature fallback requires approval for a consequential facet.
    DefaultAsk,
}

impl PermissionReason {
    /// Returns a static, redaction-safe explanation suitable for diagnostics.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::RuleAllow => "matched rule allows this facet",
            Self::RuleAsk => "matched rule requires approval",
            Self::RuleDeny => "matched rule denies this facet",
            Self::TrustedWorkspaceWrite => "trusted workspace allows this contained write",
            Self::ExternalPathRequiresApproval => {
                "path is outside the configured workspace and requires approval"
            }
            Self::MissingOrInvalidResource => "consequential facet has no usable concrete resource",
            Self::DefaultAllow => "default nature behavior allows this facet",
            Self::DefaultAsk => "default nature behavior requires approval",
        }
    }
}

/// Redaction-safe source of a facet decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PermissionDecisionSource {
    /// One ordered rule matched.
    Rule {
        /// Opaque engine-local rule identifier.
        rule_id: PermissionRuleId,
        /// Source assigned by the rule insertion path.
        rule_source: PermissionRuleSource,
    },
    /// Trusted-workspace repo-contained write handling applied.
    WorkspaceTrust,
    /// The external concrete-path boundary required approval.
    WorkspaceBoundary,
    /// No rule matched and nature fallback behavior applied.
    DefaultBehavior,
}

/// One redaction-safe per-facet result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PermissionFacetDecision {
    facet_index: usize,
    nature: ToolNature,
    resource_kind: Option<ToolResourceKind>,
    resource_state: PermissionResourceState,
    outcome: PermissionOutcome,
    reason: PermissionReason,
    source: PermissionDecisionSource,
}

impl PermissionFacetDecision {
    pub(crate) fn new(
        facet_index: usize,
        facet: &ToolPermissionFacet,
        resource_state: PermissionResourceState,
        decision: &PermissionDecision,
        source: PermissionDecisionSource,
    ) -> Self {
        let outcome = PermissionOutcome::from(decision);
        let reason = permission_reason(resource_state, outcome, &source);
        Self {
            facet_index,
            nature: facet.nature,
            resource_kind: facet.resource_kind,
            resource_state,
            outcome,
            reason,
            source,
        }
    }

    /// Returns the request-order facet index.
    #[must_use]
    pub const fn facet_index(&self) -> usize {
        self.facet_index
    }

    /// Returns the facet nature.
    #[must_use]
    pub const fn nature(&self) -> ToolNature {
        self.nature
    }

    /// Returns the resource kind without its value.
    #[must_use]
    pub const fn resource_kind(&self) -> Option<ToolResourceKind> {
        self.resource_kind
    }

    /// Returns whether a usable concrete resource was present.
    #[must_use]
    pub const fn resource_state(&self) -> PermissionResourceState {
        self.resource_state
    }

    /// Returns the redaction-safe decision class.
    #[must_use]
    pub const fn outcome(&self) -> PermissionOutcome {
        self.outcome
    }

    /// Returns the closed, redaction-safe reason code.
    #[must_use]
    pub const fn reason(&self) -> PermissionReason {
        self.reason
    }

    /// Returns the redaction-safe decision source.
    #[must_use]
    pub const fn source(&self) -> &PermissionDecisionSource {
        &self.source
    }
}

fn permission_reason(
    resource_state: PermissionResourceState,
    outcome: PermissionOutcome,
    source: &PermissionDecisionSource,
) -> PermissionReason {
    if resource_state == PermissionResourceState::MissingOrInvalid
        && outcome == PermissionOutcome::Ask
        && matches!(
            source,
            PermissionDecisionSource::DefaultBehavior
                | PermissionDecisionSource::Rule {
                    rule_source: PermissionRuleSource::Default,
                    ..
                }
        )
    {
        return PermissionReason::MissingOrInvalidResource;
    }

    match source {
        PermissionDecisionSource::Rule { .. } => match outcome {
            PermissionOutcome::Allow => PermissionReason::RuleAllow,
            PermissionOutcome::Ask => PermissionReason::RuleAsk,
            PermissionOutcome::Deny => PermissionReason::RuleDeny,
        },
        PermissionDecisionSource::WorkspaceTrust => PermissionReason::TrustedWorkspaceWrite,
        PermissionDecisionSource::WorkspaceBoundary => {
            PermissionReason::ExternalPathRequiresApproval
        }
        PermissionDecisionSource::DefaultBehavior => match outcome {
            PermissionOutcome::Allow => PermissionReason::DefaultAllow,
            PermissionOutcome::Ask | PermissionOutcome::Deny => PermissionReason::DefaultAsk,
        },
    }
}

/// Authoritative structured result for one permission request.
///
/// Serialization and `Debug` intentionally exclude the compatibility denial
/// message, raw input, concrete resources, tool name and provenance names.
#[derive(Clone, Serialize, JsonSchema)]
pub struct PermissionDecisionReport {
    outcome: PermissionOutcome,
    mode: PermissionMode,
    interaction: InteractionCapability,
    tool_source: PermissionToolSource,
    workspace_configured: bool,
    trusted_workspace: bool,
    facets: Vec<PermissionFacetDecision>,
    #[serde(skip)]
    #[schemars(skip)]
    compatibility_decision: PermissionDecision,
}

impl PermissionDecisionReport {
    pub(crate) fn new(
        decision: PermissionDecision,
        context: &PermissionContext,
        provenance: &ToolProvenance,
        workspace_configured: bool,
        trusted_workspace: bool,
        facets: Vec<PermissionFacetDecision>,
    ) -> Self {
        Self {
            outcome: PermissionOutcome::from(&decision),
            mode: context.mode(),
            interaction: context.interaction(),
            tool_source: PermissionToolSource::from(provenance),
            workspace_configured,
            trusted_workspace,
            facets,
            compatibility_decision: decision,
        }
    }

    /// Returns the aggregate redaction-safe outcome.
    #[must_use]
    pub const fn outcome(&self) -> PermissionOutcome {
        self.outcome
    }

    /// Returns the recorded runtime mode.
    #[must_use]
    pub const fn mode(&self) -> PermissionMode {
        self.mode
    }

    /// Returns the recorded interaction capability.
    #[must_use]
    pub const fn interaction(&self) -> InteractionCapability {
        self.interaction
    }

    /// Returns the redacted tool provenance class.
    #[must_use]
    pub const fn tool_source(&self) -> PermissionToolSource {
        self.tool_source
    }

    /// Returns whether the engine had a workspace root.
    #[must_use]
    pub const fn workspace_configured(&self) -> bool {
        self.workspace_configured
    }

    /// Returns whether trusted-workspace behavior was enabled.
    #[must_use]
    pub const fn trusted_workspace(&self) -> bool {
        self.trusted_workspace
    }

    /// Returns all per-facet reports in request order.
    #[must_use]
    pub fn facets(&self) -> &[PermissionFacetDecision] {
        &self.facets
    }

    /// Projects the report to the existing compatibility decision.
    #[must_use]
    pub fn decision(&self) -> PermissionDecision {
        self.compatibility_decision.clone()
    }
}

impl fmt::Debug for PermissionDecisionReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PermissionDecisionReport")
            .field("outcome", &self.outcome)
            .field("mode", &self.mode)
            .field("interaction", &self.interaction)
            .field("tool_source", &self.tool_source)
            .field("workspace_configured", &self.workspace_configured)
            .field("trusted_workspace", &self.trusted_workspace)
            .field("facets", &self.facets)
            .finish()
    }
}

impl From<&PermissionDecisionReport> for PermissionDecision {
    fn from(value: &PermissionDecisionReport) -> Self {
        value.decision()
    }
}
