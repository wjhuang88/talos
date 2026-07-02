# Iteration I083: Frontline Month 4 — Ecosystem Compatibility And Release Posture

> Document status: Planned
> Published plan date: 2026-07-02
> Planned objective: Execute weeks 13-16 of the 2026-07-02 frontline plan: opt-in shared skills
> policy/implementation, REL-002 rehearsal evidence, command/help/docs consistency, release posture,
> final closeout, and handoff.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the delegated cycle closes with ecosystem compatibility decisions, honest
> pre-1.0 posture, and residual owners for the next cycle.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| F130 | AGENT-002-B | Research | SKILL-002/ADR-022 | Opt-in shared skills ADR/policy |
| F131 | AGENT-002-B | Research | F130 | Optional shared skills discovery if cleared |
| F132 | REL-002 | Planned | GOV-003 progress | Self-bootstrap rehearsal packet |
| F133 | CMD/CONF/GOV/WEB docs | Mixed | Prior months | Command/help/docs consistency sweep |
| F134 | Release posture | Planned | all tracks | Pre-release posture report |
| F135 | Frontline plan | Planned | F134 | Final closeout matrix |
| F136 | Frontline plan | Planned | F135 | Final handoff |

### Scope

- Decide and possibly implement opt-in shared `~/.agents/skills` discovery.
- Record one Talos-assisted rehearsal with exact limits.
- Consolidate user-facing command and release posture docs.
- Close the delegated plan with validation evidence and residual owners.

### Non-Goals

- No shared `~/.agents/models.json` import.
- No shared `~/.agents/mcp.json` import.
- No automatic shared skills loading.
- No v1.0 claim.
- No release tag or crate publish unless separately approved.

### Acceptance

- Given shared skills are enabled by explicit config, when discovery runs, then Talos-owned skills
  take precedence and prompt budgets are preserved.
- Given a REL-002 rehearsal is recorded, when reviewed, then it clearly states whether Talos was
  the primary executor or only a helper.
- Given closeout completes, then validation, docs, release posture, and residual owners are
  recorded without hidden publish/v1 claims.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-skill -p talos-agent` if shared skills are implemented
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `scripts/check_publish_guard.sh .`
- `scripts/validate_public_site.sh` when `site/` changes

### Documentation To Update

- `docs/backlog/active/AGENT-002-dotagents-protocol-support.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- README/site command and release posture docs
- New closeout and handoff references under `docs/reference/` or `docs/tasks/`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: shared skills discovery changes prompts unexpectedly. Rollback: keep F131 behind a config
  flag and close with ADR/policy only.
- Risk: rehearsal evidence overclaims autonomy. Rollback: label it as non-qualifying REL-002
  evidence and record exact executor boundary.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-02 | Planning | Created as Month 4 shell for the frontline development plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
