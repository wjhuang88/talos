# Iteration I221: PERM-006-C Agent-Owned Permission Pipeline Implementation

> Document status: Complete / Closed
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
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-23 |
| Work Slice | Implement only PERM-006-C under Accepted ADR-067: Agent-owned controller/resolver contracts; one normalized authoritative request; exactly-once evaluation; Once/Session/Deny bounded resolver; proposal/revision CAS and admission fencing; one final `AfterPermissionCheck` execution gate; fail-closed deadline/cancellation/error handling; policy-free compatibility adapters and cross-surface migration for CLI print/headless, interactive TUI, inline/RPC, embedded Runtime and standalone MCP. Preserve existing serialized configuration, Runtime `ApprovalHandler` compatibility and sandbox fallback boundary. |
| Claimed At | 2026-08-23 |
| Source Issue | #55 |
| Governance Claim PR | #375 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-067 Accepted through PR #373 merge `5d2d2dcf`; claim #375 merged as `d662501c`; implementation PR #376 exact head `aed71fb4`, base `d662501c`, CI `32640691772`, independent permission/security/API approval `5386153429`, merge-time CAS and merge `f9e6706d`. Shared GitHub identity establishes Agent-role separation only. |
| Implementation PR | #376 |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Closed after owner-first closeout; PERM-006-D/E and TOOL-024 require separate owners and claims. |

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

Implementation PR: #376. Completion Commit: `49d1546c3748930177655dbedc7f3665780d92ab`.
The claim/activation status commit cannot self-certify implementation. Any residual compatibility
adapter or un-migrated surface must be recorded in this owner or a declared follow-up story.

## 2026-08-23 Activation And Local Candidate Checkpoint

I221 claim #375 is effective on `main@d662501c` (claim head `de99de1c`, base `055e5c6b`, CI
`32620749103`, independent review `5384445091`, successful merge-time CAS and merge `d662501c`).
A local implementation candidate converged from that exact main and remains unsubmitted pending
final changed-file and staged-diff review. Production CLI, TUI, Runtime and MCP composition roots
register raw tools and use the Agent-owned pipeline; legacy policy-bearing wrappers are test-only
compatibility fixtures and cannot be reached by production builds. Hooks and logs receive a
structure-only projection, while approval resolvers receive the tool-defined safe presentation;
authorization and execution retain the exact normalized request.

The permission pipeline requires one total deadline at every call site and uses non-blocking
Session fences so lock contention fails closed instead of extending that budget. Non-TUI
interactive approval is serialized by the existing event-loop stdin reader; an expired or
cancelled oneshot is discarded before later user input, and no detached blocking stdin task
survives cancellation.

Focused permission/Agent/CLI/TUI/Runtime/MCP/plugin tests passed before and after the deadline,
strict MCP proposal-hook and terminal cancellation corrections. The final exact-base command
`COLLABORATION_VALIDATION_BASE=d662501c94621f54066de2ebdc62840645d32b0f
./scripts/release_preflight.sh` passed outside the outer execution sandbox, including the macOS
Seatbelt tests that cannot nest inside that sandbox. Three existing CLI/config tests were made
deterministic by isolating their home/config paths after the sandbox exposed their dependence on
the executing user's home. No Dashboard file or I213 owner artifact is changed.

## 2026-08-23 Implementation Merge And Completion Checkpoint

PR #376 final exact head `aed71fb432086a20d1fdf2a927e0d7bf7b1f672c`, base
`d662501c94621f54066de2ebdc62840645d32b0f`, passed exact-head CI `32640691772` (5/5,
including the Windows workspace) and independent permission/security/API review `5386153429`.
Merge-time CAS confirmed the effective claim and Accepted ADR-067, stable head/base, CLEAN merge
state, no unresolved blocking feedback and the explicit serial pause of I213 / PR #372. PR #376
merged as `f9e6706d39a3c612061c6a1fb68e31bd24c29904`.

All published acceptance rows are satisfied by pre-existing implementation commit
`49d1546c3748930177655dbedc7f3665780d92ab`: one Agent-owned evaluator/final gate is used by the
in-scope surfaces; exact normalized authorization and revision/request/context CAS are enforced;
Once/Session/Deny, strict hooks, deadlines, cancellation, closure, resolver failure and stale
state fail closed; Runtime `ApprovalHandler`, serialized permission configuration and MCP
compatibility remain covered. No Dashboard/I213, TOOL-024, `/auto`, release or publication file
was changed.

The independent review recorded one non-blocking API residual: `AgentTool::project_input()` keeps
the complete input by default, so a third-party tool with secret-bearing arguments must override
that projection. PERM-006-E owns the later cross-surface documentation/conformance gate for this
contract. Legacy policy-aware wrappers remain test-only compatibility fixtures and are not
production-selectable. This closeout records status only and does not self-certify completion.
