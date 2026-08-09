# ARCH-034-R04-AG9: Symbol Input Decoding Consistency

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | PR #177 review residual 2 / invalid-text handling asymmetry |
| Status | Refinement — observable error/fallback policy requires CHANGE-CONTROL |
| Priority | P2 |
| Selected Iteration | None |
| Preserved behavior | Valid UTF-8 symbol results, ordering, traversal budgets/notices, language detection and public schemas |

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
| Authorization Mode | Not applicable until a behavior contract is selected |
| Authorization Evidence | PR #177 independent review comment `5230395611` reproduced the behavior on both `main` and the implementation head; no regression and no implementation authorization. |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Apply CHANGE-CONTROL, select fail/skip semantics, and establish a dedicated claim before implementation. |

## Confirmed Baseline

A supported-extension file containing invalid UTF-8 causes directory-mode
`list_symbols` to fail the whole request, while `find_symbol` skips it. The
independent reviewer reproduced this asymmetry on `main`, proving it is not an
I182 regression.

## Scope And Acceptance

- Decide whether both directory walkers fail, skip with explicit notice, or use
  another deterministic display-safe fallback.
- State unreadable-file and invalid-UTF-8 behavior separately; do not collapse
  I/O errors into decoding errors.
- Preserve valid-input output byte-for-byte and prevent partial parser admission
  before decoding succeeds.
- Add paired fixtures for both tools on all platforms and document any new
  observable notice/error contract.

## Exclusions And Residuals

No lossy source decoding, parser/language redesign, path containment, output cap
or silent change inside I182. AG-8 and AG-10 own their separate boundaries.

## Minimum Validation

Focused symbol tests, locked release preflight, Unix/Windows CI, documentation
sync when behavior is selected, both governance validators and independent review.
