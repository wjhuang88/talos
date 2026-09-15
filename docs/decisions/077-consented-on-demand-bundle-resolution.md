# ADR-077: Consented On-Demand Bundle Resolution

**Status:** Proposed (DIST-001-B / Issue #515)

## Decision (proposed)

- Capability resolution may discover missing Bundles, but installation requires an explicit user
  consent event tied to the exact Bundle identity, version, source and integrity digest.
- Discovery never downloads executables silently; startup remains network-independent.
- Download uses an allowlisted source policy, bounded size/time, checksum/signature verification and
  atomic quarantine-before-install. Failed verification leaves no activatable artifact.
- Installation and Plugin activation remain separate. Activation requires the existing permission
  and carrier lifecycle gates; no Bundle grants permissions by itself.
- Offline/mirror sources are supported through the same identity and verification contract.
- Cancellation, timeout, unavailable source and malformed manifests fail closed with a typed result.

## Evidence required before acceptance

Consent UX replay; offline/mirror fixture; digest/signature and quarantine tests; cancellation and
timeout tests; proof that startup and existing defaults perform no network access; independent
security review covering executable substitution and permission escalation.

This proposal authorizes no implementation, network behavior, manifest migration or permission
change until accepted and claimed.
