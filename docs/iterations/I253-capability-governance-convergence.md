# Iteration I253: Capability Architecture Governance Convergence

> Document status: Planned
> Published plan date: 2026-09-09
> MVP deliverable: An auditable requirement-to-owner map for every governance acceptance item in Issue #466, with independently executable downstream Stories.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | CAP-001-G architecture audit, terminology migration contract, child decomposition and documentation synchronization only. |
| Claimed At | Not applicable |
| Source Issue | #466 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requests unattended completion of #499 then #466; claim preparation only. |
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

Claim preparation only. Completion Commit: Pending. Status changes cannot self-certify.
Next: finalize atomic claim using the actual PR number, obtain stage checks, CAS and merge;
then start the governance implementation from that merge or later main.
