# SEARCH-001-C: Talos-Owned SearchRouter

**Status**: Active / Claimed; selected in I293
**Type**: Implementation Child
**Source Issue**: #643

Own first-valid-success routing and bounded hedging after SEARCH-001-B. The claim activates only
this bounded C slice; implementation must preserve ADR-085, the model-facing contract and the
existing paid-provider/Wikipedia compatibility constraints.

Iteration: [I293](../../iterations/I293-search-router.md). Effective governance claim: #664,
merged at `d801790454e252699684253166984662ea9dab6a`.

## Current Execution Boundary

The current session permits architecture/evidence and test-only characterization, not production
search behavior changes. The test-only candidate records the existing first-completion race;
it does not implement or complete SearchRouter. Runtime routing, typed failures, bounded hedging,
deadline/cancellation, health state and binary acceptance remain pending. Keep #643 and #624 open.
