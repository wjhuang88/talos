# ARCH-034-R04-AG10: Symbol Notice Admissibility Semantics

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | PR #177 review residual 1 / bounded-traversal notice contract noise |
| Status | Refinement — reviewed counter semantics conflict with broader omission wording |
| Priority | P2 |
| Selected Iteration | None |
| Preserved behavior | Traversal safety limits, non-following classification, admitted symbol results/order, notice schema and all non-symlink normal trees |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable until CHANGE-CONTROL selects the contract |
| Authorization Evidence | PR #177 independent review comment `5230395611` approved I182 and classified this semantic conflict as non-blocking follow-up. |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Preserve I182 semantics until a separately published change decision and effective claim exist. |

## Confirmed Baseline

AG-4 defines `symlink_skipped` as every refused traversed symlink entry, so the
counter is incremented before directory-name and language-extension exclusions.
An unsupported file symlink or a symlink named `target`/`.venv` can therefore
produce a bounded-traversal notice even though the target would not have been
admissible work. This matches the exact counter definition but conflicts with
the broader statement that a notice appears only when work is omitted.

## Scope And Acceptance

- Apply `docs/sop/CHANGE-CONTROL.md` to choose whether the exact counter wording
  or the broader omission invariant is authoritative.
- If semantics change, define classification order without following the link and
  prove cycle/depth/byte containment is unchanged.
- Cover unsupported file symlinks, skipped-name directory symlinks, admissible
  source symlinks and real cycles with exact notice JSON fixtures.
- Preserve byte-identical admitted results and the existing discriminated notice
  schema unless a separately accepted public-contract decision says otherwise.

## Exclusions And Residuals

No silent I182 edit, path containment, invalid-text policy, parser deadline,
sorting or new traversal abstraction.

## Minimum Validation

Focused symbol tests, locked release preflight, Unix/Windows CI, change-control
record, both governance validators and independent review.
