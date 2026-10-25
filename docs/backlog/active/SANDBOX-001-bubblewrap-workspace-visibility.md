# SANDBOX-001: Bubblewrap Workspace Visibility

**Status**: In Progress / proposed activation in #758; ineffective until merge
**Type**: Defect
**Source Issue**: #757
**Parent**: None; prerequisite for I300 Linux validation, not Git scope expansion
**Selected Iteration**: [I307](../../iterations/I307-bubblewrap-workspace-visibility.md)

## Requirement

An authorized Linux workspace below /tmp must stay visible and be the actual
working directory. Host temporary siblings and writes outside the workspace
retain isolation. Extra read paths remain read-only; a missing workspace must
not cause execution in HOME.

I307 owns full scope, acceptance, validation, documentation and claim.
No implementation is authorized until that governance record reaches main.
User documentation target: crates/talos-sandbox/README.md.
No completion evidence exists. This does not close T7 or I161 residuals.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | SANDBOX-001: Linux Bubblewrap workspace mount order, explicit cwd, isolation tests and docs; no Git admission, fallback policy, dependencies or API changes. |
| Claimed At | 2026-10-24 |
| Source Issue | #757 |
| Governance Claim PR | #758 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested session-handoff and failing-CI closure; no distinct maintainer reviewer available. Independent Agent security review, exact-head CI, both validators and merge-time CAS remain mandatory. Ineffective until merged to main. |
| Implementation PR | #759 |
| Last Updated | 2026-10-24 |
| Handoff / Release Condition | Governance-only preparation; implement only after effective claim. No release authorization. |
