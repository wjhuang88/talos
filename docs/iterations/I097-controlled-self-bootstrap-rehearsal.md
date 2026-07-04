# Iteration I097: Controlled Self-Bootstrap Rehearsal

> Document status: Planned
> Published plan date: 2026-07-04
> Planned objective: attempt one controlled Talos-primary documentation-only self-bootstrap
> rehearsal, or record non-qualification honestly.
> Baseline rule: preserve this target; changed targets use a new iteration ID.
> MVP deliverable: REL-002 evidence with explicit primary-executor boundary.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `REL-002` | v1 self-bootstrap release gate | Planned — not ready | I095/I096 readiness | Qualifying or non-qualifying rehearsal evidence. |
| `RUNTIME-001` residual | Embeddable runtime | Complete pre-1.0 facade | I095 | Runtime use boundary recorded. |
| `GOV-003` residual | Built-in governance | Phase 1 complete | I096 | Governance sync boundary recorded. |

### Scope

- Attempt a documentation-only owner-doc update with Talos as primary runtime if prerequisites are
  ready.
- Label every Codex or human intervention as review/fallback.
- Record validation evidence and docs sync.

### Non-Goals

- No `v1.0.0` claim from a single rehearsal.
- No release tag, publish, GitHub Release, or issue automation.
- No code-feature implementation unless a new iteration is created.

### Acceptance

- Given a rehearsal is attempted,
  When the evidence record is complete,
  Then REL-002 qualification is honest and primary-executor boundary is explicit.
- Given Codex remains primary,
  When the iteration closes,
  Then the record is explicitly non-qualifying.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
