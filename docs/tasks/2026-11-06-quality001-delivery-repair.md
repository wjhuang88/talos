# QUALITY-001: Recent Delivery Quality Repairs

> Document status: Active (proposed; effective only after claim PR #864 merges)
> Published plan date: 2026-11-06
> Deliverable: reproducible checkout regression corrected, large saved messages restored, usable bounded workspace validation, and accurate T5 Board status.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Repair the four findings from the 2026-11-06 delivery quality assessment: checkout conditional-config regression, host message restoration across large records, fixed validation workload budgets, and T5 derived Board synchronization |
| Claimed At | 2026-11-06 |
| Source Issue | None |
| Governance Claim PR | #864 |
| Authorization Mode | Independent review |
| Authorization Evidence | Session instruction requests fixing the assessed issues; independent governance review and exact-head CI required before activation |
| Implementation PR | Not started |
| Last Updated | 2026-11-06 |
| Handoff / Release Condition | Local regression and standard preflight pass, independent review and exact-head CI, then implementation evidence and owner closure |

## Baseline and inventory

Assessment baseline: `35d7264b2b2689b26610bee7dda6b04ac4895614`.
Standard preflight passed governance, formatting, check and Clippy, then failed
the conditional-checkout test at checkout.rs:276 (1122 tools tests passed).
The same workspace-built binary fails independently on Linux/Git 2.56.0.
This does not establish an authorization bypass; reconcile the test with the
already implemented target-configuration discovery before changing behavior.
Normal storage writes and legacy reads a 2097153-byte record; paged history
returns RecordTooLarge. Workspace validation uses verbose Cargo tests but caps
each stream at 16 KiB and each command at 30 seconds. T5's owner is Complete
at `322574bc9707492ea6e9d5232794f73e687071d9`, while its Board row is stale.

Inventory of nonterminal iteration headers: I310 Active (Gemini, retain), I317
Active claim merged through #851 (search helpers, retain), I302 Review (Windows
handles, retain), I277 Review (deferred device/human acceptance, retain), I249
Planned (dependency pilot, defer), I164 Paused (superseded layout, retain).
No Blocked header found; all other inspected owners are terminal. Open PRs #838,
#849, #860 and #861 concern Gemini/search and do not own this repair. Recheck
overlap before merging. This independent repair task does not reopen completed
I308/I313/I315 or change their published historical baselines.

## Scope and acceptance

1. Reconcile checkout creation and existing-branch behavior with current target
   configuration admission. Test safe creation and real target-hook refusal;
   preserve exact permission and no-mutation-on-refusal assertions.
2. Restore writer-produced TLOG and JSONL messages larger than a source page
   through the host message scan. Preserve bounded I/O steps, cancellation,
   source-version validation, legacy compatibility and no partial publication.
   Model history indexing/disclosure keeps its separate bounded authority.
   No on-disk migration, format change or whole-file fallback.
3. Make fixed Cargo validation appropriate for real workspaces with finite time
   and retained output limits. Preserve ADR-097 overflow-as-failure, cleanup,
   cancellation and exact admission. Validate actual Cargo execution, not just
   a fake executor. No model-selected arbitrary command or timeout authority.
4. Synchronize the T5 Board row from its existing owner evidence. Record the
   reusable lessons and update user-facing/SDK documentation for changed limits.
5. Focused regression checks and pinned locked standard preflight pass. Obtain
   independent security/API review and exact-head CI before merge; preserve
   unexecuted native platform checks as explicit residuals until CI provides them.

## Constraints, validation and closure ledger

Hard: permission pipeline, bounded process ownership, unchanged storage format,
separate model-disclosure authority, pinned toolchain and locked dependencies.
Soft: fixed validation timeout/output budgets; record chosen values and rationale.
Assumption: the checkout failure is a stale test; verify rather than assume.

Artifacts: Session scan/parser and regressions; validation profiles and real
Cargo fixture; checkout regression if warranted; Board, SDK/CLI documentation,
EVOLUTION and this owner. Preserve unrelated sources and completed owner history.
Run focused feature-enabled tests, both governance validators and
`./scripts/release_preflight.sh`. Windows tests require xwin/MSVC per LOCAL-DEV
and native CI; no Linux result represents native Windows execution.
Residual destination: this owner. Keep delivery Review until actual implementation
commit evidence and required review/CI exist. Roll back only this repair candidate
if acceptance fails; do not suppress tests or widen permission policy.
