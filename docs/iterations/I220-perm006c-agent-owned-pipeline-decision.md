# Iteration I220: Agent-Owned Permission Pipeline Decision

> Document status: Active / Claimed proposed by PR #370; ineffective until target-branch merge
> Published plan date: 2026-08-23
> Planned objective: decide the single-owner permission orchestration, final-decision hook,
> compatibility and migration contract required before PERM-006-C implementation.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently reviewed current-path matrix and Accepted ADR-067 make a
> separate PERM-006-C implementation iteration runnable without changing executable behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline governance session 2026-08-23 |
| Work Slice | Decide only the PERM-006-C prerequisite contract: inventory current CLI/TUI/print/inline/Runtime/MCP permission authorities; define Agent-owned orchestration, bounded approval resolver authority, authoritative normalized input, final `AfterPermissionCheck` semantics, concurrency/deadline/cancellation/fail-closed behavior, additive public compatibility, migration, rollback and the separate I221 implementation boundary. No Rust/Cargo/dependency, permission behavior, hook implementation, wrapper removal, background process, `/auto`, Dashboard, Desktop, release or publication change. |
| Claimed At | 2026-08-23 |
| Source Issue | #55 |
| Governance Claim PR | #370 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorization in the active session on 2026-08-23 explicitly permits I213 and decision-only I220 to run in parallel under the non-overlap, shared-file stop and merge-time CAS contract. Exact-head CI `32616966092` and independent Agent-role approval `5384136045` are bound to candidate head `434de513d5dd088b1b354e8a3e05ca869a425f01`; merge-time CAS remains required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | I213 owner-first drift was repaired by PR #371; merge-time CAS must re-confirm `main@836b1714` or a descendant. After merge, produce only ADR-067 and its evidence; I221 needs a later effective claim before any implementation. |

## Published Baseline

Planning target: `main@6fbb5550bc3b4b7b5827b77bc57a152d0636c339`.

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I220 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore. |
| I197 | Review | Preserve TUI-059 corrective owner and deferred acceptance. |
| I198 | Review / Claimed | Preserve SKILL-005 residual and do not reopen implementation. |
| I201 | Review / Claimed | Preserve TUI-058 corrective owner and deferred acceptance. |
| I206-I208 | Planned / Unclaimed | Preserve the ordered steering sequence; do not activate. |
| I210 | Review / Claimed | Preserve TUI-060 corrective owner and deferred acceptance. |
| I213 | Active / Claimed | Continue only in its Dashboard observation owner. The prior I219 exception is not reusable; I220 requires an explicit new non-overlap authorization or must wait. |
| I220 | Active / Claimed proposed by PR #370 | Decision-only claim is proposed; no authority exists before finalized claim/activation merge. |

Open PRs #120/#121 are archival Drafts and remain untouched. Fresh remote inventory found no open
I220, PERM-006-C or permission-pipeline implementation PR. I213 owns Dashboard observation only,
and the maintainer has now explicitly authorized this non-overlapping decision-only I220 pair.
The claim remains ineffective until merge-time CAS succeeds.

I213's owner-first record was repaired by PR #371 and is effective on `main@836b1714`;
activation PR #363 merge `e578f419` is now consistently recorded. I220 does not modify the
Dashboard owner or `crates/talos-dashboard/**`.

### Dependency And Decision Gate

PERM-006-A/I189 and PERM-006-B/I219 are Complete/Closed. ADR-065 explicitly deferred the
PERM-006-C hook transport/version migration, and the current paths still have multiple permission
authorities and pre-final-decision hook timing. ADR-067 is therefore a required prerequisite, not
an implementation detail.

The decision must freeze:

- the Agent as sole permission orchestration authority while composition roots inject one Session
  state, explicit context, optional bounded approval resolver and trusted grant source;
- normalization/validation before request construction, with the same authoritative normalized
  input approved and executed and no permission-relevant post-approval mutation;
- `AfterPermissionCheck` as the final execution-gating Allow/Deny exactly once;
- resolver scope limited to Once, Session or Deny, with no policy evaluation, grant compilation,
  authorization issuance or execution authority;
- fail-closed no-resolver, headless, channel-close, error, timeout and cancellation behavior;
- concurrent approval, revision-CAS, admission, redaction and cancellation ordering;
- additive/source-compatible Agent and Runtime constructors, preserved Runtime
  `ApprovalHandler`, and an explicit MCP public API migration/removal gate;
- separate sandbox fallback authority, shared TUI Session transition state and `/attach`'s existing
  exact proposal/admission path; and
- one cross-surface I221 migration covering terminal, print/headless, TUI, inline/RPC, embedded
  Runtime and standalone MCP without leaving alternate policy evaluators.

### Scope And Non-Goals

I220 creates a current-path/authority matrix, ADR-067 and a runnable I221 implementation boundary.
It does not change Rust, Cargo, dependencies, persistence, config, permission decisions, hooks,
wrappers, execution, background jobs, `/auto`, Dashboard/Desktop, release or publication behavior.

PERM-006-D/E, Issue #188 behavior, TOOL-024-B/C/D and Issue #59 implementation remain separate and
unauthorized. I220 cannot be reused as their implementation iteration.

### Acceptance And Validation

- The current-path matrix covers terminal, print/headless, TUI, inline/RPC, embedded Runtime and
  standalone MCP, including wrapper/state ownership and hook ordering.
- ADR-067 fixes every decision gate above, public compatibility, migration, rollback and I221's
  exact cross-surface deliverable.
- Independent exact-head permission/security/API review validates the complete matrix and ADR.
- Both governance validators pass against the explicit target base; manifest YAML parses;
  `git diff --check` and EOF checks pass.
- The decision PR contains no Rust/Cargo/dependency or executable behavior change.

### Risks And Rollback

- Risk: a partial migration leaves two permission authorities or hooks observe a decision different
  from the one that gates execution.
- Risk: approval acts on projected or stale input while execution uses a mutated original.
- Risk: a source-breaking public hook/MCP change is hidden inside implementation.
- Rollback: reject ADR-067 and leave PERM-006-C/I221 blocked; current executable behavior remains
  unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-23 | Selection prepared | A/B dependencies are Complete. Read-only current-path analysis found ADR-067 mandatory before C implementation; I220 is decision-only and requires no Rust/Cargo behavior change. |
| 2026-08-23 | Claim proposed | PR #370 backfills the actual governance claim and proposes Active I220. Maintainer authorization permits the I213/I220 non-overlap, but the claim is ineffective until merge. |
| 2026-08-23 | Owner-drift gate | I213's iteration owner still states that merged activation PR #363 is ineffective. Dashboard owner-first reconciliation remains required before I220 merge-time CAS; this lane does not edit the Dashboard owner. |

## Verification Evidence

Pending finalized claim, claim merge, decision evidence, exact-head CI and independent review.

## Completion Evidence

- Completion Commit: pending
- A later status-only closeout cannot certify itself; it must cite the pre-existing reviewed ADR
  and matrix commit.

## Variance And Residuals

I221 owns PERM-006-C implementation after ADR-067 acceptance and a separate effective claim.
PERM-006-D/E, `/auto`, TOOL-024-B/C/D and Issue #59 closure remain later governed work.
