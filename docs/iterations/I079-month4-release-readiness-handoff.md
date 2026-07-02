# Iteration I079: Month 4 — Release Readiness And Handoff

> Document status: Active (2026-07-02)
> Published plan date: 2026-07-01
> Planned objective: Execute weeks 13-16 of the 2026-07-01 replan: final tool reliability sweep,
> associative memory injection decision, third self-bootstrap rehearsal, publish gate packet,
> release/user docs consolidation, REL-002 readiness report, closeout, and final handoff.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a verified release/readiness posture with residual owners and no hidden publish or
> self-bootstrap claims.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| T130 | Tool reliability | Planned | T104/T115 | Reliability sweep |
| T131 | MEM-008 | Planned | T51 metrics | Associative injection decision |
| T132 | REL-002 | Planned | T123/T131 | Third rehearsal >60% target |
| T133 | ARCH-031 | Planned | T55/T56 gates | Publish gate packet |
| T134 | Release docs | Planned | all tracks | Docs consolidation |
| T135 | REL-002 | Planned | T132/T134 | Readiness report |
| T136 | Replan | Planned | T100-T135 | Final closeout |
| T137 | Replan | Planned | T136 | Final handoff |

### Scope

- Final reliability and release posture.
- Memory injection decision.
- Final self-bootstrap evidence and readiness report.
- Handoff artifacts.

### Non-Goals

- No real publish without maintainer approval.
- No v1.0 readiness claim unless REL-002 passes.
- No default-on memory injection without accepted decision.

### Acceptance

- Given publish gates are reviewed, when no approval exists, then real publish remains non-action with blockers recorded.
- Given REL-002 is evaluated, when criteria fail, then report names residual owners and next-quarter plan.
- Given closeout completes, then workspace tests, governance, and publish guard pass or exact blockers are recorded.

### Planned Validation

- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings` if feasible, otherwise per-slice clippy evidence
- `scripts/validate_project_governance.sh .`
- `scripts/check_publish_guard.sh .`
- Site/link validators for docs changes

### Documentation To Update

- README/site/crate docs/changelog draft
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
- REL-002 owner doc
- Final handoff task

### Risks And Rollback

- Risk: readiness report overclaims. Rollback: fail criteria explicitly and keep pre-1.0 posture.
- Risk: publish gate is mistaken for publish approval. Rollback: keep all real publish commands out of scope.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-01 | Planning | Created as Month 4 shell for the replan. |
| 2026-07-02 | Activation | Activated after I078/T126 closeout was pushed and issues #7/#8/#15 were closed. First packet is T130 tool reliability sweep. |
| 2026-07-02 | Inventory | Non-terminal iteration inventory before activation: I079 Planned -> activated; R27 high-risk gate remains In Progress; Architect-owned high-risk work group remains Paused; legacy review/planned rows remain untouched unless selected by T130-T137. |
| 2026-07-02 | T130 | Completed tool reliability sweep. Fixed the one active ignored agent test by synchronizing on session completion events, suppressed intentional example-helper dead-code warning noise, and recorded shell naming / Windows support as TOOL-006 residual rather than changing tool contracts in this slice. Evidence: `docs/tasks/2026-07-02-t130-tool-reliability-sweep.md`. |
| 2026-07-02 | T131 | Completed associative memory injection decision. ADR-033 rejects default-on associative injection for v1 readiness, keeps graph recall explicit, and requires any future automatic associative prompt section to be a separate default-disabled experiment with benchmark evidence. |
| 2026-07-02 | T132 | Completed third self-bootstrap rehearsal as gap evidence. Talos generated workspace/governance validation plans for the architecture-sensitive ADR-033 slice, but remained read-only; estimated self-bootstrap coverage was 20%, so the >60% target was missed and REL-002 remains unsatisfied. Evidence: `docs/tasks/2026-07-02-self-bootstrap-rehearsal-t132-architecture-decision.md`. |
| 2026-07-02 | T133 | Completed publish gate packet for `talos-cli` and `talos-runtime`. Dry-runs remain blocked by intended guards/dependency closure, and `talos-dashboard` was added to the product-only publish guard. Evidence: `docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md`. |
| 2026-07-02 | T134 | Consolidated release/user docs across README, README.zh-CN, public install pages, reference index, and draft release notes. Cargo install and `talos-runtime` publication remain documented as blocked, not shipped. |
| 2026-07-02 | T135 | Completed REL-002 readiness report. Verdict: not ready for `v1.0.0`; pre-1.0 releases may continue only with explicit posture and residual owner list. Evidence: `docs/reference/REL-002-READINESS-REPORT-2026-07-02.md`. |

## Verification Evidence

- Activation governance: `scripts/validate_project_governance.sh .` passed with 0 warnings before activation.
- T130 targeted validation: `cargo fmt --all -- --check`; `cargo test -p talos-agent test_interrupt_after_success_preserves_history`; `cargo test -p talos-runtime --examples`; `cargo clippy -p talos-agent -p talos-runtime -- -D warnings`; `rg -n "#\\[ignore\\]" crates docs`; `cargo test -p talos-agent`; `scripts/validate_project_governance.sh .`.
- T131 governance validation: `scripts/validate_project_governance.sh .`.
- T132 Talos rehearsal commands: `./target/debug/talos validate plan --profile workspace`; `./target/debug/talos validate plan --profile workspace --json`; `./target/debug/talos validate plan --profile governance`.
- T132 governance validation: `scripts/validate_project_governance.sh .`.
- T133 publish gate validation: `scripts/check_publish_guard.sh .`; `cargo metadata --no-deps --format-version 1`; `cargo package --list -p talos-runtime`; `cargo package --list -p talos-cli`; `cargo publish --dry-run -p talos-cli` (blocked as intended); `cargo publish --dry-run -p talos-runtime` (blocked by unpublished `talos-agent`).
- T133 governance validation: `scripts/validate_project_governance.sh .`.
- T134 docs validation: `scripts/validate_public_site.sh`; `scripts/validate_project_governance.sh .`.
- T135 governance validation: `scripts/validate_project_governance.sh .`.

## Variance And Residuals

- T130 scope variance: shell naming and Windows command support were inventoried but left to TOOL-006 because they require user-facing schema, permission, and compatibility decisions.
- T131 scope variance: no automatic associative injection code was added; the task closed as an ADR-backed decision because the available evidence does not justify new model-facing automation.
- T132 target variance: the planned >60% autonomous validation target was missed; current Talos participation remains read-only validation planning.
- T133 guard variance: publish guard drift was found and fixed by adding `talos-dashboard` to the product-only guard/matrix.
- T135 readiness variance: REL-002 is a no-go for `v1.0.0`; only pre-1.0 releases remain in scope.
- Real publish/tag/release remains explicitly out of scope without maintainer approval.
- REL-002 is not satisfied by I078/T132; T135 records the remaining Talos-primary execution gap.

## Retrospective

- Pending.
