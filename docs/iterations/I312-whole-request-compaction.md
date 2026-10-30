# Iteration I312: Whole-Request Compaction And Manual Controls

> Document status: Planned
> Published plan date: 2026-10-30
> Planned objective: deliver full handoff T4 through MEM-005 policy and MEM-003 convergence, preserving the later T5 Actor recovery deliverable.
> Baseline rule: preserve all parent acceptance; successful trimming alone cannot close this iteration.
> MVP deliverable: a long-running session compacts complete provider requests safely, exposes numeric outcomes and accepts a typed /compact command while raw history remains recoverable.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Handoff T4: whole-request pressure/admission, shared structured checkpoint generation, raw-history-preserving projection, manual command, observable outcomes and long-session regression |
| Claimed At | Not applicable |
| Source Issue | #390 (T4 subset; T5 remains outstanding) |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Preparation only; independent API/privacy review and atomic activation required |
| Implementation PR | Not started |
| Last Updated | 2026-10-30 |
| Handoff / Release Condition | MODEL-014 acceptance closeout and effective target-branch claim before implementation |

## Published Baseline

Selected owners: MEM-005 (eleven acceptance rows) and MEM-003 (five rows).
MEM-002 is Complete. I298/MEM-005-A remains completed bounded request recovery,
not authority or evidence for this new semantic compaction scope. Issue #390's
T4/T5 allocation is explicit in the proposed design; T4 alone cannot close #390.

### Scope and acceptance

1. One existing request-budget authority covers initial, continuation and summary
   calls, including stable prompt, schemas, images, input margin, output and
   applicable reasoning reservations. Explicit config/catalog/fallback precedence
   remains shared with MODEL-014; unknown usage is not zero usage.
2. Central pressure policy separates advisory from automatic critical and emergency
   handling. Evaluate before provider dispatch; post-turn maintenance marks need
   without mutating a running tool batch. Fixed-overhead overflow refuses safely.
3. One bounded structured checkpoint pipeline replaces old generic-summary
   semantics. Preserve goal, constraints, decisions, progress, facts/assumptions,
   resources, failed attempts, unresolved work and next step with raw provenance.
4. Oversized text and tool history, including restart, uses bounded sequential
   folding under the same admission/schema/privacy rules. Intermediate failure,
   invalid output, timeout or cancellation never commits a partial projection.
5. Keep raw Actor/durable history authoritative. Checkpoints are derived in-memory
   state with generation, source revision/range and lineage. Resume/fork reconstruct
   safely; model changes invalidate stale projections. No second transcript store.
6. Preserve recent complete exchanges, current instructions, stable prefix and
   tool pairing. Exclude private arguments/reasoning and UI/export-only raw output
   from summaries; user-visible diagnostics contain no checkpoint content.
7. Register /compact through CMD-001 BuiltinCommand and typed Session operations.
   Idle executes; active turns defer to a complete-batch boundary. Bound pending
   work, duplicate receipts and expiry. Shutdown/cancel/generation changes yield
   exactly one terminal outcome and cannot apply stale results.
8. Visible transient started/applied/skipped/failed/cancelled status includes
   whole-request before/after estimates and bounded reason codes, never durable
   fake user messages or display-history rewrites.
9. Reject empty/error-only/partial-error/failed-terminal/EOF/oversized/unknown-schema
   or insufficient-reclamation checkpoints atomically. A failed attempt falls back
   only to a safely admitted projection or explicit capacity refusal.
10. Real Actor and manual-command walkthroughs plus a 50-turn heavy-tool and
    text-heavy restart fixture prove bounded requests and retained task semantics.
    Repeated checkpoints retain recoverable original raw history and lineage.
11. Complete pinned locked preflight, independent API/privacy/security review,
    exact-head CI, merge CAS and owner-first implementation evidence.

### Non-goals and ownership

T5 retains stable model-callable tool discovery, voluntary semantic checkpoint
selection and bounded current-branch SessionIndex/history recovery. MEM-007/#143
retain pre-entry shell/tool output filtering. ARCH-006 stable prefix and SESSION-011
truth/migration boundaries remain intact. No paid probes, new database, dependency,
raw transcript migration, release/tag or claim to complete those separate owners.

### Design and compatibility gate

[Proposed design](../proposals/t4-whole-request-compaction-design.md) defines the
central thresholds, schema bounds, folding limits, privacy input and operation
lifecycle. [ADR-096](../decisions/096-whole-request-checkpoint-boundary.md) proposes the
SessionOp enum compatibility/migration and checkpoint-context boundary; it requires
independent review and target-branch activation before implementation. Proposed constants require
long-session evidence; changing measurements must not shrink semantic acceptance.

### Planned validation

Run focused checkpoint schema/renderer/pressure/privacy tests, request-capture
admission tests for every call path, Actor ordering/cancel/dedup tests and real
/compact walkthrough. Include text-heavy over-budget restart, early constraints
and failed attempts, repeated lineage, hidden output and missing usage fixtures.
Run `./scripts/release_preflight.sh` with Rust 1.99.0 and locked workspace commands,
both governance validators, then independent review and exact-head platform CI.
No preceding-head check certifies this implementation.

### Documentation targets

MEM-005/MEM-003 owners, CLI command reference, context/memory behavior reference,
README en/zh where commands are listed, Runtime SDK enum migration, iteration
index, Board and full handoff. Preserve original plans and historical failures.

### Risks and rollback

Generated summaries may omit or misinterpret facts: retain raw evidence, source
ranges, recent verbatim exchanges and bounded recovery ownership. Rollback removes
derived checkpoint application without deleting raw history. Never compensate for
failed compaction by replaying tools or dispatching an over-budget request.

## Preparation inventory — 2026-10-30

Observed against main `0469f8041ed0d24bb2dffb465e9ff7fa3609a034`:
I311 completion is proposed in this governance batch after #816 merged; activation
remains ineffective until the finalized claim merges.
I293/I294 retain separate search scopes (#808/#814); I277 retains deferred human
Review; I302 retains diagnostic Review; I249 stays deferred Planned; I164 remains
Paused with superseded layout in I165. Merged #811 activates independent Gemini I310/ADR-094 at
`80b9393375812475901b1b9565cbff6fca273537`; preserve that distinct scope. Terminal owners remain terminal. Recheck target owners and overlap
immediately before proposing activation; this inventory grants no ownership.

## Execution and completion evidence

None. Planned/unclaimed preparation only. No production implementation authorized
or claimed. Overall handoff remains Active through T4 → T5 → T1 → T6.

## Preparation closure ledger

Requested outcome: effective T4 activation after I311 acceptance closeout.
Artifacts: I312, proposed ADR-096/design, MEM-005/MEM-003 reconciliation, indexes,
Board and handoff. Preserve original owner acceptance and independent T5 scope.
Required evidence: independent design/API/privacy review, both governance
validators, exact-head CI and CAS. Current evidence: preparation only; none of
these drafts authorizes production code. Residuals remain I312; full queue remains
in the handoff owner. Activation/package synchronization is not yet complete.

## Preparation design review — 2026-10-30

Independent Agent static review accepts the design direction, with no remaining
architectural blocker; this is not immutable-head approval or activation. It ran
no tests and shares the external account identity. Implementation must additionally
prove bounded input scanning/temporary allocation (stream large entries), final-
only whole-request fit/reclamation (intermediate folds validate their own request
and schema), exactly-once terminal outcomes under cancellation/application races,
and successful long-session fixtures within the proposed 64-call/120-second
bounds. Safe refusal alone cannot satisfy semantic long-session acceptance.
