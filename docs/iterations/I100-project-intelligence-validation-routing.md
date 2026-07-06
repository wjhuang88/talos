# Iteration I100: Project Intelligence And Validation Routing

> Document status: Active
> Published plan date: 2026-07-06
> Planned objective: make project-type and governance recognition extensible and use it to route
> validation/adapters without hardcoded Rust assumptions.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: detector-registry hardening plus validation/host-tool adapter tests proving
> guidance is injected only after confirmed project type detection.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `VALIDATION-001` | Internal validation service | Complete first slice | I095, GOV-003 | Hardened detector/adapters and internal-first validation routing. |
| `GOV-003` | Built-in governance | Phase 1 partial; I096 mutation gate complete | VALIDATION-001 | Governance task recognition remains internal-first and preview-gated. |

### Scope

- Keep project-type detection as a registry of detector strategies.
- Add missing detector metadata/tests for common project types and mixed workspaces.
- Ensure host-tool adapter guidance is injected only after project type and selected profile match.
- Strengthen governance detection so governance tasks route to internal validation/mutation gates.
- Record unavailable-host behavior for adapters without treating Cargo as a generic Talos default.

### Non-Goals

- No arbitrary validation command configuration.
- No hidden TUI execution of host tools.
- No web dashboard write route.
- No automatic governance mutation without preview/confirm gates.

### Acceptance

- Given a Rust-only project,
  When validation profile selection includes Rust host-tool checks,
  Then Cargo adapter guidance can appear.
- Given a governance-only profile in a Rust repository,
  When validation plan output is rendered,
  Then Cargo guidance does not appear unless a host-tool check was selected.
- Given a Node/Python/Go/Java or mixed project fixture,
  When project detection runs,
  Then the matching detectors return independent project types.
- Given a governance mutation intent is recognized,
  When a write-capable action is needed,
  Then Talos produces preview/confirm behavior rather than silent mutation.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-conversation validation::tests`
- `cargo test -p talos-conversation slash_validate`
- `cargo test -p talos-cli validation`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

### Documentation To Update

- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: detector logic becomes a monolithic conditional that is hard to extend safely.
- Rollback: reject the implementation and keep the existing registry until a clean strategy surface
  is restored.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-06 | Planning | Created as Month 3 of the 2026-07-06 autonomy/permission/runtime hardening plan. Not active until I099 closes or is explicitly paused. |
| 2026-07-06 | Activation | Activated after I099 completed and was pushed. The phase starts from the existing VALIDATION-001 first slice, which already has a `ProjectTypeDetector` strategy registry and demand-driven adapter instructions. This activation selects hardening, fixture coverage, governance routing evidence, and host-tool adapter boundary cleanup. No arbitrary validation command execution, hidden TUI host-tool execution, permission-default relaxation, release action, or runtime `catalog.db` path is authorized. |

## Verification Evidence

- Pending.

## Variance And Residuals

- Pending.

## Retrospective

- Pending.
