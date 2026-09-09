# Iteration I253: Capability Architecture Governance Convergence

> Document status: Active
> Proposed through PR #518; claim and activation are ineffective until merge to main.
> Published plan date: 2026-09-09
> MVP deliverable: An auditable requirement-to-owner map for every governance acceptance item in Issue #466, with independently executable downstream Stories.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex mainline execution Agent |
| Work Slice | CAP-001-G architecture audit, terminology migration contract, child decomposition and documentation synchronization only. |
| Claimed At | 2026-09-09 |
| Source Issue | #466 |
| Governance Claim PR | #518 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized single-developer unattended completion of #499 then #466 and accepted ADR-072. Independent Agent-role review and exact-head CI remain required; shared GitHub identity does not prove natural-person separation. Proposed claim is ineffective until merge. |
| Implementation PR | Not started |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Claim and activation must reach main before governance implementation; no runtime implementation authority. |

## Published Baseline

### Selected Story

CAP-001-G, child of CAP-001. Depends on accepted ADR-072 and completed CAP-001-P0/I246.
I252 is already Complete; its implementation is preserved, not repeated or broadened.

### Scope

1. Reconcile every Issue #466 governance acceptance row against current code and document evidence.
2. Inventory executable Plugin versus distribution Bundle references, compatibility-sensitive APIs,
   manifest/config fields, current Arborium consumers and runtime boundaries. Retain historical size
   measurements as historical, not as new measurements.
3. Preserve ADR-027 and historical ADR-029; add explicit supersession pointers and a staged migration
   contract without renaming any production API or stored field.
4. Create bounded child owners and corresponding Issues for remaining CAP, Bundle, Text, Language,
   distribution and Browser outcomes after collision checks. Give each dependencies, constraints,
   acceptance, validation, documentation and reciprocal parent links. Preserve completed A/I252.
5. Separate TOOL-008 feature trimming from runtime loading, DIST-001 policy from installation, and
   WEB-005 product/security semantics from the connector. Preserve completed owner evidence.
6. Synchronize architecture, requirement convergence, proposal/ADR indexes, backlog and Issue map.
   Board lists selected children only, never the entire Epic as implementation work.

### Non-Goals

No Rust/Cargo, runtime registry/resolver, public API or persisted-schema mutation, Plugin loading,
network installation, permission, sandbox, release, Browser or Desktop/Dashboard implementation.
No reopening historical completed Stories. No activation of CAP-001-B/C or domain children.
Implementation stages 3-7 in #466 must be preserved as owned downstream work, not declared shipped.

### Acceptance

- Every #466 governance acceptance bullet has a specific repository evidence pointer or remains
  explicitly unfulfilled; a green validator cannot substitute for semantic coverage.
- All six domain terms, object-specific lifecycles, shared Language Provider path, UI-independent
  text semantics, plugin source versus Bundle artifact layout, and domain-specific fallback are
  documented as current behavior or target contract without conflating the two.
- Migration records cover PluginManifest/Metadata, package root, cache identity and contributions;
  dual-read, controlled-write, unknown-field handling, rollback and later schema ADR gates are explicit.
- Each executable downstream Story has one Issue and one authoritative owner; dependencies are
  acyclic, and only the next dependency-ready child may be Ready (not Active or Claimed).
- No runtime loading, parser-size reduction, installation or Browser behavior is claimed on the
  strength of descriptors, policy documents or Cargo feature trimming.
- Existing completed owners and historical plans remain intact; owner-first derived synchronization
  and Issue updates agree with actual evidence.

### Validation And Documentation

Governance-only infrastructure exception: no new binary behavior. Run both repository governance
validators with explicit base, YAML parsing, local links/changed-file inventory and diff checks.
Independently review the full #466 acceptance matrix, not only validator output. No Rust build is
required for documentation-only changes. Required documentation: CAP-001 and child owners,
docs/reference/ARCHITECTURE.md, docs/roadmap/REQUIREMENT-CONVERGENCE.md, relevant proposals/ADRs,
backlog/iteration indexes, Board, manifest and Issue/doc matrix.

### Risks And Rollback

Main risk: narrowing #466 to already-delivered descriptors or claiming downstream implementation
through planning. Preserve original Issue requirements and distinguish governance completion from
product delivery. Revert only this slice's documentation if needed; never erase historical evidence.

## Inventory And Startup Contract

Baseline: main `c878eabc`; 2026-09-09 API open-PR inventory is empty.

| Item | State | Disposition |
|---|---|---|
| I249 | Planned / Unclaimed | Retain dependency pilot without activation; full upgrade I250 is Complete. |
| I164 | Paused / superseded | Retain, do not resume. |
| I252 | Closed (completion evidence preserved) | Preserve descriptor implementation and retrospective review evidence. |
| I251 / #499 | Closed (completion evidence preserved) | Preserve closure; #499 is CLOSED. |
| Other iterations | Terminal or legacy historical | No Active/Review/Blocked current delivery header; I162 Complete/Review-outcome is terminal. |
| CAP-001 / #466 | Refinement / Unclaimed | Parent remains unselected; select only this governance child. |

