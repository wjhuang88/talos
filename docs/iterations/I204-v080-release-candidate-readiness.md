# Iteration I204: v0.8.0 Release-Candidate Registry Readiness

> Document status: Planned
> Published plan date: 2026-08-16
> Planned objective: validate a candidate v0.8.0 workspace and produce a reviewed GO/NO-GO packet before I203 release execution.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a reproducible candidate-only version-aligned package/dry-run matrix and reviewed GO/NO-GO packet.
> Activation rule: I204 is not implementation authority until its claim is effective on main and activation inventory is appended.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release-governance session` |
| Work Slice | `ARCH-031-E / I204` only: candidate v0.8.0 version alignment, registry visibility, metadata closure, package/dry-run evidence and reviewed GO/NO-GO packet. No tag, GitHub Release, real Cargo publication or I203 implementation. |
| Claimed At | 2026-08-16 |
| Source Issue | None |
| Governance Claim PR | #257 |
| Authorization Mode | Independent review |
| Authorization Evidence | I162 closeout is effective at `main@9fc2c7f1`; this claim remains ineffective until independent exact-head review, CI and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-08-16 |
| Handoff / Release Condition | Start from current main, preserve all I162 Published Baselines, and keep I203 Blocked until this iteration produces a reviewed GO. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `ARCH-031-E` | `ARCH-031` | Planned | I162 Complete/Closed reviewed NO-GO; registry/network access | Candidate v0.8.0 readiness packet without irreversible release actions |

## Authorized Scope

- candidate-only version alignment and migration evidence;
- locked metadata closure and registry visibility;
- package/dry-run matrix for the authorized closure;
- external registry-mode fixture and reviewed GO/NO-GO packet.

## Forbidden Changes

- no real publication, tag, GitHub Release, or I203 activation;
- no runtime behavior/API implementation;
- no `talos-models` publication or guard weakening outside explicit candidate evidence.

## Non-Terminal Inventory At Selection

I162 is Complete/Closed with reviewed NO-GO; I203/REL-003 is Blocked/Unclaimed; I188/I189/I195/I196
remain Planned/Claimed and unactivated; I164 remains Paused. No Active or Review implementation
iteration is imported into I204.

## Acceptance

- candidate `0.8.0` manifests and internal dependencies resolve in an isolated worktree;
- registry API/search and compatible package versions are independently verified;
- metadata-derived closure and package/dry-run results are complete;
- external registry-mode SDK/CLI fixtures pass;
- reviewed GO/NO-GO packet is explicit and I203 remains blocked on NO-GO.

## Validation

Run locked governance, metadata, package/dry-run, fixture, formatting and workspace validation. A
registry/network failure remains a named NO-GO blocker. No real publish or tag is permitted.

## Completion Evidence

- Completion Commit: pending
- Keep Review/Blocked if registry, candidate package, fixture or independent review is incomplete.
