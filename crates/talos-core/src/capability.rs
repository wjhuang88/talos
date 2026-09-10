//! UI-neutral capability descriptors and offline resolution (CAP-001-A/B).

use std::collections::BTreeMap;
use std::time::Instant;

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
        validate_origin(self.provenance, self.carrier)?;
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
    /// Legacy descriptor without origin information; never valid for use.
    #[default]
    Unknown,
    /// Explicitly declared built-in implementation; not a trust attestation.
    BuiltIn,
    /// Loadable plugin; verification belongs to its loader.
    Plugin,
    /// External implementation; authorization remains a host responsibility.
    External,
}

/// Delivery carrier classification.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub enum Carrier {
    /// Legacy descriptor without carrier information; never valid for use.
    #[default]
    Unknown,
    /// In-process built-in implementation.
    BuiltIn,
    /// WebAssembly implementation.
    Wasm,
    /// Model Context Protocol connection.
    Mcp,
    /// Helper process.
    Helper,
    /// Remote connector.
    Remote,
}

impl ProviderDescriptor {
    /// Validate this provider and each nested capability descriptor.
    pub fn validate(&self) -> Result<(), DescriptorError> {
        validate_id(&self.id)?;
        validate_origin(self.provenance, self.carrier)?;
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

/// A deterministic, UI-neutral registry of host-declared providers.
#[derive(Clone, Debug, Default)]
pub struct CapabilityRegistry {
    providers: BTreeMap<String, ProviderDescriptor>,
}

/// A capability resolution request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRequest {
    /// Stable capability identifier.
    pub capability_id: String,
    /// Required compatible capability contract major version.
    pub version: String,
}

/// The typed result of resolving a capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionResult {
    /// A compatible provider was selected deterministically.
    Available(ProviderDescriptor),
    /// No provider advertises the requested capability.
    Unavailable,
    /// Providers exist, but none match the requested major version.
    Incompatible,
    /// The request or a registered descriptor is invalid.
    Invalid(DescriptorError),
    /// Resolution was cancelled or exceeded its deadline.
    Cancelled,
    /// The host cancellation callback panicked; no provider was selected.
    Error,
}

impl CapabilityRegistry {
    /// Register a validated provider without exposing tools or schemas.
    ///
    /// A valid descriptor replaces the previous descriptor with the same provider ID.
    /// Validation happens before mutation; invalid updates preserve the old record.
    /// Registration order between distinct provider IDs does not affect selection.
    pub fn register(&mut self, provider: ProviderDescriptor) -> Result<(), DescriptorError> {
        provider.validate()?;
        self.providers.insert(provider.id.clone(), provider);
        Ok(())
    }

    /// Resolve a request using deterministic provider-id ordering.
    pub fn resolve(&self, request: &CapabilityRequest) -> ResolutionResult {
        self.resolve_with(request, || false, None)
    }

    /// Resolve while observing cancellation and an optional deadline.
    ///
    /// The callback must return promptly: a synchronous callback cannot be preempted.
    /// Unwinding callback panics return [`ResolutionResult::Error`]; aborting panics
    /// cannot be recovered. A failed callback is not invoked again in this request.
    pub fn resolve_with<F: Fn() -> bool>(
        &self,
        request: &CapabilityRequest,
        is_cancelled: F,
        deadline: Option<Instant>,
    ) -> ResolutionResult {
        let check = || {
            if deadline.is_some_and(|at| Instant::now() >= at) {
                return Err(ResolutionResult::Cancelled);
            }
            // Only the host callback crosses the unwind boundary. On failure we
            // abandon this resolution rather than reusing potentially changed state.
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(&is_cancelled)) {
                Err(_) => Err(ResolutionResult::Error),
                Ok(true) => Err(ResolutionResult::Cancelled),
                Ok(false) if deadline.is_some_and(|at| Instant::now() >= at) => {
                    Err(ResolutionResult::Cancelled)
                }
                Ok(false) => Ok(()),
            }
        };
        if let Err(result) = check() {
            return result;
        }
        if let Err(error) = validate_id(&request.capability_id) {
            return ResolutionResult::Invalid(error);
        }
        let required = match parse_version(&request.version) {
            Ok(version) => version,
            Err(error) => return ResolutionResult::Invalid(error),
        };
        let mut found = false;
        for provider in self.providers.values() {
            if let Err(result) = check() {
                return result;
            }
            for capability in &provider.capabilities {
                if let Err(result) = check() {
                    return result;
                }
                if capability.id != request.capability_id {
                    continue;
                }
                found = true;
                if parse_version(&capability.version)
                    .map(|v| v.0 == required.0)
                    .unwrap_or(false)
                {
                    let selected = provider.clone();
                    if let Err(result) = check() {
                        return result;
                    }
                    return ResolutionResult::Available(selected);
                }
            }
        }
        if let Err(result) = check() {
            return result;
        }
        if found {
            ResolutionResult::Incompatible
        } else {
            ResolutionResult::Unavailable
        }
    }
}