Work mode: Standard, single-developer unattended. Ordered work: audit, migration/decomposition,
owner/index synchronization, independent governance acceptance audit, evidence-based closeout.
Allowed actions: scoped docs, local validation, stable PR/Issue coordination under existing
maintainer authorization. No destructive operations, purchases, release or runtime changes.
Use one claim stage then one locally converged governance implementation candidate; preserve
exact-head review/CI and CAS. Pause on a decision that would change accepted ADR-072.

### Child Issue Map (2026-09-09)

| Owner | Issue | State | Dependency disposition |
|---|---:|---|---|
| CAP-001-B | #512 | Ready / Unclaimed | Only dependency-ready child; not activated |
| CAP-001-C | #513 | Refinement / Unclaimed | Waits for CAP-001-B |
| TEXT-001 | #511 | Refinement / Unclaimed | Waits for CAP-001-C |
| LANG-001 | #510 | Refinement / Unclaimed | Waits for TEXT-001 and CAP-001-C |
| LANG-002 | #516 | Refinement / Unclaimed | Waits for LANG-001 |
| LANG-003 | #517 | Refinement / Unclaimed | Waits for LANG-002, BUNDLE-001 and DIST-001-A |
| BUNDLE-001 | #514 | Refinement / Unclaimed | Waits for CAP-001-C |
| DIST-001-A | #509 | Refinement / Unclaimed | Waits for BUNDLE-001 and CAP-001-C |
| DIST-001-B | #515 | Refinement / Unclaimed | Waits for DIST-001-A, CAP-001-B/C and BUNDLE-001 |
| BROWSER-001 | #508 | Refinement / Unclaimed | Waits for CAP-001-B/C and WEB-005 |

These Issues are intake tracking only. Creating them does not activate an owner or grant
implementation authority; each requires a separately selected runnable iteration and effective
Collaboration Claim.

## Required Reads

- Issue #466 full body/comments and Issue #467; ADR-072; CAP-001 and I252.
- PLUGIN-001, TOOL-008/012/014, DIST-001, WEB-005 and DESKTOP-001 owners.
- ADR-027/028/029/030, plugin-encapsulation-format, optional-runtime-asset-distribution,
  web-005-browser-session-continuity-design, architecture and requirement-convergence documents.
- Issue #308 for Session/Preset product semantics; CAP-001-P0 compatibility evidence.

## Execution And Completion Evidence

Completion Commit: Pending. Status changes cannot self-certify.
Next: finalize atomic claim using the actual PR number, obtain stage checks, CAS and merge;
then start the governance implementation from that merge or later main.

### 2026-09-09 Atomic Claim Candidate Checkpoint

PR #518 now proposes Active / Claimed for I253 and CAP-001-G only. This checkpoint
supersedes the earlier claim-preparation current state, not the Published Baseline.
Both ownership and activation remain ineffective until merge. No implementation branch
may start before that merge or a later main commit. CAP-001 and all ten downstream
children retain their existing unclaimed states; ADR acceptance grants no runtime authority.

Fresh fetch confirms main remains `c878eabcee9ca3064b15291ce8fa073874915791`.
Inventory: I249 remains Planned/Unclaimed and deferred; I164 remains Paused/superseded;
all other current iteration headers are terminal, with I162's Complete/Review-outcome
treated as terminal. I253 is the sole proposed activation. PR #518 is the only open PR;
there is one worktree and no stash. Earlier CI `34322110835` proved the planning head
`111199b9`, not this changed claim candidate; fresh exact-head CI/review are required.

Next gate: validate this finalized candidate locally, push it once, obtain independent
Agent-role governance review, then perform merge-time CAS. After merge, audit the full
#466 acceptance matrix and migration contracts locally; do not claim the intake map
alone completes #466. No Rust build is required for this documentation-only candidate.

The earlier dated Child Issue Map is superseded for LANG-002 dependency detail: the
Rust WASM vertical slice requires LANG-001, CAP-001-C and DIST-001-A (verified manual
installation), not just LANG-001. BUNDLE-001 owns the actual compatible schema/manifest
migration after its decision gate, not only the migration plan. These corrections
preserve #466's installation/loading and WASM requirements without activating children.

### 2026-09-09 Claim Review Disposition

The dated Child Issue Map remains historical. Current dependencies are owned by the
child documents: TEXT-001 depends on completed CAP-001-A, LANG-001 on TEXT-001 and
CAP-001-A, and BUNDLE-001 on ADR-072 plus manifest compatibility evidence and a dedicated
migration decision. CAP-001-C is not a prerequisite for those contracts. TEXT-001
remains Refinement pending this governance audit and its consumer ownership inventory;
only CAP-001-B has completed readiness preparation. This is scheduling, not a fabricated
technical dependency. LANG-002 still needs CAP-001-C and DIST-001-A for real WASM loading.

The following acceptance remains unfulfilled and belongs to post-claim I253 execution,
not this activation candidate: ADR-029 supersession note; linked supersession of the
old Plugin encapsulation proposal and ADR/proposal indexes; plugins/ runtime-source
versus bundles/ artifact layout; concrete PluginManifest skills/tools/hooks versus
missing mcp/provides evidence; full #466 acceptance matrix and migration audit; and
distinct executable Issue mapping for CAP-001-G with a reciprocal Epic child link.
Issue #466 remains the parent requirement source, not authorization to implement all
children. These rows must be completed before governance closeout; intake alone is
not completion evidence.
