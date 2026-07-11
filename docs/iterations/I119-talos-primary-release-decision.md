# Iteration I119: Talos-Primary Release Decision

> Document status: Planned
> Published plan date: 2026-07-12
> Planned objective: Produce reproducible Talos-primary development evidence and make an honest
> evidence-based pre-1.0 or v1.0 release decision.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: two bounded tasks have replayable Talos-primary evidence, followed by a dated
> REL-002 audit, release decision, and final handoff.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| N130 | REL-002 | Planned | I116-I118 Complete | Two sole-primary Talos task packets |
| N131 | REL-002/GOV-003 | Refinement | N130 | Replayable validation/permission/git/issue evidence |
| N132 | REL-002 | Planned | N130-N131 | Independent dated criterion audit |
| N133 | Release | Planned, conditional | N132 | Pre-1.0 release or explicitly approved v1.0 only if fully qualified |
| N134 | Four-month closeout | Planned | N132-N133 | Final matrix and next-owner handoff |

### Scope

- Talos binary is the sole primary planner/executor for selected bounded work.
- Capture immutable session, permission, validation, git, failure/recovery, and issue-sync evidence.
- Audit REL-002 without treating test count, external-agent work, or governance mutation alone as
  self-bootstrap qualification.

### Non-Goals

- Predetermined v1.0 outcome, hidden external-agent authorship, automatic release, crates.io publish,
  or lowering any permission/release gate to manufacture qualifying evidence.

### Acceptance

- Given each selected task, when evidence is replayed, then Talos—not an external agent—authored the
  plan/implementation/validation loop and all external intervention is disclosed.
- Given the REL-002 matrix, when any criterion lacks direct evidence, then the verdict remains NO-GO.
- Any release tag matches synchronized manifests, passes release preflight, and has explicit user
  authorization.

### Planned Validation

- `./scripts/release_preflight.sh`
- `scripts/talos_smoke.sh` plus task-specific real-binary validation
- evidence schema/replay audit and governance validation
- release preflight, installer smoke, and tag/version check when authorized

### Documentation To Update

- REL-002 owner/readiness report, release notes, README status, Board/backlog, final handoff

### Risks And Rollback

- Risk: external tools remain necessary for authorship or recovery.
- Rollback: classify the packet non-qualifying, preserve NO-GO, and publish only pre-1.0 if approved.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-12 | Planning | Published as Month 4 shell; activation waits for I118 Complete and explicit task selection. |
