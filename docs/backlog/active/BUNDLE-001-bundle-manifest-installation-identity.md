# BUNDLE-001: Bundle Manifest, Installation Identity And Terminology Migration

| Field | Value |
|---|---|
| Story ID | BUNDLE-001 |
| Type | Distribution contract and compatible manifest implementation |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #514](https://github.com/wjhuang88/talos/issues/514) |
| Depends On | ADR-072; existing PluginManifest compatibility evidence; dedicated manifest migration ADR before implementation |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Bundle terminology, manifest shape, installation identity and staged compatible manifest implementation after its migration decision gate. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires a dedicated migration ADR or accepted change-control record before schema edits. |

## Goal And Scope

Separate install/distribution Bundles from executable Plugins. Define package root, cache identity,
contents, integrity metadata, unknown-field handling, rollback and dual-read/controlled-write
migration rules. This owner also owns implementing and testing that compatible schema/manifest
migration after a dedicated migration ADR or accepted change-control record. The present intake
does not authorize schema edits; later iteration selection must include that explicit decision gate.

## Non-Goals

No installer, network download, marketplace, activation, Plugin loader, release or publication.
No ungated breaking manifest change. Historical Plugin terminology remains preserved with explicit mapping.

## Acceptance

- Current PluginManifest/PluginMetadata/package-root uses are inventoried.
- Bundle and Plugin ownership/lifecycles are unambiguous.
- Migration matrix specifies dual-read, controlled-write, unknown fields and rollback.
- A later schema ADR gates any rename; old configs remain readable until that gate is met.
- After that decision, implement Bundle manifest parsing, validation and installation identity,
  with compatibility fixtures for legacy PluginManifest/PluginMetadata and package roots,
  controlled writes, unknown fields and rollback. Planning text alone cannot satisfy completion
  or unblock DIST-001-A.
- Installation never implies activation, registration or permission grant.

## Validation And Documentation

Manifest compatibility fixtures, migration ADR review, YAML/TOML/schema validation, architecture
and user-facing terminology documentation. No Cargo or runtime behavior is authorized by this
intake; implementation requires its own selected iteration and effective claim.
