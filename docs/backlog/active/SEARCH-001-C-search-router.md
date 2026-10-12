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
search behavior changes. The merged #681 test-only stage records the existing first-completion race;
it does not implement or complete SearchRouter. Runtime routing, typed failures, bounded hedging,
deadline/cancellation, health state and binary acceptance remain pending. Keep #643 and #624 open.

Existing test evidence: #681 merge `41381f8055b66d54de42988e2b936a12e5af3ac3`.
The [router integration plan](../../reference/SEARCH-001-C-ROUTER-INTEGRATION-PLAN.md)
audits the caller-context gap and supplies the pending implementation/acceptance matrix;
it does not activate a router or complete C.

The 2026-10-10 I293 stage adds a disposable twelve-case offline scheduling experiment, with
paused-clock, launch/drop and bounded-concurrency observations. It exercises already-classified
synthetic outcomes and control-boundary races; it does not call production search or establish
HTTP cancellation, admission, caller-context, health or binary integration. All full C acceptance
rows remain pending. See the iteration owner for recovery provenance and current validation.

#696 scheduling stage merged at `444a8137f054b33802f26bffdb55fb0f83a7a861` with full
exact-head CI3129 and independent Agent technical review. The next bounded test-only stage
experiments with URL admission/filtering/truncation and injected-clock validation boundaries.
It also records URL-parser repair behavior as a policy gap, not strict syntax or fetch safety.
Neither stage integrates production search or completes C-V02/C-V05; full C acceptance remains
pending. The iteration owner retains validation, limits and remaining integration gates.

#706 result-admission stage merged at `4b03149ca452bbaf4e96b580ab74135bc4ac670d` with full
CI3136 and independent technical review. The next test-only target drives actual Agent permission
dispatch and Session Interrupt using an offline Network stand-in, to characterize approval TTL,
the detached coordination token and owned-future cleanup. It does not call WebSearchTool or prove
HTTP/DNS cancellation, deadline propagation or full C-V10 acceptance. C and Epic remain open;
the iteration owner records fresh validation and residuals.


#707 caller stage merged at `7e817ab59364bf282c49941ec501f82b0c6c40fb` with six successful
CI3141 jobs and independent exact-head Agent review. The next test-only stage uses the real
registered WebSearchTool metadata and PermissionSessionState without executing search: default
Ask cannot compile a typed resource proposal; explicit fixture Allow can admit without a grant.
Production registration uses the Agent pipeline; the apparent CLI/SDK PermissionAwareTool classes
are test-only. This records a configuration-specific resource gap, not binary/transport acceptance
or authorization to repair policy. See I293 and the router plan for source audit and fresh gates.
All full C acceptance remains Pending; #643/#624 stay open.
