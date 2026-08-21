# SOP: Documentation Sync Check

## Purpose

Keep documentation honest. Documentation drift—where docs claim a state the code or governance
owners do not have—is a correctness defect, not cosmetic.

> Originating lesson (I008): iteration, README, and roadmap marked self-evolution COMPLETE while the
> feature was not wired into the binary. Status claims must trace to runtime reality.

## When to Run

- Before marking any Story, iteration, or task Complete.
- Before merging a governance claim or closure.
- At the Session End Checklist.
- During a governance audit.

## Authoritative Status Sources

| Fact | Source of truth |
|---|---|
| Iteration state | Iteration owner document; `docs/iterations/README.md` mirrors it |
| Story/task scope and acceptance | Owner under `docs/backlog/active/`, `docs/iterations/`, or `docs/tasks/` |
| Collaboration ownership | Collaboration Claim in owner document on target branch |
| Open claim pending state | GitHub open PR only; never persisted as Claim State |
| User-visible behavior | Runtime evidence plus affected user documentation |
| Test count / overall status | Actual locked workspace validation output |
| Governance capability state | `.agent-governance/manifest.yaml` |
| Governance profile recommendation | `scripts/assess_project_scale.sh .` output |
| Public-facing status | `README.md` / `README.zh-CN.md` |

## Checklist

### Delivery And Completion

- [ ] README status agrees with current owner documents.
- [ ] README and README.zh-CN maintain bilingual parity for shared claims.
- [ ] Every Complete iteration has runtime evidence, not only unit tests.
- [ ] Every Complete iteration, Story, and long-task phase records
      `Completion Commit: <SHA>` in its owner document.
- [ ] Completion SHA identifies an already-existing implementation/evidence commit, not the
      documentation-only closure commit.
- [ ] Missing/malformed completion evidence keeps the owner Review, Partial, or Blocked.
- [ ] Published objectives, dependencies, exclusions, acceptance, validation, and docs targets
      remain visible.
- [ ] Non-infrastructure iterations identify runnable, testable deliverables and end-to-end evidence.

### Collaboration Claim

- [ ] `Claim Pending` is not persisted in an owner document.
- [ ] A Claimed/Handoff Pending/Closed record has Responsible Actor, Executing Agent, Work Slice,
      concrete dates, claim PR/commit, Authorization Mode, and Authorization Evidence.
- [ ] Governance Claim PR matches the actual `#NN`, or Direct commit references a real SHA.
- [ ] One owner document has at most one effective claim; parallel slices use child owners.
- [ ] Closed claim agrees with Complete or Cancelled delivery state.
- [ ] Grandfathered pre-adoption work is handled according to `AGENT-COLLABORATION.md` and is not
      retroactively blocked without a triggering new branch/PR or owner lifecycle change.
- [ ] Immediately before claim merge, target branch, overlapping PRs, claimant/scope, dependencies,
      exact-head CI, authorization, and review feedback were rechecked as the merge-time CAS gate.
- [ ] A new iteration's Claimed and Active state are proposed atomically and both are described as
      ineffective until target-branch merge; any separate activation PR has a recorded dependency reason.
- [ ] Single-maintainer merges record why independent review was unavailable and show exact-head CI
      plus both governance validators.
- [ ] Emergency overrides contain the minimum incident/security record and a reconciliation owner
      due within two business days.

### Inventories And Governance

- [ ] `docs/iterations/README.md` reflects every iteration.
- [ ] Active, Review, Planned, and Blocked iterations have current dispositions before new work is
      activated.
- [ ] Board mirrors owner documents and never substitutes for owner evidence.
- [ ] The implementation candidate was locally converged before first push; CI/review evidence is
      attached only to submitted stable heads and was refreshed after substantive changes.
- [ ] Review evidence names the exact head/base, covers the assigned risk surface and gives an
      explicit verdict; unrelated or generic Agent output is not treated as review.
- [ ] `.agent-governance/manifest.yaml` status, audit date, skill version/refresh, and next actions
      are current.
- [ ] `scripts/assess_project_scale.sh .` supports current profile/branch/worktree assumptions.
- [ ] Test counts cited in docs match actual output.
- [ ] No doc claims behavior contradicted by dead-code warnings or runtime absence.
- [ ] Referenced ADRs exist.

## Validation

Run both governance validators:

```bash
scripts/validate_project_governance.sh .
bash scripts/validate_collaboration_claims.sh .
```

The collaboration validator checks templates/SOP integration, persistent Claim State values,
required fields, authorization consistency, Closed/completion consistency, and changed active owners
that must migrate after adoption.

When profile, branch mode, worktree mode, or governance depth may have changed, also run:

```bash
scripts/assess_project_scale.sh .
```

A failing check, stale owner, invalid claim, missing completion commit, or unowned residual means
documentation is not in sync. Repair before claiming completion or merging ownership.
