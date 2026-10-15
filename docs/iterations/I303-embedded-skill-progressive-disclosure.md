# Iteration I303: Embedded Skill Progressive Disclosure

> Document status: Active
> Published plan date: 2026-10-15
> Planned objective: let `talos-runtime` embedders deliver Skill Level 1/2 content to the model.
> MVP deliverable: runtime-only embedder sets a skill context and the next provider request contains
> `SKILL.md`; opt-in `skill_read` reads bodies/references under confinement and budget.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | OpenCode / Claude Opus 5.5 |
| Work Slice | #719 A-D: shared confined Skill Level 1/2 disclosure in talos-skill, load_reference hardening, CLI reuse, RuntimeHandle skill-context methods, opt-in runtime skill_read tool, tests and docs only. |
| Claimed At | 2026-10-15 |
| Source Issue | #719 |
| Governance Claim PR | #720 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer selected full A+B+C+D scope and Single-maintainer merge on 2026-10-15; no distinct maintainer reviewer available. Independent Agent security/API review, exact-head CI, both validators and merge-time CAS are mandatory. |
| Implementation PR | Not started |
| Last Updated | 2026-10-15 |
| Handoff / Release Condition | No implementation before this claim and ADR-092 reach main; release only after evidence-bearing closeout or explicit handoff. |

## Published Baseline

Selected Story: [SKILL-006](../backlog/active/SKILL-006-embedded-skill-progressive-disclosure.md);
preserve its scope, exclusions, acceptance, validation and documentation targets. Contract:
[ADR-092](../decisions/092-embedded-skill-progressive-disclosure.md). The Active/Claimed values
become effective only when the finalized #720 merges into main.

### Selection Inventory And Dispositions

Selection baseline: main `f4313237` (includes #718). Open PRs at selection: #716 (I293 Search form
POST) and #705 (Windows timeout diagnostic); neither touches talos-skill, Skill runtime wiring or
`RuntimeHandle` skill APIs.

| Owner | Current state | Disposition |
|---|---|---|
| I293 | Active / Claimed | Retain Search scope; no overlap (talos-tools search transport). |
| I294 | Active / Claimed | Retain provider evidence; no overlap. |
| I300 | Active / Claimed | Retain Git repair; no overlap. |
| I301 | Review / Claimed | Retain startup/storage acceptance; #711 merged, closeout separate. |
| I302 | Active / Claimed | Retain Windows test-only diagnostics; no overlap. |
| I277 | Review / deferred device acceptance | Retain. |
| I249 | Planned | Retain dependency-pilot deferral; no dependency change here. |
| I164 | Paused | Retain. |

No Blocked iteration header found. SKILL-001..005 are Complete and are not reopened.

### Shared-File Coordination

`crates/talos-runtime/src/lib.rs` is a large shared file; implementation adds isolated builder and
handle methods plus a new module, rebasing on main before the stable candidate to avoid conflicts.
Board/index rows are batched governance per ADR-071 and must not block other slices.

### Risks And Rollback

Path confinement is security-relevant: require independent review of canonicalization, symlink,
size and UTF-8 handling. Revert only this slice if regression appears; CLI budgets must not change.

## Actual Activation And Execution

2026-10-15: governance-only draft prepared; no production/test/dependency edit or implementation
branch.

## Verification Evidence / Completion Evidence

Pending. No Completion Commit exists; not Complete.

## Variance And Residuals

Symlink check/use race and host prompt-guidance policy (ADR-092) remain documented residuals.
