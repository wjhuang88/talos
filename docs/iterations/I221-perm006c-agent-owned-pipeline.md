# Iteration I221: PERM-006-C Agent-Owned Permission Pipeline Implementation

> Document status: Active / Claimed proposed by PR #375; ineffective until target-branch merge
> Published plan date: 2026-08-23
> Planned objective: implement the Accepted ADR-067 single-owner permission orchestration and
> cross-surface migration without widening authorization or changing unrelated product lanes.
> Baseline rule: preserve this target; a changed objective requires a new iteration ID.
> MVP deliverable: one runnable Agent-owned pipeline used by all in-scope surfaces, with exact
> normalized input, bounded resolver, revision-CAS admission, final execution-gating hook and
> cross-surface validation evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-23 |
| Work Slice | Implement only PERM-006-C under Accepted ADR-067: Agent-owned controller/resolver contracts; one normalized authoritative request; exactly-once evaluation; Once/Session/Deny bounded resolver; proposal/revision CAS and admission fencing; one final `AfterPermissionCheck` execution gate; fail-closed deadline/cancellation/error handling; policy-free compatibility adapters and cross-surface migration for CLI print/headless, interactive TUI, inline/RPC, embedded Runtime and standalone MCP. Preserve existing serialized configuration, Runtime `ApprovalHandler` compatibility and sandbox fallback boundary. |
| Claimed At | 2026-08-23 |
| Source Issue | #55 |
| Governance Claim PR | #375 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-067 Accepted through PR #373 merge `5d2d2dcf`; exact decision CI `32619757871`; independent permission/security/API review `5384374028`. Maintainer authorization for the mainline permission chain remains limited to this bounded I221 slice; fresh exact-head claim CI/review/CAS are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Claim is ineffective until merge. After merge, create the implementation branch from the claim merge or later `main`; do not modify implementation before claim effectiveness. Completion requires pre-existing implementation commit evidence, exact-head CI, independent permission/security/API review and merge-time CAS. |

## Published Baseline

Planning target: `main@055e5c6b` after I220 closeout.

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I221 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore. |
| I197/I198/I201/I210 | Review / Claimed | Preserve corrective owners and deferred acceptance. |
| I206-I208 | Planned / Unclaimed | Preserve ordered steering sequence; do not activate. |
| I213 | Active / Claimed | Dashboard lane only; I221 must not modify Dashboard owner or `crates/talos-dashboard/**`. |
| I220 | Closed (decision accepted) | ADR-067 is the governing prerequisite. |
| I221 | Active / Claimed proposed by #375 | Implementation claim is proposed; no implementation authority exists before merge. |

Open PR #372 remains the Dashboard implementation lane and must be kept separate. PRs #120/#121
remain archival Drafts. Any shared derived-file update is owner-first and must preserve union
semantics; a production authority overlap with I213 stops I221 immediately.

## Scope And Non-Goals

### In scope

- Agent-owned controller and bounded `ApprovalResolver` contracts.
- Shared normalized request, proposal identity, permission revision and admission fence.
- CLI print/headless, interactive TUI, inline/RPC, embedded Runtime and standalone MCP adapters.
- Exactly-once evaluation and final `AfterPermissionCheck` hook ordering.
- One total deadline, cancellation propagation and fail-closed errors/closures/timeouts.
- Policy-free compatibility adapters and additive Runtime/MCP public API migration.

### Explicitly excluded

- `crates/talos-dashboard/**`, Dashboard owner or live-activity behavior.
- PERM-006-D/E, PERM-007 behavior, persistent grants and unrelated permission stories.
- TOOL-024-B/C/D, Issue #59 background jobs and session job persistence.
- `/auto` model-assisted behavior (ADR-064 remains separate).
- Sandbox fallback policy changes; fallback remains a separate bounded authority.
- Release, version, tag, crates.io publication, Desktop and product UI work.

## Acceptance And Validation

- Every in-scope surface routes through one Agent-owned evaluator and one final execution gate.
- Approval is for the exact normalized request; post-approval permission mutation invalidates it.
- Once/Session/Deny resolver scope, Deny dominance, capability-relative Session grants and
  no-resolver/headless fail-closed behavior are tested.
- Concurrent approval, stale revision, Session closure, deadline and cancellation cases fail closed.
- Hooks are ordered proposal -> before-check -> evaluation/resolution/admission -> final hook ->
  execution, with final hook Deny preventing execution.
- Existing Runtime `ApprovalHandler`, serialized permission configuration and compatibility messages
  remain covered by migration tests.
- Cross-surface tests prove no alternate policy evaluator remains; redaction tests cover logs/hooks.
- Both governance validators, locked required tests, exact-head CI, independent permission/security/
  API review and merge-time CAS pass before completion.

## Verification And Completion

Implementation PR: Not started. Completion Commit: pending.
The claim/activation status commit cannot self-certify implementation. Any residual compatibility
adapter or un-migrated surface must be recorded in this owner or a declared follow-up story.
