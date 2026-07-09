# Iteration I108: Architecture-Sensitive Self-Bootstrap

> Document status: Planned
> Published plan date: 2026-07-08
> Planned objective: have Talos route and complete one bounded architecture-sensitive session as
> the primary executor without bypassing review gates.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an architecture-sensitive audit, diagnostic, or bounded change with risk
> classification, evidence, review outcome, and REL-002 classification.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SBT120` | 2026-07-08 self-bootstrap plan | Planned | I107 closeout | Architecture-sensitive owner selected and risk-classified. |
| `SBT121` | 2026-07-08 self-bootstrap plan | Planned | SBT120 | Talos performs the bounded architecture work. |
| `SBT122` | 2026-07-08 self-bootstrap plan | Planned | SBT121 | External review checks claims without taking over implementation. |
| `SBT123` | 2026-07-08 self-bootstrap plan | Planned | SBT122 | Session is classified against REL-002. |

### Scope

- Prefer `ARCH-032` Single Data Flow Audit unless a different bounded owner is explicitly selected.
- If code changes are selected, keep them narrow and inside existing architecture boundaries.
- Record review findings and residuals without hiding external intervention.

### Non-Goals

- No session-storage default migration unless a separate `SESSION-004` gate is explicitly activated.
- No permission policy, sandbox, credential, dependency, release, or broad orchestration change.
- No new ADR unless the selected work actually requires a decision record.

### Acceptance

- Given architecture-sensitive work is selected
  When Talos classifies the risk
  Then the correct owner docs and review gates are named before implementation.
- Given Talos completes the work
  When evidence is reviewed
  Then claims are backed by source, tests, runtime evidence, or explicit audit artifacts.
- Given external review changes the result
  When REL-002 is updated
  Then the qualification status reflects the real executor boundary.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Architecture evidence or runtime scenario required by the selected owner.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- Selected architecture/backlog owner doc.
- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: Talos misclassifies high-risk work or drifts into a forbidden boundary.
- Rollback: stop the iteration, record the risk-routing defect, and require maintainer review before
  resuming.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-08 | Planning | Created as Month 3 of the 2026-07-08 Talos-primary self-bootstrap plan. |
| 2026-07-09 | Activation (SBT120) | ARCH-032 selected as default work item per four-month plan. Risk classification: audit-only, no code changes, no permission/sandbox/dependency/storage gates. Runtime: glm-5.2 external. |
| 2026-07-09 | Architecture work (SBT121) | ARCH-032 Single Data Flow Audit completed. All 12 src/ directories audited. Zero `broadcast::channel` usages. All mpsc channels single-consumer. Three watch channels are state distribution (compliant). Hook system uses trait-method dispatch (not channels). No deviations found. ARCHITECTURE.md updated with Channel Topology Audit section. |
| 2026-07-09 | Review (SBT122) | External review: audit claims verified against source code. All channel counts and classifications traceable to file:line locations. No architecture-sensitive code changes made. |
| 2026-07-09 | Closeout (SBT123) | Session classified non-qualifying for REL-002 (external runtime glm-5.2). ARCH-032 status: Complete. I108 moved to Review. |

## Verification Evidence

- ARCH-032 audit: all 12 src/ directories examined for channel patterns (mpsc, broadcast, watch, oneshot).
- Zero `broadcast::channel` usages confirmed across the workspace.
- ARCHITECTURE.md updated with factual current-state channel topology diagrams.
- Governance validation: 0 warnings.
- REL-002 classification: NON-QUALIFYING (runtime was glm-5.2 external, not talos binary).

## Variance And Residuals

- No deviations found. No follow-up stories required. The workspace is fully ADR-006 compliant.

## Retrospective

- This iteration's audit was executed by external runtime (glm-5.2 via zai-coding-plan). Per REL-002 criterion 7, the session is non-qualifying. The audit findings are factually correct and traceable to source, but the self-bootstrap capability was not demonstrated.
