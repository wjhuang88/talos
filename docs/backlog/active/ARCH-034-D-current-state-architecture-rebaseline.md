# ARCH-034-D: Current-State Workspace Architecture Rebaseline

| Field | Value |
|---|---|
| Type | Architecture Spike / Validation Repair |
| Parent Epic | ARCH-034 |
| Status | In Progress |
| Priority | P1 |
| Selected Iteration | I171 (Active; Claim PR #138 effective on `main@349d0cd1`) |
| Preserved behavior | All product/runtime/public API behavior |

## Problem

The accepted ARCH-034-A audit is tied to the v0.4.0 workspace at commit `db1ccf9` on
2026-07-20. The current source of truth is v0.7.0 with substantial session, CLI, TUI, provider,
tool-composition, and validation changes. Its measurements and several finding dispositions are no
longer current: for example, `cargo clippy --workspace --all-targets --locked -- -D warnings` now
passes, while production roots and recent change hotspots have moved.

The old report remains historical evidence. It is not sufficient proof that the current workspace
has been fully audited or that every current architecture issue has an owner.

## Goal

Produce a reproducible current-state architecture report and finding register that re-evaluates all
workspace crates and every material production root/seam, reconciles the existing ARCH-034 findings,
and creates one bounded remediation story for each accepted actionable issue.

## Scope

- Recompute the internal crate graph, fan-in/fan-out, cycles, public-boundary ownership, raw and
  non-test source measurements, and recent change hotspots from current source.
- Classify large or change-hot production roots by responsibilities and reasons to change; LOC is a
  locator, never the verdict.
- Trace the current provider, tool, permission-facet, TUI command/panel, session backend, plugin,
  runtime-consumer, storage, and process/native-boundary extension scenarios.
- Re-audit semantic duplication, composition seams, state/data flow, concurrency/cancellation,
  persistence, native/panic containment, `unsafe` ownership, test/source separation, and
  architecture-document drift.
- Reconcile every prior ARCH-034 finding as Closed, Still valid, Superseded, or Deferred with a
  current trigger and owner.
- Add or repair deterministic audit/validation harnesses when the harness itself prevents a
  truthful baseline. Such repairs may change tests/scripts only and must not alter production
  behavior.
- Create Ready or Refinement `ARCH-034-Rxx` children for accepted findings; do not implement
  production refactors in this spike.

## Non-Goals

- No production refactor, public API change, new dependency, feature work, release, tag, publish,
  deployment, permission-policy change, or sandbox/process-hardening change.
- No line-count-only split, speculative trait, shared-utils bucket, global registry/event bus, or
  cleanup that lacks an evidence-backed finding.
- No rewrite of the v0.4.0 report or I144 history.

## Acceptance

- All workspace crates have current responsibility, dependency-direction, fan-in/fan-out, and
  public-boundary verdicts derived from current manifests/source.
- Every production file above the selected evidence threshold and every top recent hotspot is
  responsibility-classified with counterevidence; no file is condemned solely by LOC.
- The named extension scenarios and native/panic/process boundaries have current touch-point and
  failure-containment traces.
- Suspected duplication is either proven semantically equivalent or rejected as textual
  similarity.
- Every old and new finding has severity, confidence, proof, counterevidence, preserved behavior,
  recommended action, disposition, owner, and verification path.
- Every actionable finding is represented by one coherent remediation story; deferred items name a
  trigger and owner.
- No production code or product/runtime/public API behavior changes.
- Required locked workspace, governance, and diff checks pass. A pre-existing validation failure
  must be repaired within the test/harness-only scope or remain an explicit blocker; it cannot be
  hidden by a narrower command.

## Deliverables

- `docs/reference/ARCHITECTURE-AUDIT-2026-08.md`
- `docs/reference/ARCHITECTURE-AUDIT-2026-08-findings.json`
- Deterministic project-local architecture measurement/fitness harness additions justified by the
  audit, if needed.
- Reconciled ARCH-034 parent/B/C/R01 state and bounded remediation owners.

## Validation

- `cargo metadata --locked --no-deps --format-version 1`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `scripts/assess_project_scale.sh .`
- `git diff --check`

## Required Reads

- ARCH-034 parent and ARCH-034-A/B/C/R01
- I144 and I158
- `docs/reference/ARCHITECTURE.md`
- both July 2026 audit artifacts
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/LONG-RUNNING-TASK.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/TESTING.md`

## Residual Destination

Production remediation belongs to separately claimed `ARCH-034-Rxx` stories and later iteration
IDs. Security-sensitive permission, sandbox, or process-hardening findings additionally require the
independent review path before implementation.
