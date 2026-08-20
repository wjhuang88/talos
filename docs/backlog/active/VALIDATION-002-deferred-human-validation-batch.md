# VALIDATION-002: Deferred Human Review And Acceptance Batch

| Field | Value |
|---|---|
| Story ID | VALIDATION-002 |
| Type | Governance / Human Validation Story |
| Priority | P0 within the mainline long-task closeout |
| Status | In Progress - I211 Active / Claimed; activation ineffective before activation PR merge |
| Source | [GitHub Issue #302](https://github.com/wjhuang88/talos/issues/302) |
| Selected Iteration | I211 - Active / Claimed; activation ineffective before activation PR merge |
| Depends On | Terminal dispositions for I200, I197, I201, I212, I210 and I198; exact implementation heads recorded in Issue #302 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline validation session |
| Work Slice | I211/VALIDATION-002/#302 evidence only: execute and record the already-defined natural-person and real-terminal rows for I200, I197, I201, I212, I210 and I198; synchronize source owners first; create separately governed corrective owners for failures. No product implementation, policy, dependency, persistence, release or publication change. |
| Claimed At | 2026-08-20 |
| Source Issue | #302 |
| Governance Claim PR | #326 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #326 exact head `d51d5721` passed CI `32347993402`, independent Agent approval `5353284891`, both governance validators and merge-time CAS, then merged as `285fc3c7`. |
| Implementation PR | None - evidence-only validation slice |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Every #302 row must pass or name a separately governed corrective owner; synchronize source owners first before long-task closure. |

## Identity / Goal / Value

Provide one scheduled human validation phase for long-running work so unavailable natural-person
review or device-dependent acceptance does not idle the implementation queue, while preserving a
truthful record that those gates have not yet passed.

## Scope

- Bind each source iteration's exact implementation head, merge commit, CI and Agent technical
  review before accepting a deferred row.
- Obtain independent natural-person review for I200, I197, I201, I212, I210 and I198 against their recorded
  implementation heads.
- Run I200's mouse-wheel and touchpad terminal matrix on the integrated `main` build.
- Run the owner-defined human/manual matrices added by I197 and I201 when their implementations
  exist; record any applicable I198 compatibility inspection.
- Record OS, terminal, input device, integrated `main` head, result and corrective owner for every
  manual row.

## Exclusions And Hard Boundaries

- No product implementation, bug repair, permission-policy change, release, tag or publication.
- No deferral of exact-head CI, locked checks, independent Agent technical review, governance
  validation or merge-time CAS required before a child implementation merge.
- No deferral of independent security review required by `AGENTS.md` for sandbox,
  `talos-permission`, process-hardening or permission-policy changes.
- I197 remains presentation/layout-only. Any permission semantic or policy change stops and needs a
  separately authorized security-reviewed iteration.

## Acceptance

- Issue #302 identifies every included source iteration and exact implementation head.
- A natural-person reviewer records a per-head conclusion with shared-account identity limits
  disclosed where applicable.
- Every device/manual row records the integrated `main` head, environment and result.
- Passed rows are synchronized owner-first into their source Story/iteration before derived views.
- Failed rows leave the source owner in Review and name a separately governed corrective owner.
- The long-task closeout does not claim Complete while any required Issue #302 row is open.

## Change Control - 2026-08-19 MODEL-013 Priority Advance

The maintainer advanced MODEL-013/#312 into I212 before I198. Add one I212 natural-person
custom-provider walkthrough after its exact implementation head exists: verify an exact catalog
match visibly supplies a catalog-derived context window, an explicit value remains unchanged, and
an ambiguous/unknown identity stays manual/unknown without a network request. This extends only the
deferred validation inventory; it does not authorize I212 implementation or rewrite I211's
published baseline.

## Change Control - 2026-08-19 Provider Reconnect Status

After I210 has an implementation head, add a natural-person live-provider/mock-provider walkthrough
that observes initial `Connecting…`, at least one truthful `Reconnecting… (attempt n/m)` transition,
and clearing on success, terminal failure and cancellation. Record the integrated `main` head and
confirm the displayed counts match the structured retry facts. This row does not waive I210's ADR,
claim, exact-head CI, technical review or CAS gates.

## Residual Destination

Implementation defects found during the batch use new corrective Stories/iterations. Automation of
this scheduling mode belongs to a separately claimed governance improvement; I211 changes no SOP
validator or CI behavior.

## 2026-08-20 Claim Preparation Checkpoint

The implementation queue is terminal: I200 merged as `9628e183`, I197 as `d98f37e7`, I201 as
`7f5a6df2`, I212 as `5a1709cb`, I210 as `9d5c8a71`, and I198 as `15a3d424`. Each remains Review
only for its Issue #302 natural-person/manual rows. No active iteration or overlapping open PR owns
this evidence slice; I189 remains Planned/Claimed but unactivated, I206-I208 remain separately
Planned/Unclaimed, and I164 remains Paused.

PR #326 proposes the evidence-only claim. It records no human pass and authorizes no product
repair. The claim is ineffective until its finalized exact head reaches `main`.

## 2026-08-20 Claim Review Correction Checkpoint

Independent Agent claim review `5353122975` bound to PR #326 head `229b9754` requested one
correction: Issue #302 did not yet record I200's final implementation disposition. Issue comment
`5353130091` now records PR #301 final head `8a58cb2d`, implementation evidence `3afeeb28`, CI
`32149762367`, Agent technical review `5330234992`, merge-time CAS and merge `9628e183`, while
preserving I200 as Review with its mouse/touchpad rows unpassed.

That remote evidence resolves the sole review blocker without claiming a human pass. Open
PR #327 is an unrelated Dashboard claim and does not overlap I211's evidence-only slice. Any new
#326 head requires fresh exact-head CI and independent claim review before merge-time CAS.

## 2026-08-20 Claim Merge And Activation Proposal

PR #326 exact head `d51d5721` passed CI `32347993402`, independent Agent approval `5353284891`,
both governance validators and merge-time CAS, then merged as `285fc3c7`. The effective claim now
permits an evidence-only activation proposal from that exact merge. Activation remains ineffective
until its own PR reaches `main`; no human row is treated as passed and no product repair is
authorized.
