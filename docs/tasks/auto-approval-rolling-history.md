# AUTO-HISTORY-001: Session-local rolling approval judgment history

Document status: Active / Claimed proposed via #916; ineffective until this governance record merges to main.
Deliverable: subsequent approval-model requests include bounded, sanitized previous model judgments
and attributable eventual human outcomes from the same live session, across every supported review family.
User-facing documentation: README.md and README.zh-CN.md auto-approval lifecycle and limits.
Source requirement: [durable intake](../proposals/session-auto-approval-rolling-history.md).

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | AUTO-HISTORY-001: bounded live-session model/human judgment history, provider request integration for all supported auto-review families, correlated fallback outcomes, isolation/redaction tests and documentation; excludes permission eligibility expansion, reusable grants, disk persistence, background continuation, archive recovery and unrelated TUI changes |
| Claimed At | 2026-11-11 |
| Source Issue | None; explicit session request recorded in durable intake |
| Governance Claim PR | #916 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requested completion of session-recovery handoff including this original requirement; independent security/design review required before activation and protected implementation delivery |
| Implementation PR | Not started |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Effective claim on main, bounded-history implementation across all supported review families, current-head security review and CI, documented runtime acceptance and existing implementation commit evidence |

## Constraints and selected design

Hard: policy Deny and explicit human checkpoints remain authoritative; history is untrusted
reference data, never a grant, instruction or substituted current-request evidence. Preserve all
current digest/revision/expiry checks, permission admission and sandbox boundaries. Do not store
hidden chain-of-thought, raw provider errors or secrets. No cross-session copying or persistence.

Soft design for review: eight entries, at most 1024 serialized UTF-8 bytes per entry and 8192
serialized bytes for the whole envelope. Enforce limits after JSON escaping. Evict oldest first.
Reset on resolver replacement, session/fork change or explicit auto reset. Generation-bound
monotonic correlation tokens ensure concurrent/delayed human results cannot update another entry
or resurrect an evicted entry. Snapshot prior history before inserting the current request.

Record sanitized tool/action context, public model effect summary/decision points and result,
plus separately attributed human Pending/Allowed/Denied/Cancelled/Unavailable. Cancellation/drop
must settle only its own pending token. Missing history or lock failure keeps existing safe
fallback behavior. History receives no additional tools, retries or model authority.

Read ADR-064 and its accepted successor decisions for each current review family, PERM-006,
PERM-007-H, auto_resolver and its action/network/invocation/file-mutation/prepared-command modules,
permission_pipeline, bounded_model, CLI/runtime composition and existing approval presentation.
Avoid public API breakage; any unavoidable change requires an ADR and migration plan first.

## Acceptance and validation

1. Captured actual provider requests include prior bounded history for ordinary invocation,
   shell/context/script, file mutation, built-in invocation/action, network and prepared command.
2. Real fallback oneshots update the exact entry with human allow/deny/cancel/unavailable;
   model/human attribution remains distinct. No pending or failed action is represented as allowed.
3. Concurrent calls, late outcomes, eviction, reset, rebind and session-ID reuse cannot cross-update
   or leak history. The current request is absent from its own prior-history snapshot.
4. Oversized Unicode/JSON-escaped inputs remain within count/per-entry/aggregate byte limits;
   sanitization excludes credentials/control sequences while retaining useful bounded context.
5. Hostile previous approvals do not bypass explicit Ask/Deny, eligibility, current-request
   digest/expiry/revision checks or execution admission. Empty/unavailable history is fail-safe.
6. Run pinned locked focused real-resolver/provider/fallback regressions, standard
   ./scripts/release_preflight.sh, applicable native Windows gate and exact-head independent
   security review. Document live-session lifecycle in both READMEs; do not close on ring-buffer
   unit tests alone. No default background continuation or release authorization is implied.

## Selection and ownership inventory

Preparation base: 6c81390a (current origin/main when isolated governance worktree was created).
I249 Planned retained; I277/I302/I318/I320 Review retained; I319/I322 Active retained.
No Blocked owner selected. Historical Complete headers with embedded older Review notes
(I146/I147/I150/I162) are not reopened. Existing open implementation PRs #861/#907/#912
own search/storage work and do not transfer authority here. Recovery emergency, background
continuation and approval-presentation owners retain their scopes and dirty changes.
This is a separate requirement; it does not replace any published iteration baseline.

## Local preparation and residuals

Governance-only isolated worktree; no implementation branch/commit/PR created for this scope.
PR-number backfill is recorded. Both governance validators, independent design/security review,
merge-time main/owner CAS and main merge remain before the claim becomes effective.
Implementation, tests, native acceptance and delivery commits remain unimplemented.
Residual destination: this owner. No Completion Commit exists; no Complete claim is made.
