# Iteration I278: Repository-Root Plugin Source Delivery

> Document status: Planned
> Published plan date: 2026-09-18
> Planned objective: Deliver the omitted repository-root plugins/ source and independent build/package path from #466.
> Baseline rule: preserve this objective and acceptance; append execution facts.
> MVP deliverable: build an optional Plugin/Provider from root plugins/ source, package/install it as a verified Bundle and execute its real input-dependent capability.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | CAP-001-D root Plugin source delivery only |
| Claimed At | Not applicable |
| Source Issue | #466 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | 2026-09-18 maintainer next-cycle scheduling instruction; not activation |
| Implementation PR | Not started |
| Last Updated | 2026-09-18 |
| Handoff / Release Condition | Next after I277 local convergence, independent review and merge; deferred human rows remain with I277/#29 and do not count as passed |

## Published Baseline

Selected Story: [CAP-001-D](../backlog/active/CAP-001-D-root-plugin-source-layout.md).
Dependencies: existing CAP-001-B/C, language contracts, Bundle/manual installation and
ADR-027/072/073. Verify current target-branch dependency evidence and effective claim before code.

Scope: inventory concrete providers versus fixtures; implement root source organization and
independent build/package entrypoints; exercise installed artifacts; update affected consumers,
tests and layout documentation. Host infrastructure remains in `crates/talos-plugin/`.

Acceptance: every CAP-001-D acceptance item is mandatory. Empty directories, source relocation
without executable packaging, and canned fixture results cannot establish completion. Prove
input-dependent capability behavior, safe failure, dependency isolation and reference consistency.

Non-goals: Desktop, new Browser capabilities, marketplace, implicit activation/acquisition,
default distribution changes, release or publication. Preserve published prior iteration evidence.

Validation: focused build/package/install/consumer and failure tests; locked preflight for code;
governance validators and whitespace checks; stable-candidate CI and appropriate independent review.
Documentation: Plugin build/package/usage guide, host README, architecture layout, ADR-072,
Story/iteration and owner-first derived indexes.

Risk: existing artifacts may prove only a test seam, not distributable implementation. Inventory
this before moving files and implement the missing bounded delivery rather than relabel fixtures.
Rollback: revert the source/build migration without changing installation identity or host policy.

## Scheduling And Inventory — 2026-09-18

The maintainer explicitly prioritizes this omitted requirement for the next iteration cycle.
The local iteration-header inventory found I277 Review and I249 Planned, with no Active or
Blocked current header. I164 remains Paused under its superseded layout target. Completed-owner
historical Review checkpoints are not new active claims.

| Owner | Disposition |
|---|---|
| I277 | Continue current Desktop corrections and machine/review/merge gates first. Retain deferred human validation in #29; do not fabricate Complete. |
| I249 | Keep published dependency pilot Planned/Unclaimed; not selected ahead of I278. |
| I164 | Keep Paused; no reactivation or layout work authorized. |
| I278 | Planned/Unclaimed, first next implementation cycle; establish target-branch claim at handoff. |

No plugin implementation begins in the I277 branch. Keep this planning change separate from the
Desktop implementation commit/candidate. Reuse #466 as source; no per-subtask Issue is required.

Evidence correction: I253's R1 `Satisfied` label explicitly excluded source delivery. Its cited
ADR-072/architecture layout contract is not explicit in the current files. This plan owns both
the missing contract clarification and implementation; the historical label is not acceptance.
