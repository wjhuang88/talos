# ADR-073: Bundle Manifest Migration Contract

## Status

Accepted

Accepted by maintainer authorization on 2026-09-11. Implementation remains separately governed by BUNDLE-001.

## Context

ADR-072 separates installable Bundles from executable Plugins, but BUNDLE-001 still needs a
concrete compatibility contract before changing persisted manifests. Existing manifests use
`PluginManifest`, `[plugin]`, package roots, and legacy cache identities.

## Decision

The migration is staged and reversible:

1. **Dual-read first.** Readers accept the supported legacy Plugin shape and the versioned Bundle
   shape, but reject duplicate roots, malformed versions, invalid identities, and unsupported
   carriers. No reader infers target fields from unknown data.
2. **Controlled-write only.** Writers emit Bundle vocabulary only when an explicit schema version
   and migration flag are present. Legacy files remain readable and are never rewritten merely by
   loading them.
3. **Unknown-field preservation is mandatory.** A migration must retain unknown top-level tables,
   nested fields, and contribution data byte-for-byte or reject the write before changing the
   source. Silent loss is not allowed.
4. **Identity is stable.** Bundle identity derives from normalized name/version plus digest where
   applicable. Collisions, mismatched digests, and incompatible versions fail closed.
5. **Installation is separate.** Manifest migration, installation, Plugin activation, Provider
   registration, and permission grants are distinct operations. A successful migration never
   executes or activates an artifact.
6. **Rollback is metadata-only.** On failed or interrupted migration, remove only newly written
   target metadata and retain the original manifest, cache entry, and inactive artifact.

## Compatibility matrix

| Legacy surface | Target surface | Required behavior |
|---|---|---|
| `PluginManifest` / `PluginMetadata` | `BundleManifest` / `BundleMetadata` | Dual-read; controlled-write after explicit version gate; preserve unknown fields. |
| `[plugin]` root | `[bundle]` root | Accept exactly one root; reject duplicates; never merge roots. |
| name + version | Bundle identity | Normalize deterministically; reject collisions and invalid semver. |
| Plugin package/cache key | Bundle installation key | Read legacy first, then target; write target only after verified migration. |
| top-level skills/tools/hooks | Typed contributions | Preserve legacy values; registration remains opt-in and separate from disclosure. |
| artifact path | Nested Plugin artifact | Permit only safe relative paths and supported carriers; never execute during migration. |

## Consequences and rollback

Existing manifests remain valid during the dual-read window. A future schema implementation must
provide compatibility fixtures for every matrix row and prove that corrupt, partial, unknown, or
incompatible input leaves the prior bytes and runtime state unchanged. The migration flag and
schema version are the change-control boundary; no online download, activation, permission grant,
release, or Desktop behavior is authorized by this ADR.

## Required follow-up

BUNDLE-001 must establish an effective implementation claim and a separate security/API review
before modifying persisted schema or installation behavior. DIST-001-A remains blocked until that
implementation is complete.
