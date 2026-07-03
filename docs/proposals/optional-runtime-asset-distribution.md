# Optional Runtime Asset Distribution

> Status: ADR-ready proposal from I091 A8.
> Scope: policy only; no runtime downloader, registry client, marketplace, or package installer is
> implemented by this proposal.

## Problem

Talos needs a way to keep the default release small while supporting future optional assets such as
local micro-model weights, WASM plugin packages, tree-sitter language resources, and non-executable
capability packs. Shipping every asset in the default binary would make installation heavy, but
runtime downloads create security, reproducibility, offline, and consent risks.

## Proposed Approach

Use an explicit, verified optional asset model. Talos may learn that an optional asset exists from a
local manifest, a Talos release manifest, or a configured mirror, but it must not download, install,
execute, or activate that asset without an explicit user action or an explicit policy-owned prompt.

The first implementation slice must support manual install only. Missing optional assets degrade to
the current behavior with a diagnostic and, where appropriate, a command the user can run later.
Automatic prompts and third-party registries require a follow-up ADR and a separate iteration.

## Asset Classes

| Class | Examples | Execution posture | Verification floor |
|---|---|---|---|
| Model weights | local router, title, or compaction helper weights | Data only; never permission authority | Manifest checksum plus model/runtime compatibility |
| WASM plugin package | plugin manifest, `.wasm`, bundled skills/resources | Executable only after plugin loader, provenance, sandbox, and permission gates pass | Manifest checksum, artifact checksum, signature when available |
| Resource pack | templates, parser data, language resources, examples | Non-executable data | Manifest checksum and declared type validation |

Dynamic libraries, native binary hooks, Node/Python runtimes, and standalone executable hook carriers
are not valid optional asset classes under this policy.

## Manifest Policy

Every optional asset is described by a versioned manifest. The manifest must be data-only and small
enough to inspect before downloading large payloads.

Required fields:

- stable asset id, display name, class, and publisher/source identity;
- asset version and compatible Talos version/protocol range;
- payload URLs or mirror keys;
- byte size and media type for every payload;
- SHA-256 checksum for the manifest and every payload;
- optional signature metadata and signing key identity;
- install scope support: user, workspace, or both;
- declared runtime requirements, such as `wasm` feature, model runtime, or resource consumer;
- failure fallback text and uninstall/cleanup metadata.

Plugin-package manifests must also reference the plugin package manifest and must not substitute for
plugin provenance, capability declarations, or permission policy.

## Source And URL Policy

Talos-owned release manifests may point to Talos-controlled release assets. Third-party package
registries and marketplaces are out of scope until a later ADR defines trust, moderation, revocation,
and namespace rules.

Allowed source modes:

- local file path supplied by the user;
- configured mirror root supplied by user or enterprise policy;
- Talos release asset manifest shipped with or referenced by the installed Talos version.

Disallowed source modes for the first implementation:

- implicit web search for packages;
- automatic third-party registry lookup;
- installing from arbitrary URLs pasted into conversation text;
- transitive package downloads triggered by a package without showing the full asset list first.

## Consent And UX

Asset installation is explicit. The first supported shape should be a CLI command such as:

```text
talos assets install <asset-id>
talos assets list
talos assets status <asset-id>
talos assets remove <asset-id>
```

Future TUI prompts may be added after the manual command path exists. A prompt must show the asset
id, class, publisher/source, version, size, install scope, verification mode, and fallback behavior.
The default response is cancel unless the user explicitly accepts.

CI, enterprise, and high-security configurations must be able to disable online asset installation.
When disabled, Talos may still read pre-seeded local caches but must not open a network path.

## Cache And Install Layout

Use Talos-controlled state paths. Do not install optional assets into source trees or hidden runtime
paths that bypass user cleanup.

Recommended layout:

```text
~/.talos/assets/
  manifests/<asset-id>/<version>/manifest.toml
  payloads/sha256/<hex-digest>
  installed/<asset-id>/current.toml
  logs/install-history.jsonl

<workspace>/.talos/assets/
  installed/<asset-id>/current.toml
```

Payloads are content-addressed by checksum. Install records point to verified payloads and include
scope, Talos version, install time, source mode, and verification result. Workspace-scoped assets
may reference user-scope payload blobs but must keep workspace activation records local.

## Verification And Activation

Verification happens before install records are written:

