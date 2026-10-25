# SANDBOX-001: Bubblewrap Workspace Visibility

**Status**: Active
**Prior Implementation Evidence:** 3beb011a77e0cc80422d915443e6caf9f8f74742
**Type**: Defect
**Source Issue**: #757
**Parent**: None; prerequisite for I300 Linux validation, not Git scope expansion
**Selected Iteration**: [I307](../../iterations/I307-bubblewrap-workspace-visibility.md)

## Reopened Isolation Defect — 2026-10-26

The prior Complete state is withdrawn: broad or aliased workspace/read-overlay
roots can cover protected /tmp, /dev or /proc mounts after the I307 reorder.
Native narrow-root success does not validate these inputs. I307 retains the same
objective and acceptance; its follow-up requires canonical overlap rejection,
non-execution/sentinel regressions and independent native Linux security evidence.
No follow-up source implementation exists yet; governance reactivation precedes it.

## Requirement

An authorized Linux workspace below /tmp must stay visible and be the actual
working directory. Host temporary siblings and writes outside the workspace
retain isolation. Extra read paths remain read-only; a missing workspace must
not cause execution in HOME.

I307 owns full scope, acceptance, validation, documentation and claim.
The governance record reached main through #758 before implementation began.
User documentation target: crates/talos-sandbox/README.md.
PR #759 merged with exact-head CI 40113538524 successful, independent security
review ACCEPT and both governance validators passing. Native Linux validates
all six I307 acceptance rows. This does not close T7 or I161 residuals.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | SANDBOX-001: Linux Bubblewrap workspace mount order, explicit cwd, isolation tests and docs; no Git admission, fallback policy, dependencies or API changes. |
| Claimed At | 2026-10-24 |
| Source Issue | #757 |
| Governance Claim PR | #761 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested session-handoff and failing-CI closure; no distinct maintainer reviewer available. Independent Agent security review, exact-head CI, both validators and merge-time CAS remain mandatory. Prior claim #758 and implementation #759 are historical; this follow-up claim is ineffective until its governance merge. |
| Implementation PR | None; follow-up pending |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Reopened for protected-mount overlap correction; no implementation before effective follow-up claim. No release authorization. |
