# SANDBOX-001: Bubblewrap Workspace Visibility

**Status**: Complete
Completion Commit: f643d87759189eed9d14cef965b7e9260a51d1ab
**Prior Implementation Evidence:** 3beb011a77e0cc80422d915443e6caf9f8f74742
**Type**: Defect
**Source Issue**: #757
**Parent**: None; prerequisite for I300 Linux validation, not Git scope expansion
**Selected Iteration**: [I307](../../iterations/I307-bubblewrap-workspace-visibility.md)

## Follow-up completion — 2026-10-26

PR #762 merged as `f643d87759189eed9d14cef965b7e9260a51d1ab` after exact-head
CI 40213017639 passed all macOS/Linux/Windows jobs, independent Agent security
review approved `48a6285de`, both validators passed and merge CAS remained valid.
Inspected Linux logs prove all five isolation regressions plus 37 sandbox unit
tests/two doctests and 112 Desktop tests, including Auto/cancellation. The
[I307 owner](../../iterations/I307-bubblewrap-workspace-visibility.md) records
full evidence and preserves the historical reopened finding below. No I307 item
is deferred; T7 still requires integration and final validation.

## Reopened Isolation Defect — 2026-10-26

The prior Complete state is withdrawn: broad or aliased workspace/read-overlay
roots can cover protected /tmp, /dev or /proc mounts after the I307 reorder.
Native narrow-root success does not validate these inputs. I307 retains the same
objective and acceptance; its follow-up requires canonical overlap rejection,
non-execution/sentinel regressions and independent native Linux security evidence.
Follow-up claim #761 is effective on main `d8da359ad`. A local implementation
now checks canonical workspace/read roots against lexical and canonical protected
mounts, with 16 rejection combinations and an ordinary-alias success regression.
macOS sandbox tests pass (41 unit tests, two doctests); native Linux execution,
final security review and implementation merge remain pending. Full standard
preflight and both governance validators passed; implementation `51245b28c`
is a local candidate, not completion evidence.

## Requirement

An authorized Linux workspace below /tmp must stay visible and be the actual
working directory. Host temporary siblings and writes outside the workspace
retain isolation. Extra read paths remain read-only; a missing workspace must
not cause execution in HOME.

I307 owns full scope, acceptance, validation, documentation and claim.
The governance record reached main through #758 before implementation began.
User documentation target: crates/talos-sandbox/README.md.
PR #759 merged with exact-head CI 40113538524 successful, independent security
review ACCEPT and both governance validators passing. The prior native Linux
evidence covered the narrow-workspace cases; it did not cover the reopened broad-root defect. This does not close T7 or I161 residuals.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | SANDBOX-001: Linux Bubblewrap workspace mount order, explicit cwd, isolation tests and docs; no Git admission, fallback policy, dependencies or API changes. |
| Claimed At | 2026-10-24 |
| Source Issue | #757 |
| Governance Claim PR | #761 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested session-handoff and failing-CI closure; no distinct maintainer reviewer available. Independent Agent security review, exact-head CI, both validators and merge-time CAS remain mandatory. Prior claim #758 and implementation #759 are historical; follow-up claim #761 became effective on main before implementation #762. |
| Implementation PR | #762 (merged as f643d87759189eed9d14cef965b7e9260a51d1ab) |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Protected-mount acceptance complete at #762; I300 retains its own integration and final validation. No release authorization. |
