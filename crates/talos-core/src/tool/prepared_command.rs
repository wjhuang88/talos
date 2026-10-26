//! Tool-owned command review evidence, separate from caller arguments and authority.

use super::PreparedPlanIdentity;

/// Transitive effects disclosed by a retained command producer.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PreparedCommandEffect {
    /// Starts a host executable.
    Execute,
    /// May communicate with remote endpoints.
    Network,
    /// May modify the worktree.
    WorktreeMutation,
    /// May modify repository metadata.
    MetadataMutation,
    /// May recurse into child repositories.
    Recursion,
    /// May run hooks, filters, helpers or other configured programs.
    ConfiguredPrograms,
}

/// Redacted review material supplied by the implementation retaining the command.
/// This is descriptive evidence, never an executable command or authorization.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct PreparedCommandDescription {
    /// Versioned operation identifier owned by the producer.
    pub operation: String,
    /// Fixed argument words with sensitive parameter values replaced by categories.
    pub arguments: Vec<String>,
    /// Complete effect classes, including transitive configured-program effects.
    pub effects: Vec<PreparedCommandEffect>,
    /// Whether authorization promises path and endpoint confinement.
    pub confined: bool,
    /// Known target facts, without claiming exhaustive effect discovery.
    pub targets: PreparedCommandTargets,
}

/// Bounded, redacted target facts captured by a trusted command producer.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct PreparedCommandTargets {
    /// Worktree label relative to the registered workspace, or an outside marker.
    pub repository: String,
    /// Known metadata roots, relative to the workspace or marked outside it.
    pub metadata: Vec<String>,
    /// Whether the invocation retains upstream selection or an explicit remote.
    pub remote_selection: String,
    /// Sanitized configured hosts; not necessarily the selected destination.
    pub configured_remote_hosts: Vec<String>,
    /// Sanitized host from an explicit invocation URL, when safely available.
    pub invocation_remote_host: Option<String>,
    /// Known configured-program categories, never command values.
    pub configured_program_categories: Vec<String>,
    /// Dynamically selected destinations, paths and programs remain unknown.
    pub dynamic_effects_unknown: bool,
}

/// In-memory evidence tied to the exact retained preparation.
/// The identity is deliberately not serializable: displayed text, JSON and
/// digests of configuration values cannot recreate this binding.
#[derive(Debug, Clone)]
pub struct PreparedCommandEvidence {
    identity: PreparedPlanIdentity,
    description: PreparedCommandDescription,
}

impl PreparedCommandEvidence {
    /// Binds implementation-produced review material to its retained plan.
    /// Producers must disclose effects truthfully and omit secrets and raw
    /// configuration/environment values. This grants no execution authority.
    pub fn new(identity: PreparedPlanIdentity, description: PreparedCommandDescription) -> Self {
        Self {
            identity,
            description,
        }
    }

    /// Tests binding to the preparation about to enter the permission pipeline.
    pub fn belongs_to(&self, identity: &PreparedPlanIdentity) -> bool {
        self.identity.same_plan(identity)
    }

    /// Returns the producer's redacted review material.
    pub fn description(&self) -> &PreparedCommandDescription {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copied_description_cannot_rebind_retained_evidence() {
        let plan = PreparedPlanIdentity::new();
        let evidence = PreparedCommandEvidence::new(
            plan.clone(),
            PreparedCommandDescription {
                operation: "test-command-v1".into(),
                arguments: vec!["<withheld>".into()],
                effects: vec![PreparedCommandEffect::Execute],
                confined: false,
                targets: PreparedCommandTargets {
                    repository: ".".into(),
                    metadata: vec![".git".into()],
                    remote_selection: "configured upstream".into(),
                    configured_remote_hosts: vec![],
                    invocation_remote_host: None,
                    configured_program_categories: vec![],
                    dynamic_effects_unknown: true,
                },
            },
        );
        assert!(evidence.clone().belongs_to(&plan));
        assert!(!evidence.belongs_to(&PreparedPlanIdentity::new()));
        let serialized = serde_json::to_value(evidence.description()).expect("review projection");
        assert!(serialized.get("identity").is_none());
    }
}
