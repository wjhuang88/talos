//! UI-neutral capability and provider descriptor contracts (CAP-001-A).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A stable description of a capability exposed by a provider or plugin.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct CapabilityDescriptor {
    /// Globally stable capability identifier.
    pub id: String,
    /// Descriptor contract version (major.minor.patch).
    pub version: String,
    /// Human-readable display name.
    pub name: String,
    /// Implementation provenance.
    #[serde(default)]
    pub provenance: Provenance,
    /// Delivery carrier.
    #[serde(default)]
    pub carrier: Carrier,
    /// Optional provider-specific metadata; not interpreted by the core.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl CapabilityDescriptor {
    /// Validate required fields and the descriptor version shape.
    pub fn validate(&self) -> Result<(), DescriptorError> {
        validate_id(&self.id)?;
        validate_version(&self.version)
    }
}

/// A stable description of a provider and its capabilities.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ProviderDescriptor {
    /// Globally stable provider identifier.
    pub id: String,
    /// Provider implementation version.
    pub version: String,
    /// Implementation provenance.
    #[serde(default)]
    pub provenance: Provenance,
    /// Delivery carrier.
    #[serde(default)]
    pub carrier: Carrier,
    /// Capabilities offered by this provider.
    #[serde(default)]
    pub capabilities: Vec<CapabilityDescriptor>,
    /// Optional provider-specific metadata.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

/// Provenance classification for descriptor consumers.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub enum Provenance {
    #[default]
    BuiltIn,
    Plugin,
    External,
}

/// Delivery carrier classification.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub enum Carrier {
    #[default]
    BuiltIn,
    Wasm,
    Mcp,
    Helper,
    Remote,
}

impl ProviderDescriptor {
    /// Validate this provider and each nested capability descriptor.
    pub fn validate(&self) -> Result<(), DescriptorError> {
        validate_id(&self.id)?;
        validate_version(&self.version)?;
        for capability in &self.capabilities {
            capability.validate()?;
        }
        Ok(())
    }

    /// Return whether this provider's major version is compatible with a requirement.
    pub fn is_compatible_with(&self, required: &str) -> Result<bool, DescriptorError> {
        let actual = parse_version(&self.version)?;
        let required = parse_version(required)?;
        Ok(actual.0 == required.0)
    }
}

/// Errors produced when validating a descriptor contract.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DescriptorError {
    /// An identifier was absent or malformed.
    #[error("descriptor id must contain only ASCII letters, digits, '.', '-', or '_': {0}")]
    InvalidId(String),
    /// A version was not a three-component numeric version.
    #[error("descriptor version must be major.minor.patch: {0}")]
    InvalidVersion(String),
}

fn validate_id(id: &str) -> Result<(), DescriptorError> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
    {
        return Err(DescriptorError::InvalidId(id.to_owned()));
    }
    Ok(())
}

fn validate_version(version: &str) -> Result<(), DescriptorError> {
    parse_version(version).map(|_| ())
}

fn parse_version(version: &str) -> Result<(u64, u64, u64), DescriptorError> {
    let mut parts = version.split('.');
    let parsed = (parts.next(), parts.next(), parts.next(), parts.next());
    match parsed {
        (Some(a), Some(b), Some(c), None) => match (a.parse(), b.parse(), c.parse()) {
            (Ok(a), Ok(b), Ok(c)) => Ok((a, b, c)),
            _ => Err(DescriptorError::InvalidVersion(version.to_owned())),
        },
        _ => Err(DescriptorError::InvalidVersion(version.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(version: &str) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "provider.test".into(),
            version: version.into(),
            capabilities: vec![],
            provenance: Provenance::BuiltIn,
            carrier: Carrier::BuiltIn,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn validates_nested_descriptors_and_major_compatibility() {
        let descriptor = provider("1.2.3");
        assert!(descriptor.validate().is_ok());
        assert!(descriptor.is_compatible_with("1.9.0").unwrap());
        assert!(!descriptor.is_compatible_with("2.0.0").unwrap());
    }

    #[test]
    fn rejects_malformed_ids_and_versions() {
        let mut descriptor = provider("1.2");
        assert!(matches!(
            descriptor.validate(),
            Err(DescriptorError::InvalidVersion(_))
        ));
        descriptor.version = "1.2.3".into();
        descriptor.id = "bad id".into();
        assert!(matches!(
            descriptor.validate(),
            Err(DescriptorError::InvalidId(_))
        ));
    }

    #[test]
    fn offline_fixture_has_deterministic_serialization_and_round_trip() {
        let mut descriptor = provider("1.2.3");
        descriptor.metadata.insert("z".into(), "last".into());
        descriptor.metadata.insert("a".into(), "first".into());
        let encoded = serde_json::to_string(&descriptor).unwrap();
        let decoded: ProviderDescriptor = serde_json::from_str(&encoded).unwrap();
        assert_eq!(descriptor, decoded);
        assert!(decoded.validate().is_ok());
        assert_eq!(serde_json::to_string(&decoded).unwrap(), encoded);
        assert!(encoded.find("\"a\"").unwrap() < encoded.find("\"z\"").unwrap());
    }

    #[test]
    fn unknown_carrier_and_provenance_fail_closed() {
        let descriptor = provider("1.2.3");
        let mut value = serde_json::to_value(&descriptor).unwrap();
        value["carrier"] = serde_json::json!("FutureCarrier");
        assert!(serde_json::from_value::<ProviderDescriptor>(value).is_err());
        let mut value = serde_json::to_value(&descriptor).unwrap();
        value["provenance"] = serde_json::json!("FutureProvenance");
        assert!(serde_json::from_value::<ProviderDescriptor>(value).is_err());
    }
}
