# Iteration I226: Windows Job Object Process-Tree Ownership

> Document status: Active / Claimed
> Published plan date: 2026-08-25
> Planned objective: implement ADR-068's assigned-before-exec Windows Job Object boundary for TOOL-024-D1-B.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: an authorized Windows background command is owned by a Job Object before resume, with bounded descendant cleanup and real Windows evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session |
| Work Slice | TOOL-024-D1-B Windows launcher, Job Object ownership, allowlisted stdio inheritance, fail-closed cleanup and focused Windows tests only. |
| Claimed At | 2026-08-25 |
| Source Issue | #59 |
| Governance Claim PR | #393 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-068 accepted on `main@93ee3253`; I225 decision evidence `fca45c46` / CI `32797375011` / review `5404361120`; closeout review `5405268380`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-25 |
| Handoff / Release Condition | Claim effective on `main@d1f2a126` after PR #393 merge; implementation begins from this merge or later. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-D1-B | TOOL-024 / Issue #59 | Ready / Unclaimed | I225 Complete; ADR-068 Accepted | Windows Job Object launcher with real child/grandchild and handle-isolation evidence. |

### Scope

- Implement ADR-068 in the minimum Windows production boundary and integrate the existing supervisor.

### Non-Goals

- D2 CLI/TUI, I223 validation, Dashboard, permissions, `/auto`, persistence, Unix behavior, release and Desktop.

### Acceptance

- Given a permitted Windows background command, when launched, then Job assignment precedes thread resume and descendants are owned.
- Given any ownership setup uncertainty, when launch is attempted, then it fails closed with no leaked child or handle.
- Given concurrent launches, when a child inspects inherited handles, then only required stdio is visible.

### Planned Validation

- Focused Windows tests plus full locked workspace validation and release preflight.
- Exact-head CI and independent process/unsafe/API review.
- Real Windows marker, descendant, cancellation, timeout, shutdown and concurrent-handle fixtures.

### Documentation To Update

- TOOL-024-D1-B owner, TOOL-024 parent, Issue #59 long task, Board and iteration index.
- Directly affected user/API documentation only; D2 help and TUI projection remain separate.

### Risks And Rollback

- Risk: any pre-assignment execution or broad handle inheritance violates ADR-068.
- Rollback: retain `background_process_tree_unsupported` on Windows and remove the private launcher feature gate.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-25 | Atomic claim + activation | PR #393 exact head `2905c99d`, CI `32810069430`, independent approval `5405428154`, merge-time CAS and merge `d1f2a126`; implementation starts from this merge. |

## Verification Evidence

- Pending effective claim and implementation candidate.

## Completion Evidence

- Completion Commit: Pending.

## Variance And Residuals

- D2 and I223 remain separately governed and are not activated by I226.

## Retrospective

- Pending implementation.
