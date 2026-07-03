# DIST-001: Optional Runtime Asset Distribution

## Outcome

Talos has a distribution strategy for large optional runtime assets so the default installation
stays lightweight while users can explicitly install additional WASM plugins, local model weights,
or other bulky capabilities after installation.

## Status

Policy proposal complete via I091 A8 on 2026-07-04. Follow-up ADR required before any online
asset installation, runtime download path, marketplace behavior, or automatic prompt.

I091 scope is policy-only unless a narrow local diagnostic gap is found. Do not implement online
asset installation, marketplace behavior, remote plugin package fetching, automatic prompts, or
runtime download paths in this iteration.

I091 A8 produced `docs/proposals/optional-runtime-asset-distribution.md`, an ADR-ready policy for
manifest schema, checksum/signature verification, Talos-controlled cache layout, consent UX,
offline/mirror behavior, activation separation, uninstall/cleanup, and failure fallback.

## Priority

P3.

## Origin

User feedback on 2026-06-19: future size-expanding features such as WASM plugins and local model
files should not necessarily inflate the default release. Talos can prompt users after install and
download optional assets online at runtime when needed.

## Problem

Some planned capabilities are valuable but may substantially increase release size:

- local micro-model weights and inference assets;
- optional WASM plugins;
- large language-specific resources;
- future local indexes, templates, or capability packs.

Bundling all of them into the default binary or release archive would make Talos slower to
download, harder to distribute, and less suitable for minimal environments. At the same time,
runtime downloads create security, reproducibility, offline, and consent concerns.

## Scope

Design an optional asset distribution model for large non-core capabilities.

Required areas:

- asset manifest format;
- source registry and URL policy;
- user prompt and consent flow;
- checksum/signature verification;
- install path and cache layout;
- version compatibility with the Talos binary;
- offline/air-gapped behavior;
- uninstall and cleanup behavior;
- proxy/mirror support;
- failure and fallback behavior;
- observability without cluttering conversation history.

## Distribution Direction

Default releases should include only core runtime assets required for normal Talos operation.

Large optional assets should be handled by one of these paths:

1. User explicitly installs them through a command or TUI prompt.
2. Talos detects a missing optional capability and asks before downloading.
3. Enterprise/offline users pre-seed an asset cache from a trusted mirror.
4. Users disable online asset installation entirely.

Potential examples:

- `talos assets install local-router-model`
- `talos assets install wasm-plugin:<name>`
- TUI prompt: "This feature requires a 120 MB local model. Download now?"

I091 A8 policy narrows first implementation to an explicit manual command path before automatic
prompts. Download and install must be separate from activation: installed plugin packages do not
register capabilities until plugin loader/provenance/sandbox/permission gates accept them, and
installed model weights never become permission authority.

## Hard Boundaries

- Talos must not silently download large executable or model assets.
- Runtime downloads must be opt-in and interruptible.
- Downloaded assets must be verified by checksum and, if available, signature.
- A failed or missing optional asset must degrade to the existing behavior.
- Core features must not depend on startup-time network access.
- Asset installation must respect proxy, mirror, and offline/disabled-network configuration.
- Downloaded plugins must still go through plugin protocol, provenance, sandbox, and permission
  gates.
- Downloaded model files must not become permission authority or replace provider configuration.

## Research Questions

- Should optional asset metadata live in Talos release metadata, a signed registry file, or
  provider/plugin-specific manifests?
- What verification level is required for model weights versus WASM plugins?
- Where should user-scoped and workspace-scoped asset caches live?
- How should asset versions map to Talos binary versions and protocol versions?
- Can users export/import asset bundles for offline machines?
- Should asset download prompts render in the future slash/input popup layer?
- How should policies disable runtime downloads in CI, enterprise, or high-security settings?
- What asset cleanup and disk-usage reporting should be exposed?

## Acceptance Criteria

- [x] A distribution proposal defines asset manifests, install/cache layout, verification, update,
      uninstall, and offline behavior.
- [x] The proposal distinguishes model weights, WASM plugins, and non-executable resource packs.
- [x] Runtime download consent and prompt UX are specified.
- [x] Security policy covers checksum/signature verification, provenance, mirrors, and disabled
      online installs.
- [x] MODEL-002 and PLUGIN-001 both reference the shared distribution strategy instead of defining
      incompatible download paths.
- [ ] A follow-up ADR is drafted before implementing online asset installation.

## Non-Goals

- Do not implement asset downloads in this item.
- Do not choose a plugin marketplace protocol.
- Do not bundle model weights or WASM plugin packages.
- Do not make optional assets required for normal startup.

## Relationship To Other Work

- `MODEL-002` depends on this strategy before any local model weights are shipped or downloaded.
- `PLUGIN-001` depends on this strategy before remote WASM plugin package installation is allowed.
  **(2026-06-30) The plugin-package *distribution* slice of DIST-001 is additionally blocked on
  `docs/proposals/plugin-encapsulation-format.md` — the package format must be decided before its
  distribution path. DIST-001's broader research scope (model weights, non-executable resource packs)
  can proceed independently.**
- `TUI-010` may provide the future prompt surface for optional asset installation.
- `AGENT-001` may influence shared config locations for asset policy, but Talos-owned assets should
  remain under Talos-controlled state unless a later ADR decides otherwise.

## Residual Work Destination

If accepted, create a distribution ADR and a narrow implementation story for manual installation of
one safe asset type before adding automatic prompts or third-party plugin registries.
