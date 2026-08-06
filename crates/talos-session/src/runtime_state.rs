//! Durable Session-owned runtime activation identity.
//!
//! The transcript remains the model-visible audit trail, but it is subject to
//! normal compaction. This sidecar state is the machine authority used to
//! reconstruct the exact provider/model/variant and generation independently of
//! the active transcript retention window.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Exact declarative model identity owned by one durable Session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRuntimeIdentity {
    pub provider: String,
    pub model: String,
    pub variant: Option<String>,
}

impl SessionRuntimeIdentity {
    #[must_use]
    pub fn new(provider: &str, model: &str, variant: Option<&str>) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            variant: normalize_variant_id(variant).map(str::to_string),
        }
    }

    #[must_use]
    pub fn display_name(&self) -> String {
        match self.variant.as_deref() {
            Some(variant) => format!("{}/{}@{variant}", self.provider, self.model),
            None => format!("{}/{}", self.provider, self.model),
        }
    }
}

/// Immutable logical activation identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRuntimeActivation {
    pub version: u8,
    pub activation_id: String,
    pub generation: u64,
    pub previous: SessionRuntimeIdentity,
    pub target: SessionRuntimeIdentity,
}

impl SessionRuntimeActivation {
    #[must_use]
    pub fn new(
        generation: u64,
        previous: SessionRuntimeIdentity,
        target: SessionRuntimeIdentity,
    ) -> Self {
        let canonical = serde_json::to_vec(&(generation, &previous, &target))
            .expect("runtime activation identity contains only serializable values");
        let digest = Sha256::digest(canonical);
        let suffix: String = digest
            .iter()
            .take(16)
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self {
            version: 1,
            activation_id: format!("model-activation-g{generation}-{suffix}"),
            generation,
            previous,
            target,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.version == 1
            && self.activation_id
                == Self::new(self.generation, self.previous.clone(), self.target.clone())
                    .activation_id
    }
}

/// Whether a staged activation has crossed the transcript marker barrier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionRuntimeActivationStatus {
    PendingMarker,
    Committed,
}

impl SessionRuntimeActivationStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::PendingMarker => "pending_marker",
            Self::Committed => "committed",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "pending_marker" => Some(Self::PendingMarker),
            "committed" => Some(Self::Committed),
            _ => None,
        }
    }
}

/// Current durable Session runtime state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRuntimeState {
    pub activation: SessionRuntimeActivation,
    pub status: SessionRuntimeActivationStatus,
}

fn normalize_variant_id(variant: Option<&str>) -> Option<&str> {
    let variant = variant.map(str::trim).filter(|value| !value.is_empty())?;
    (!variant.eq_ignore_ascii_case("default")).then_some(variant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_spellings_share_one_identity() {
        assert_eq!(
            SessionRuntimeIdentity::new("openai", "o3", None),
            SessionRuntimeIdentity::new("openai", "o3", Some(" DEFAULT "))
        );
    }

    #[test]
    fn activation_validation_detects_tampering() {
        let mut activation = SessionRuntimeActivation::new(
            7,
            SessionRuntimeIdentity::new("openai", "o3", None),
            SessionRuntimeIdentity::new("openai", "o3", Some("high-reasoning")),
        );
        assert!(activation.is_valid());
        activation.target.variant = Some("low-reasoning".to_string());
        assert!(!activation.is_valid());
    }
}
