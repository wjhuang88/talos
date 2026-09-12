# Iteration I261: Rust WASM Language Provider Vertical Slice

> Document status: Planned / Claimed (pending governance merge)
> Parent: CAP-001 / #466; LANG-002 / #516
> Objective: prove one manually installed Rust WASM Language Provider serves highlighting and symbol consumers through the shared contract.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | One Rust WASM LanguageProvider vertical slice through verified Bundle installation, bounded loading, registration, and shared consumers. |
| Source Issue | #516 |
| Governance Claim PR | Pending |
| Implementation PR | Not started |
| Authorization Evidence | Proposed claim is ineffective until merged to main; implementation remains unauthorized. |
| Last Updated | 2026-09-12 |

Claim preparation is governance-only; implementation starts only after the claim reaches main.

### Claim Preparation Checkpoint — 2026-09-12

Owner and work slice are recorded locally for governance review. This checkpoint does not
authorize implementation; the claim becomes effective only when its governance PR merges to main.

## Scope and Acceptance

- Resolve one `language.rust` WASM Provider through the shared contract for TUI highlighting and symbol queries.
- Use a manually verified Bundle; enforce loader limits, trap/resource containment, timeout handling, and plain-code fallback.
- Keep parser-native types behind the provider boundary and preserve no-network startup behavior.

## Non-Goals

No remaining-language migration, marketplace/download flow, Desktop binding, or silent executable acquisition.

## Dependencies

LANG-001/I257, CAP-001-C/I255, BUNDLE-001/I259, and DIST-001-A/I260 are Complete / Closed.

## Governance Gate

Before implementation, assign an owner, establish an effective Collaboration Claim, and obtain an independent security/dependency review. Until then this iteration remains Planned / Unclaimed.
