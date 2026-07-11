# Iteration I089: Ecosystem, Self-Bootstrap, And Closeout

> Document status: Superseded before activation (2026-07-12)
> Published plan date: 2026-07-03
> Planned objective: Execute weeks 13-16 of the 2026-07-03 four-month hardening plan: decide opt-in
> shared Skills compatibility, record REL-002 rehearsal evidence, sweep docs, and hand off the next
> cycle with honest release posture.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: the cycle closes with ecosystem decisions, REL-002 evidence, release posture, and
> residual owners.
> Supersession: I106-I109 produced non-qualifying evidence and the next sole-primary evidence target
> is materially different; it is replanned under I119.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| H130 | AGENT-002-B | Research | SKILL-002/ADR-022 | Opt-in shared Skills policy for `~/.agents/skills` |
| H131 | REL-002/GOV-003 | Planned | H130 as needed | REL-002 rehearsal packet with exact primary-executor boundary |
| H132 | CMD/CONF/GOV/WEB docs | Mixed | H100-H131 | Command/help/docs sweep |
| H133 | Hardening plan | Planned | H132 | Final four-month matrix, residual owners, release posture, and handoff |

### Scope

- Decide whether opt-in `~/.agents/skills` discovery is ready for implementation.
- If implemented, preserve Talos-owned config precedence and prompt budgets.
- Record one REL-002 rehearsal without overclaiming self-bootstrap readiness.
- Sweep docs for `/model`, `/connect`, `/agile`, `/plugins`, `/hooks`, install, and release posture.

### Non-Goals

- No shared `~/.agents/models.json` import.
- No shared `~/.agents/mcp.json` import.
- No automatic shared Skills loading.
- No v1.0 claim.
- No crate publish or release tag unless separately approved.

### Acceptance

- Given shared Skills are enabled by explicit config, Talos-owned Skills take precedence and prompt
  budgets remain bounded.
- Given REL-002 rehearsal evidence is recorded, it states whether Talos was the primary executor or
  only a helper.
- Given the cycle closes, release posture, residual owners, and next handoff are documented with
  validation evidence.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test -p talos-skill -p talos-agent` if shared Skills are implemented
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
- `docs/tasks/2026-07-03-four-month-product-hardening-plan.md`
- `docs/BOARD.md` after owner docs

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-03 | Planning | Created as the final iteration shell for the 2026-07-03 four-month hardening plan. |

## Verification Evidence

- Planned.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
