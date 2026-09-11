# Iteration I257: Shared Language Provider

> Document status: Active / Claimed
> Objective: Implement LANG-001 shared LanguageProvider contract and migrate existing TUI and symbol-tool consumers.
> MVP deliverable: one renderer-neutral provider seam with guarded built-in implementation and injectable dispatch tests.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | LanguageProvider contract, built-in Arborium adapter, TUI HighlightEngine and four talos-tools symbol consumers; compatibility wrappers and tests. |
| Claimed At | 2026-09-11 |
| Source Issue | #510 (parent #466) |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Proposed claim; ineffective until merged to main. |
| Implementation PR | Not started |
| Last Updated | 2026-09-11 |
| Handoff / Release Condition | Claim must merge before implementation; preserve I256 and I246 compatibility. |

## Dependencies and non-goals

TEXT-001/I256, CAP-001-A and ADR-072 are complete. Excludes WASM, dynamic loading, bundle/installation, Desktop/Dashboard, permission, release and default feature expansion.

## Acceptance and validation

- TUI and `find_symbol_in_file`, `find_refs_in_file`, `collect_file_symbols`, `list_imports_in_file` dispatch through one provider seam.
- Existing aliases, constructors, serde, extension mapping, traversal/permission rules and fallback behavior remain compatible.
- Fake/recording provider tests prove dispatch; guarded parser failures fail safely.
- Focused tests, locked workspace preflight, governance validators, exact-head CI and independent API/compatibility review pass before merge.

## Published Baseline

LANG-001 was Refinement / Unclaimed before this selection. I256 completion at `a1a215e0` removes the prior dependency blocker. This baseline is historical and must not be rewritten.