/// Errors produced when validating a descriptor contract.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DescriptorError {
    /// Missing origin information cannot imply built-in identity or authorization.
    #[error("descriptor provenance and carrier must be explicit")]
    UnknownOrigin,
    /// An identifier was absent or malformed.
    #[error("descriptor id must contain only ASCII letters, digits, '.', '-', or '_': {0}")]
    InvalidId(String),
    /// A version was not a three-component numeric version.
    #[error("descriptor version must be major.minor.patch: {0}")]
    InvalidVersion(String),
}

fn validate_origin(provenance: Provenance, carrier: Carrier) -> Result<(), DescriptorError> {
    if provenance == Provenance::Unknown || carrier == Carrier::Unknown {
        return Err(DescriptorError::UnknownOrigin);
    }
    Ok(())
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

    #[test]
    fn legacy_origin_is_readable_but_never_inferred_as_built_in() {
        let legacy = r#"{"id":"provider.legacy","version":"1.0.0"}"#;
        let descriptor: ProviderDescriptor = serde_json::from_str(legacy).unwrap();
        assert_eq!(descriptor.provenance, Provenance::Unknown);
        assert_eq!(descriptor.carrier, Carrier::Unknown);
        assert_eq!(descriptor.validate(), Err(DescriptorError::UnknownOrigin));
        for field in ["carrier", "provenance"] {
            let mut value = serde_json::to_value(provider("1.0.0")).unwrap();
            value.as_object_mut().unwrap().remove(field);
            let decoded: ProviderDescriptor = serde_json::from_value(value).unwrap();
            assert_eq!(decoded.validate(), Err(DescriptorError::UnknownOrigin));
        }
    }

    #[test]
    fn registry_resolves_deterministically_and_fails_closed() {
        let capability = CapabilityDescriptor {
            id: "text.search".into(),
            version: "1.0.0".into(),
            name: "Search".into(),
            provenance: Provenance::BuiltIn,
            carrier: Carrier::BuiltIn,
            metadata: BTreeMap::new(),
        };
        let mut first = provider("1.2.0");
        first.id = "z.provider".into();
        first.capabilities = vec![capability.clone()];
        let mut second = provider("1.3.0");
        second.id = "a.provider".into();
        second.capabilities = vec![capability];
        let mut registry = CapabilityRegistry::default();
        registry.register(first).unwrap();
        registry.register(second).unwrap();
        let request = CapabilityRequest {
            capability_id: "text.search".into(),
            version: "1.0.0".into(),
        };
        assert!(
            matches!(registry.resolve(&request), ResolutionResult::Available(p) if p.id == "a.provider")
        );
        assert_eq!(
            registry.resolve(&CapabilityRequest {
                capability_id: "missing".into(),
                version: "1.0.0".into()
            }),
            ResolutionResult::Unavailable
        );
        assert!(matches!(
            registry.resolve(&CapabilityRequest {
                capability_id: "text.search".into(),
                version: "bad".into()
            }),
            ResolutionResult::Invalid(_)
        ));
    }

    #[test]
    fn registry_matches_capability_version_not_provider_version() {
        let mut descriptor = provider("2.0.0");
        descriptor.capabilities.push(CapabilityDescriptor {
            id: "text.search".into(),
            version: "1.4.0".into(),
            name: "Search".into(),
            provenance: Provenance::BuiltIn,
            carrier: Carrier::BuiltIn,
            metadata: BTreeMap::new(),
        });
        let mut registry = CapabilityRegistry::default();
        registry
            .register(descriptor)
            .expect("valid provider fixture");
        let mut request = CapabilityRequest {
            capability_id: "text.search".into(),
            version: "1.0.0".into(),
        };
        assert!(matches!(
            registry.resolve(&request),
            ResolutionResult::Available(_)
        ));
        request.version = "2.0.0".into();
        assert_eq!(registry.resolve(&request), ResolutionResult::Incompatible);
    }

    #[test]
    fn registry_cancelled_and_expired_requests_do_not_resolve() {
        let registry = CapabilityRegistry::default();
        let request = CapabilityRequest {
            capability_id: "text.search".into(),
            version: "1.0.0".into(),
        };
        assert_eq!(
            registry.resolve_with(&request, || true, None),
            ResolutionResult::Cancelled
        );
        assert_eq!(
            registry.resolve_with(&request, || false, Some(Instant::now())),
            ResolutionResult::Cancelled
        );
    }

    #[test]
    fn registry_callback_panic_returns_error_without_unwinding() {
        let registry = CapabilityRegistry::default();
        let request = CapabilityRequest {
            capability_id: "text.search".into(),
            version: "1.0.0".into(),
        };
        assert_eq!(
            registry.resolve_with(&request, || panic!("host callback failed"), None),
            ResolutionResult::Error
        );
        assert_eq!(registry.resolve(&request), ResolutionResult::Unavailable);
    }

    #[test]
    fn registry_cancellation_during_capability_scan_fails_closed() {
        let mut descriptor = provider("1.0.0");
        descriptor.capabilities = ["first", "target"]
            .map(|id| CapabilityDescriptor {
                id: id.into(),
                version: "1.0.0".into(),
                name: id.into(),
                provenance: Provenance::BuiltIn,
                carrier: Carrier::BuiltIn,
                metadata: BTreeMap::new(),
            })
            .to_vec();
        let mut registry = CapabilityRegistry::default();
        assert!(registry.register(descriptor).is_ok());
        let request = CapabilityRequest {
            capability_id: "target".into(),
            version: "1.0.0".into(),
        };
        // Checks 4 and 5 guard the target capability and the final return,
        // respectively. Neither cancellation point may expose a selection.
        for cancel_at in [4, 5] {
            let calls = std::cell::Cell::new(0);
            assert_eq!(
                registry.resolve_with(
                    &request,
                    || {
                        calls.set(calls.get() + 1);
                        calls.get() >= cancel_at
                    },
                    None
                ),
                ResolutionResult::Cancelled
            );
        }
        let calls = std::cell::Cell::new(0);
        assert_eq!(
            registry.resolve_with(
                &request,
                || {
                    calls.set(calls.get() + 1);
                    assert!(calls.get() < 4, "callback failed during scan");
                    false
                },
                None
            ),
            ResolutionResult::Error
        );
        assert!(matches!(
            registry.resolve(&request),
            ResolutionResult::Available(_)
        ));
    }

    #[test]
    fn registry_rejects_invalid_request_identity() {
        let registry = CapabilityRegistry::default();
        assert!(matches!(
            registry.resolve(&CapabilityRequest {
                capability_id: "invalid id".into(),
                version: "1.0.0".into(),
            }),
            ResolutionResult::Invalid(DescriptorError::InvalidId(_))
        ));
    }

    #[test]
    fn registry_selection_is_independent_of_distinct_provider_registration_order() {
        let make_provider = |id: &str| {
            let mut descriptor = provider("3.0.0");
            descriptor.id = id.into();
            descriptor.capabilities.push(CapabilityDescriptor {
                id: "text.search".into(),
                version: "1.0.0".into(),
                name: "Search".into(),
                provenance: Provenance::BuiltIn,
                carrier: Carrier::BuiltIn,
                metadata: BTreeMap::new(),
            });
            descriptor
        };
        let request = CapabilityRequest {
            capability_id: "text.search".into(),
            version: "1.0.0".into(),
        };
        let mut forward = CapabilityRegistry::default();
        let mut reverse = CapabilityRegistry::default();
        for id in ["a.provider", "z.provider"] {
            assert!(forward.register(make_provider(id)).is_ok());
        }
        for id in ["z.provider", "a.provider"] {
            assert!(reverse.register(make_provider(id)).is_ok());
        }
        assert_eq!(forward.resolve(&request), reverse.resolve(&request));
        assert!(
            matches!(forward.resolve(&request), ResolutionResult::Available(p) if p.id == "a.provider")
        );

        let mut invalid = make_provider("a.provider");
        invalid.capabilities[0].version = "invalid".into();
        assert!(forward.register(invalid).is_err());
        assert_eq!(forward.resolve(&request), reverse.resolve(&request));

        let mut replacement = make_provider("a.provider");
        replacement.version = "4.0.0".into();
        assert!(forward.register(replacement).is_ok());
        assert!(
            matches!(forward.resolve(&request), ResolutionResult::Available(p)
            if p.id == "a.provider" && p.version == "4.0.0")
        );
    }
}
