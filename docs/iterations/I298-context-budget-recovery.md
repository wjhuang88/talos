# Iteration I298: Authoritative Context Budget And Overflow Recovery

> Document status: Complete
> Published plan date: 2026-10-09
> Planned objective: Fix misleading zero context usage and recover bounded tool loops before request budget overflow.
> MVP deliverable: A TUI session with missing provider usage displays request-budget estimates and continues after recoverable context pressure without losing durable tool results.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | I298 / MEM-005-A: authoritative request budget telemetry, bounded request-only overflow recovery, TUI estimate/unknown display, regression tests and migration documentation. |
| Claimed At | 2026-10-09 |
| Source Issue | None; maintainer incident and explicit repair request |
| Governance Claim PR | #690 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested this repair in single-maintainer mode and accepted ADR-087 on 2026-10-09. No independent human maintainer is available; independent Agent API/privacy review and exact-head gates remain required. |
| Implementation PR | #691 (merged) |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Implementation merged with exact-head CI, independent Agent API/privacy approval and CAS; ship only in the next minor release under ADR-087, not the I297 patch. |

## Published Baseline

### Selected Stories

MEM-005-A is a bounded corrective child of MEM-005, related to TUI-017. It does not
claim completion of MEM-005, MEM-003 or MEM-007.

### Scope And Acceptance

- Derive transient budget facts from the exact request admission calculation, including input,
  tools, safety margin and output reservation. Missing provider usage must not mean zero context.
- Check initial and continuation requests. When over budget, compact only the model-facing
  request projection using bounded deterministic recovery. Preserve system/instruction messages,
  user intent, complete tool call/result identity and the most recent usable tool evidence.
- Recalculate after recovery; dispatch only within the actual limit. Impossible requests fail
  clearly without increasing the limit, silently dropping instructions or replaying tools.
- Preserve raw UI/export/durable history, hidden-output boundaries and permission decisions.
- Test long multi-tool turns, missing usage, irreducible requests, Unicode, tool pairing,
  raw-history preservation and cancellation. No real provider or private log fixture required.
- Record additive core event and any presentation API migration in ADR-087. Breaking Rust
  presentation constructors require the next pre-1.0 minor release, never the I297 patch.

### Non-Goals

No provider retry/timeout defaults, model catalog inference, dependency/toolchain changes,
permissions policy, automatic tool replay, persistent schema change, new compression model,
manual compact command, or release publication. No rewriting old completion evidence.

### Validation And Documentation

Focused locked tests for core/agent/conversation/TUI and affected runtime adapters; full
`./scripts/release_preflight.sh` before implementation merge. Run both governance validators,
`git diff --check`, independent Agent API/privacy review, exact-head CI and merge-time CAS.
Update README.md, README.zh-CN.md and ADR-087 migration guidance; owner first, indexes second.
Rollback is a source revert; no session migration or deletion is necessary.

## Activation Checkpoint — 2026-10-09

Claim PR #690 merged as `f4a70729`; implementation started from that main baseline.
Governance head: `19df7b7ebd150909ea91d1a8bf06ee47d31f1ab5`;
base: `9e376155ddc7b6c3644bf65f69b4c388956c1b5c`.
Independent Agent document/API/privacy review approved the governance candidate only.
CI `37908264351` passed applicable documentation-route checks; Windows Rust and Linux Desktop
were skipped. The executing agent did not check CI or record CAS before merging #690.
Later CI verification is after-the-fact evidence, not retroactive premerge CAS. Implementation
must obtain fresh code review and CI and perform CAS before merge.

## Selection Inventory — 2026-10-09

