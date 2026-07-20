# ARCH-034-B: Evidence-Gated Architecture Finding Remediation

| Field | Value |
|---|---|
| Type | Technical Epic child |
| Parent Epic | ARCH-034 |
| Status | Refinement — blocked on ARCH-034-A |
| Priority | P1 after accepted findings |

## Execution Contract

This is a remediation program, not authorization for a monolithic refactor.
ARCH-034-A findings create one new `ARCH-034-Rxx` story per coherent root or
cross-crate seam. Each story declares preserved behavior, public API impact,
tests, rollback and documentation. Only Ready stories may enter an iteration.

## Required Remediation Classes

- Wrong crate ownership or dependency direction: move the contract to its domain,
  retain anti-corruption adapters, and avoid CLI/TUI leakage into reusable crates.
- Low-cohesion module: split by stable responsibility/state ownership, not arbitrary
  size; keep facade/re-exports when semver requires them.
- Duplicate policy/logic: select one authoritative owner and migrate all callers with
  equivalence tests.
- Style/error inconsistency: use crate-local mechanical batches with Clippy/rustfmt,
  never mix with semantic changes.
- Rigid extension seam: prove the change against a named scenario before extracting a
  trait or registry boundary.

## Acceptance

- All accepted P0/P1 findings have closed remediation stories or an explicit ADR-backed
  defer decision.
- Each change is behavior-preserving unless a separate product story authorizes behavior.
- Before/after dependency, responsibility, duplication and extension-touch metrics are
  recorded; improvement must not increase hidden coupling elsewhere.
- Full locked validation and applicable security/semver review pass per story.

## Required Reads

- Parent ARCH-034
- ARCH-034-A accepted audit and finding register
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/CHANGE-CONTROL.md`
