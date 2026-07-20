# ARCH-034-A: Architecture Evidence And Boundary Audit

| Field | Value |
|---|---|
| Type | Architecture Spike |
| Parent Epic | ARCH-034 |
| Status | Ready |
| Priority | P1 |

## Deliverables

- `docs/reference/ARCHITECTURE-AUDIT-2026-07.md`: evidence, diagrams, verdicts.
- Machine-readable finding register with stable IDs, severity, confidence, owner,
  affected seams, proof, recommended action and disposition.
- Current crate graph/fan-in/fan-out, module/production/test LOC, change hotspots,
  public API/visibility map and extension-scenario traces.
- Duplicate inventory separating exact/textual clones from duplicated domain policy.
- Reconciled ARCH-011/022/023/030 status and current measurements.

## Method

1. Establish a clean locked baseline and compile/test/lint evidence.
2. Trace dependency direction from Cargo metadata and validate `talos-core` purity.
3. Classify each large/change-hot module by responsibilities and reasons to change.
4. Trace representative feature additions end-to-end and count touched owners/seams.
5. Search duplicated validation, mapping, state transition, persistence and permission
   logic; confirm findings manually before classification.
6. Review public APIs, error boundaries, state ownership and native/panic containment.
7. Score findings: P0 safety/correctness, P1 boundary/scalability, P2 maintainability,
   P3 observation. Record counterevidence and “no change” decisions too.

## Acceptance

- Every one of 21 crates has an owner/responsibility and dependency-direction verdict.
- Every production file above the selected evidence threshold is classified; no file
  is condemned solely by LOC.
- At least the seven named extension scenarios have a touch-point/coupling trace.
- Every suspected duplicate has semantic equivalence evidence or is rejected.
- No production code is changed in this spike.
- Audit conclusions distinguish fact, measurement, inference and recommendation.

## Validation

- `cargo metadata --locked --no-deps --format-version 1`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Required Reads

- Parent ARCH-034 and every Required Read listed there.
- `docs/sop/CHANGE-CONTROL.md`
- `docs/sop/TESTING.md`
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