Baseline: main `9e376155ddc7b6c3644bf65f69b4c388956c1b5c`.
I293/I294 remain Active for search only. I277 remains Review with deferred device acceptance;
I290/I291 remain Review for Auto locale. I249 remains deferred Planned and I164 Paused.
I296 (#688) and I297 (#689) remain separate unmerged governance candidates; this maintainer's
repair request takes priority in this session, not their implementation or release authority.
Open PR #682 touches Auto locale, not request budgeting. Shared agent tests and README updates
must be union-merged with that work. No overlapping context-budget PR was found.
Historic nonterminal wording in completed iteration checkpoints is not reactivation authority.

## Incident Evidence And Limits

The maintainer supplied a private session log: 190 valid records, 58 tool-result records,
approximately 112 KB of tool-result text, and a turn with 17 tool-use responses followed by a
provider error. The log does not contain request budget decomposition or usage and cannot prove
the exact cause of the reported 128006/128000 rejection. Do not commit the private log.
Source inspection confirms usage-based display, history-only pre-turn compaction, and immediate
continuation budget rejection. Storage compaction markers are not semantic summaries.

## Execution And Completion

- 2026-10-09: Maintainer accepted ADR-087, including the presentation API change and next-minor
  release boundary. Effective claim and implementation validation remain pending.
- 2026-10-09: Repair requested; governance candidate prepared. Implementation not started.
- 2026-10-09: Independent Agent design review identified protected Context/multimodal/reasoning,
  atomic tool-exchange removal, final image reservation and legacy precompaction boundaries.
  ADR-087 now records these constraints. New recovery must not mutate raw history; existing
  pre-turn compaction is unchanged and is not covered by a broader no-compaction promise.
- Validation: project governance and Collaboration Claim validators passed with 0 warnings;
  `git diff --check` passed before the review clarifications. No code validation claimed.
- Completion Commit: 8d4ad50e4f8e9bc5c620db2641056b1c1118183d.
- 2026-10-09 local checkpoint: request-plan tests 11/11 and Agent library tests 455/455 passed
  (`cargo test -p talos-agent --locked --lib`). Admission/billing and session-reset regression
  passed in talos-conversation. TUI tests and Clippy failed to build because disk space was
  exhausted, not because verification passed. Authorized `cargo clean` removed 13 GiB of
  rebuildable artifacts. Full validation, remaining regression coverage and code review are
  pending; implementation is not complete.
- Residual: broader MEM-005 policy and archival semantic-summary design remain with their owners.
- 2026-10-09 stable local candidate: authoritative numeric budget events, correlated pre-start
  rejection, projection-only recovery and unknown/estimated TUI display implemented. Focused
  Agent/conversation/TUI suites passed 456/178/595 tests; CLI identity regression passed.
  `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
  ./scripts/release_preflight.sh` exited 0, including locked workspace check, Clippy,
  all workspace tests/doctests and both external Runtime SDK fixture configurations.
  Governance and Collaboration Claim checks reported 0 warnings. Low-debug environment settings
  conserve disk only; repository profiles and the pinned 1.97.0 toolchain are unchanged.
  Independent Agent pre-review found no remaining API/privacy blockers after real continuation
  and identity-correlation tests; final exact-head approval, remote CI and CAS remain pending.

## Implementation Acceptance And Closeout — 2026-10-09

PR #691 merged to main as `8d4ad50e4f8e9bc5c620db2641056b1c1118183d`.
Final head `90f63c092f913d4b45bf1b9060e0b78fce3562e5`, base
`f4a707298993a994585311ad9dc8e8c8504ec642`. CI `37914111373` passed all six jobs,
including macOS full preflight, Windows workspace and Linux Desktop. Independent Agent
API/privacy/security APPROVE: PR #691 comment `6078582512`; no blockers, Agent identity
disclosed (not human approval). Merge-time CAS: comment `6079057562`, unchanged head/base,
effective claim, no competing slice, clean mergeability and all gates passed before merge.

Acceptance uses deterministic provider/tool fixtures: missing billing still yields admission
facts, initial rejection is identity-correlated, continuation recovery executes each tool once,
preserves durable results, protects instructions/reasoning/latest exchange and refuses
irreducible requests. TUI regressions cover unknown and estimated/over-limit labels. Existing
cancellation tests passed in full workspace validation. No private incident log is distributed.
These checks establish local admission behavior, not exact provider tokenizer equivalence.

No residual within MEM-005-A. Broader compaction policy, semantic summaries and manual commands
remain MEM-005 work; existing pre-turn compaction is unchanged. No release performed. Next-minor
publication must include ADR-087 StatusSnapshot migration; I297 patch must exclude this API change.