1. Read and validate the manifest schema.
2. Check Talos version/protocol compatibility.
3. Download or read payloads into a temporary path.
4. Verify byte count and SHA-256 checksum.
5. Verify signature when the source requires one or the manifest declares one.
6. Move payloads into content-addressed storage.
7. Write install record atomically.

Activation is a separate step from installation. Installed plugin packages do not register tools,
commands, skills, or hooks until the plugin loader explicitly accepts the package and applies
provenance, sandbox, and permission gates. Installed model weights do not become a decision
authority; missing, incompatible, slow, or corrupted weights must fall back to deterministic or
provider-backed behavior.

## Offline, Mirror, And Proxy Behavior

Offline mode is first-class. If online installation is disabled or unavailable, Talos should:

- list missing assets as unavailable, not fatal;
- accept pre-seeded manifests and payload blobs from a configured local path;
- verify all pre-seeded assets with the same checksum/signature policy;
- avoid startup network access;
- produce diagnostics outside normal conversation history.

Mirrors must preserve asset ids, versions, checksums, and signing metadata. A mirror may rewrite the
root location, but it must not rewrite payload identity without an explicit enterprise policy.

## Update, Uninstall, And Cleanup

Updates are explicit and versioned. Talos must not replace an installed optional asset silently.

Uninstall removes install records first and then garbage-collects unreferenced payload blobs. Cleanup
commands should report disk usage by class and scope. A failed uninstall must leave either the old
asset active or no asset active; it must not leave a half-active executable package.

## Failure And Fallback

Failures are non-fatal to normal startup and provider conversations:

- manifest invalid: reject the asset and show a diagnostic;
- checksum/signature mismatch: delete temporary payloads and reject;
- incompatible Talos/protocol version: keep existing installed version if any;
- missing optional asset: continue without the capability;
- plugin trap or loader failure after install: disable that package and preserve the install record
  for diagnosis;
- model load failure: disable the helper and use deterministic/provider fallback.

Diagnostics should be visible through future `talos assets status` and related TUI surfaces, not
injected into the model context by default.

## Security Policy

- No silent downloads.
- No startup-time network dependency.
- No install from conversation text without a dedicated command/prompt.
- No executable activation during download or manifest inspection.
- No dynamic libraries or standalone executable hook carriers.
- No permission-default changes due to asset installation.
- Checksums are mandatory; signatures are required for Talos-controlled release channels once a
  signing key policy exists.
- Mirrors are location policy, not trust expansion.
- Revocation, publisher trust, and third-party marketplace behavior require a later ADR.

## Relationship To Existing Work

- `DIST-001` owns this policy and remains the gate before online asset installation.
- `PLUGIN-001` may use this policy only after local explicit plugin loading is safe; remote plugin
  install still needs a follow-up ADR.
- `MODEL-002` may reference this policy for model-weight distribution, but model runtime and model
  authority remain separate ADR topics.
- `TOOL-008` language-resource downloads, if pursued, should use the resource-pack class.

## ADR Draft Outline

A later ADR should decide:

- manifest schema location and versioning;
- signing requirement and key-management model;
- exact user/workspace state paths;
- whether first implementation supports local-file install only or also Talos release manifests;
- network client/proxy implementation boundary;
- command/TUI prompt surface;
- revocation and enterprise mirror policy.

No online installer, automatic prompt, third-party registry, marketplace, or plugin execution path
should be implemented before that ADR is accepted.

## Alternatives Considered

- Bundle all optional assets in default releases. Rejected because it increases release size and
  forces unused runtime assets onto every user.
- Download on first use without prompting. Rejected because it hides network, disk, and executable
  trust decisions.
- Treat plugin package installation as separate from model/resource distribution. Rejected because
  checksum, cache, offline, mirror, and consent concerns are shared, even though activation gates
  differ.
- Allow arbitrary URL install as the v1 path. Rejected because it creates a trust and provenance
  surface before Talos has publisher, signature, and revocation policy.

## Open Questions

- Which signing mechanism should Talos use for release-channel assets?
- Should workspace-scoped assets be allowed to reference user-scope payload blobs by default?
- What is the minimum useful local-file-only installer slice?
- Which TUI prompt surface should host asset consent after manual commands exist?
- How should enterprise policy express allowed mirrors and disabled online install?

## Dependencies

- Accepted follow-up ADR for any online asset installation.
- Local explicit plugin package loading and provenance before remote plugin packages can be useful.
- MODEL-002 dependency/runtime ADR before any model weights are installed or loaded.
