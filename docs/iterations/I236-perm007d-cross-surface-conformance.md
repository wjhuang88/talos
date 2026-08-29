# Iteration I236: PERM-007-D Cross-Surface Conformance

> Document status: Active / Claimed (proposed; ineffective until this governance record merges)
> Published plan date: 2026-08-30
> Planned objective: complete the remaining PERM-007-D cross-surface conformance, rollout and rollback evidence for Issue #188 without widening the accepted ADR-064 authority.
> MVP deliverable: runnable conformance fixtures proving equivalent `Ask` requests have equivalent auto/fallback semantics across Goal, interactive CLI/TUI, headless, Runtime and MCP entrypoints.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline permission session |
| Work Slice | PERM-007-D/I236 only: cross-surface wiring and conformance evidence for the existing I234 resolver contract, bounded rollout/rollback and user/API documentation. No new authority class, sandbox fallback, Dashboard/Desktop, release or publication work. |
| Claimed At | 2026-08-30 |
| Source Issue | #188 |
| Governance Claim PR | #436 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-064 Accepted; PERM-006-A/B/C, PERM-007-B and PERM-007-C/I234 are Complete/Closed. I234 implementation PR #434 merged as `c5be0109b3da4f81e221fa37f734af2431e35255`; closeout PR #435 merged as `469f50d1959e57551b1c58537d17f15f32dd303c`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-30 |
| Handoff / Release Condition | Requires exact-head CI, independent cross-surface permission/security/API review and merge-time CAS. Issue #188 remains open until D closeout. |

## Current Nonterminal Inventory And Disposition

| State | Iterations | Disposition |
|---|---|---|
| Active | I236 (proposed) | This claim is ineffective until its governance PR reaches `main`; no implementation branch before then. |
| Review | None | No review iteration blocks the claim. |
| Planned | I207, I208 | Preserve as unclaimed steering children; no overlap. |
| Blocked | None with a current iteration document status | PERM-007-D was unblocked by I234 completion; no other authority is transferred. |
| Paused | I164 | Superseded; do not restore. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-D | PERM-007 / Issue #188 | Ready / Unclaimed | ADR-064; PERM-006-A/B/C; PERM-007-B/I233; PERM-007-C/I234 | Cross-surface behavior-equivalence evidence, bounded rollout and rollback for the accepted resolver. |

### Scope

- Wire the already-implemented resolver semantics through each supported permission surface without changing eligibility or authority.
- Prove equivalent normalized requests preserve Deny precedence, auto-off fallback, human-required escalation, headless Deny and redacted audit semantics.
- Add rollout diagnostics and a reversible kill switch using existing `auto.enabled` and session `/auto off`; no persistent implicit grants.
- Document supported/unsupported surfaces, degradation behavior, operator rollback and API compatibility.

### Non-Goals

- No new model evaluator or permission policy, grant widening, sandbox fallback, Execute/Network authority or existing-file modification.
- No Dashboard/Desktop implementation, release/version/tag/publication, or unrelated CLI/TUI refactor.

## Acceptance And Validation

- A shared fixture matrix covers Goal, interactive CLI/TUI, headless, embedded Runtime and standalone MCP with the same request/context and records equivalent decision, fallback and audit outcomes.
- Hard Deny and unsupported contexts remain Deny on every surface; auto-off and assessor failure never produce automatic authorization.
- Rollout can be disabled through configuration/session control and resumes the pre-I234 human/headless behavior without migration or grant residue.
- Exact-head focused tests, locked workspace validation, governance validators, diff check, independent review and merge-time CAS all pass.

## Completion Evidence

Completion Commit: Pending. A status-only governance commit cannot self-certify implementation.

## Residuals

Issue #188 remains open until I236 implementation and owner-first closeout provide cross-surface evidence and a Completion Commit.
