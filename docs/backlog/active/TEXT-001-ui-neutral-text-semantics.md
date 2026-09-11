# TEXT-001: UI-Neutral Text Semantics

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | TEXT-001 |
| Type | Shared text contract |
| Parent | CAP-001 / #466 |
| Status | Complete / Closed |
| Selected Iteration | I256 |
| Source Issue | [GitHub Issue #511](https://github.com/wjhuang88/talos/issues/511) |
| Depends On | CAP-001-A; ADR-072 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | Shared text classification, language identity and validated semantic fallback; narrow TUI adapters only. |
| Claimed At | 2026-09-10 |
| Authorization Evidence | #530 effective at c8b596980136a577921bf92cdcbe1e517e64d10a; exact head c5f94f31, CI 34474441483, independent API/compatibility APPROVE 5618377071 and CAS 5618385106. Shared account establishes Agent-role separation only. |
| Governance Claim PR | #530 |
| Implementation PR | #531 |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-11 |
| Handoff / Release Condition | Implementation merged through #531; LANG-001 requires its own effective claim. |

## Completion Evidence (2026-09-11)

Completion Commit: `a1a215e02635a984c4ee3524d18f4c096eb83fa9` (PR #531 merge).
Exact-head CI `34476929013` and independent approval `5618698607` passed.
This supersedes the dated pre-merge checkpoints below, without rewriting their history.
Shared/default/optional-parser tests, full workspace preflight and the binary PTY fixture
passed; I256 records the detailed acceptance and limits. Desktop integration is not claimed.

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md) and [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md).
- I246/CAP-001-P0 compatibility evidence; this contract must remain UI-neutral and does not authorize parser loading or distribution.

## Goal And Scope

Define shared text/code-block semantics and deterministic plain-text fallback for TUI, Desktop and
tools. Results must not expose Ratatui, GPUI, Arborium or Tree-sitter types.

CAP-001-C is not a prerequisite for this contract. I253 is complete and I256 records the
consumer ownership inventory. #530 established selection; scheduling CAP-001-B/C first
does not create a technical dependency on their Plugin/Carrier branch.

## Non-Goals

No parser trimming, dynamic loading, Bundle installation, WASM provider, UI layout, Desktop binding,
or language-specific implementation.

## Acceptance

- Language IDs and aliases normalize deterministically.
- Semantic results are renderer-independent and streaming-safe.
- Unsupported, unavailable or malformed providers fall back explicitly to plain text.
- TUI and Desktop can consume the same contract without importing each other's crates.

## Validation And Documentation

Contract fixtures, fallback/property tests, architecture/API docs and an explicit shared-file
ownership inventory. No default Cargo feature expansion is permitted by this owner.

## I256 Selection Preparation (2026-09-10)

I253 audit is Complete, as are CAP-001-A and I246 compatibility preparation. The consumer
inventory and bounded shared-file agreement are recorded in
[I256](../../iterations/I256-ui-neutral-text-semantics.md): reuse existing `talos-text`, extract
renderer-neutral stream classification and validate semantic fallback, with narrow TUI adapters.
Current main has no competing Active/Review iteration or open PR. This is a proposed serial
ownership boundary, not effective authority before atomic claim merge and plan review.
Preserve public constructors/serialized forms, canonical-only TUI grammar admission, existing
stream rendering, Cargo defaults and dependency versions. LanguageProvider dispatch and symbol
migration remain LANG-001; parser loading, distribution and Desktop binding remain excluded.
No implementation code is authorized by this preparation.

## Atomic Claim Proposal (2026-09-10)

#530 proposes In Progress / Claimed with I256 Active / Claimed; both are ineffective until
main merge. I256 preserves the detailed compatibility contract and explicitly permits two
exceptional-input corrections: bounded fallback for previously unbounded held blocks/first
lines, and whole-result plain-text fallback for malformed highlights instead of clamp/skip.
Normal rendering, public constructors/serde and default build behavior remain unchanged.
No other TUI, Desktop, LanguageProvider, installation or permission authority is transferred.

## Effective Activation (2026-09-10)

#530 merged at `c8b596980136a577921bf92cdcbe1e517e64d10a` after exact head
`c5f94f3114d60260528a8ddf40ceadaddadcd7a8` / base
`dba3419c88f89ed6aa108de8d638ebe1c8ac3c64`, CI `34474441483`, independent
API/compatibility approval `5618377071` and CAS `5618385106`. This supersedes
proposal-only current wording; the Published Baseline and dated preparation facts are preserved.
Implementation branch `feat/i256-ui-neutral-text-semantics` starts exactly at the claim merge.
No implementation PR exists yet; all implementation/testing corrections remain local.

## Local Acceptance (2026-09-10)

I256 records passing shared/default/optional-parser tests, 571 TUI library tests, full locked
workspace preflight and real binary PTY evidence. Shared classification and whole-result highlight
validation are consumed by the TUI; legacy public types/defaults and normal rendering are preserved.
README/architecture/rustdoc describe the implemented boundary. Review / Claimed awaits stable
candidate exact-head CI, independent API/compatibility approval and CAS; no Complete claim or
implementation SHA exists yet. Detailed acceptance and changed-file inventory are in I256.
