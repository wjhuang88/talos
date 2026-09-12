# LANG-002: Rust WASM Language Provider Vertical Slice

| Field | Value |
|---|---|
| Story ID | LANG-002 |
| Type | Language Provider implementation |
| Parent | CAP-001 / #466 |
| Status | Complete / Closed |
| Selected Iteration | I261 |
| Source Issue | [GitHub Issue #516](https://github.com/wjhuang88/talos/issues/516) |
| Depends On | LANG-001; CAP-001-C; DIST-001-A verified manual installation |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | One Rust WASM LanguageProvider vertical slice through verified Bundle installation, Plugin loading and the shared consumer contract. |
| Claimed At | 2026-09-12 |
| Authorization Evidence | Claim PR #542 merged to main as `aa91ddc2`; implementation authorized within this Work Slice. |
| Governance Claim PR | #542 |
| Implementation PR | #543 (merged as `935c4b86`) |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-12 |
| Handoff / Release Condition | Requires a separate security/dependency review before implementation merge. |

Completion Commit: `935c4b861c4bda75cb0d05383db110a50256afbe`

Exact-head CI `34694711467` and independent security/dependency approval `5646284106` are recorded in I261 closeout.

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), and [ADR-027](../../decisions/027-plugin-runtime-boundary.md).
- [LANG-001](LANG-001-language-provider-contract-migration.md), [CAP-001-C](CAP-001-C-plugin-capability-carriers.md), and [DIST-001-A](DIST-001-A-verified-manual-bundle-installation.md).

## Goal And Scope

Prove one end-to-end `language.rust` WASM Provider can serve highlighting and symbol consumers
through the shared contract. A manually verified Bundle supplies the artifact; installed and
authorized Plugin loading, initialization and activation register the Provider. Built-in-only
adapters or descriptor fixtures do not satisfy this vertical slice.

## Non-Goals

No remaining-language migration, marketplace/download flow, broad parser trimming, Desktop binding,
or silent executable acquisition.

## Acceptance

- Rust code resolves one Provider and both TUI and symbol paths consume it.
- The same WASM Provider is installed through DIST-001-A, loaded under ADR-027 limits,
  registered through CAP-001-C, and exercised by both real consumers without startup network.
- Missing, corrupt, incompatible or timed-out Provider leaves the process healthy.
- WASM traps and resource exhaustion are contained; highlighting falls back to plain code
  and symbol/query consumers report explicit provider-unavailable results.
- No parser-native types cross the shared boundary and no default behavior regresses.
- Record comparable before/after binary-size and static-parser dependency evidence. Merely
  moving all parsers into another statically linked crate is not progressive loading.

## Validation And Documentation

Offline end-to-end fixtures, failure-path tests, dependency/size evidence, locked checks and
security review. Binary-size reduction is not claimed unless measured by this slice.
