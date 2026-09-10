//! Exclusive lifecycle ownership must never remove a replacement or another owner.
use talos_core::capability::{Carrier, Provenance, ProviderRegistrationError};
use talos_core::{
    CapabilityDescriptor, CapabilityRegistry, CapabilityRequest, ProviderDescriptor,
    ResolutionResult,
};

fn provider(id: &str) -> ProviderDescriptor {
    ProviderDescriptor {
        id: id.into(),
        version: "1.0.0".into(),
        provenance: Provenance::Plugin,
        carrier: Carrier::Wasm,
        metadata: Default::default(),
        capabilities: vec![CapabilityDescriptor {
            id: id.into(),
            version: "1.0.0".into(),
            name: id.into(),
            provenance: Provenance::Plugin,
            carrier: Carrier::Wasm,
            metadata: Default::default(),
        }],
    }
}

fn resolve(registry: &CapabilityRegistry, id: &str) -> ResolutionResult {
    registry.resolve(&CapabilityRequest {
        capability_id: id.into(),
        version: "1.0.0".into(),
    })
}

#[test]
fn collision_and_invalid_batch_leave_no_partial_registration() {
    let mut registry = CapabilityRegistry::default();
    let lease = registry
        .register_owned(vec![provider("existing")])
        .expect("first owner");
    assert_eq!(
        registry
            .register_owned(vec![provider("new"), provider("existing")])
            .unwrap_err(),
        ProviderRegistrationError::Occupied("existing".into())
    );
    assert_eq!(resolve(&registry, "new"), ResolutionResult::Unavailable);
    let mut invalid = provider("invalid");
    invalid.version = "bad".into();
    assert!(
        registry
            .register_owned(vec![provider("new"), invalid])
            .is_err()
    );
    assert_eq!(resolve(&registry, "new"), ResolutionResult::Unavailable);
    assert!(
        registry
            .register_owned(vec![provider("new"), provider("new")])
            .is_err()
    );
    assert_eq!(resolve(&registry, "new"), ResolutionResult::Unavailable);
    registry.withdraw(&lease);
    assert_eq!(
        resolve(&registry, "existing"),
        ResolutionResult::Unavailable
    );
}

#[test]
fn stale_lease_preserves_replacement_and_unrelated_owners() {
    let mut registry = CapabilityRegistry::default();
    let old = registry
        .register_owned(vec![provider("one")])
        .expect("first owner");
    let other = registry
        .register_owned(vec![provider("two")])
        .expect("second owner");
    registry
        .register(provider("one"))
        .expect("legacy host replacement");
    registry.withdraw(&old);
    registry.withdraw(&old);
    assert!(matches!(
        resolve(&registry, "one"),
        ResolutionResult::Available(_)
    ));
    assert!(matches!(
        resolve(&registry, "two"),
        ResolutionResult::Available(_)
    ));
    registry.withdraw(&other);
    let new = registry
        .register_owned(vec![provider("two")])
        .expect("new generation");
    registry.withdraw(&other);
    assert!(matches!(
        resolve(&registry, "two"),
        ResolutionResult::Available(_)
    ));
    registry.withdraw(&new);
    assert_eq!(resolve(&registry, "two"), ResolutionResult::Unavailable);
}

#[test]
fn registry_snapshot_cannot_confuse_later_registrations() {
    let mut original = CapabilityRegistry::default();
    let lease = original
        .register_owned(vec![provider("one")])
        .expect("owner");
    let mut snapshot = original.clone();
    original.withdraw(&lease);
    assert!(matches!(
        resolve(&snapshot, "one"),
        ResolutionResult::Available(_)
    ));
    snapshot
        .register(provider("one"))
        .expect("replace snapshot");
    snapshot.withdraw(&lease);
    assert!(matches!(
        resolve(&snapshot, "one"),
        ResolutionResult::Available(_)
    ));
    assert_eq!(resolve(&original, "one"), ResolutionResult::Unavailable);
}
