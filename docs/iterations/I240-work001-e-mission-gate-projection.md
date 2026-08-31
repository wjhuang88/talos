# Iteration I240: Mission Final Gate And UI-Neutral Projection

> Document status: Review / Claimed
> Published plan date: 2026-08-31
> Planned objective: implement WORK-001-E/P4 as a shared Mission gate and UI-neutral projection
> without introducing Desktop/UI authority.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: a runnable non-GPUI walkthrough proves Goal evaluation, rework/staleness,
> Mission final evaluation, Delivery gating and TUI-compatible projection.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline WORK-001 session |
| Work Slice | WORK-001-E/I240 P4 only: Mission final-evaluation gate, UI-neutral state/event projection, TUI-compatible adapter and non-GPUI end-to-end evidence. |
| Claimed At | 2026-08-31 |
| Source Issue | #29 |
| Governance Claim PR | #450 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #450 merged as `c6ebc1a41d3824cea77a3a3f15862e95bbe27432`; exact-head implementation CI and independent review required |
| Implementation PR | #451 (`699147c98caaaa7ac0ce2bba16c1feb8d6cb3b45`); CI `33373616347` passed |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Claim+activation must merge before implementation; owner-first closeout after implementation evidence |

Before implementation, follow `docs/sop/AGENT-COLLABORATION.md`. Claim and Active state are
ineffective until the finalized governance record reaches `main`.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WORK-001-E | WORK-001 | Review / Claimed | WORK-001-D/I239 Complete; P1/P2 contracts; DESKTOP-001 section 20.5 | Mission gate, UI-neutral projection and non-GPUI end-to-end evidence |

### Non-Terminal Inventory And Disposition

| State | Iterations / owners | Disposition |
|---|---|---|
| Active | None | No active implementation is displaced. |
| Review | None | No review work is bypassed. |
| Planned / Unclaimed | I207, I208; unrelated governance/backlog items remain separately owned | Preserve and do not activate or overlap from I240. |
| Blocked | None with a current implementation iteration; unrelated blocked owners retain their blockers | P4 is selectable because I239 is Complete, but implementation remains unauthorized until this claim merges. |
| Paused | I164 | Superseded by I165; preserve history and do not resume. |

### Scope

- Mission-level final evaluation and Delivery eligibility gate over existing P1-P3 state.
- Minimal serializable UI-neutral state/event projection and TUI adapter compatibility.
- Non-GPUI end-to-end test/walkthrough and runtime/API documentation.

### Non-Goals

- Desktop, GPUI, Dashboard, localization, SESSION-009, `/auto`, permission/sandbox expansion,
  persistence migration, release or publication.

### Acceptance

- Given child Goal results, When Mission evaluation is absent or stale, Then Delivery is denied.
- Given a mutation after a child evaluation, When the gate runs, Then rework/staleness requires a
  new Completion Claim and evaluation.
- Given all required current child results and a passing Mission evaluation, When projection emits,
  Then Delivery eligibility and ordered UI-neutral events are deterministic.
- Given the current TUI bridge, When the walkthrough consumes the projection, Then compatibility is
  preserved without GPUI or Desktop dependencies.

### Planned Validation

- Focused Work Domain, Completion Claim, evaluator, Mission gate and projection tests.
- Serialization/order tests and a runnable non-GPUI end-to-end fixture or transcript.
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`.
- `./scripts/release_preflight.sh`, both governance validators and `git diff --check`.
- Exact-head CI and independent runtime/security/API review after local convergence.

### Documentation To Update

- `docs/reference/WORK-EVALUATION-API.md` and TUI/runtime adapter guidance.
- WORK-001-E owner, WORK-001 parent, Board, Product Backlog, iterations README and manifest.

### Risks And Rollback

- Risk: a second projection or Mission authority diverges from P1-P3 state.
- Rollback: keep the existing Work Domain, evaluator and TUI path intact; remove only the new gate/
  adapter behind a bounded API boundary and retain compatibility tests.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-31 | Atomic claim+activation | PR #450 merged as `c6ebc1a41d3824cea77a3a3f15862e95bbe27432`; I240 is effective and implementation started from that merge. |
| 2026-08-31 | Local end-to-end convergence | Added a storage-neutral fixture covering Completion Claim/report, revision staleness, rework, Mission gate denial, refreshed Goal/Mission PASS and serializable UI-neutral Delivery projection. Focused and full locked validation pass; PR #451 remains under exact-head CI and independent review. |

## Verification Evidence

- PR #451 exact head `699147c98caaaa7ac0ce2bba16c1feb8d6cb3b45`: Mission gate/projection and TUI forwarding tests pass. The storage-neutral
  `p4_non_persistent_fixture_covers_claim_staleness_gate_and_delivery_projection` test covers the
  complete P4 sequence; exact-head CI and independent review remain pending.

## Completion Evidence

- Completion Commit: Pending
- A status-only documentation commit cannot certify implementation completion.

## Variance And Residuals

- None at planning time; Desktop binding, persistence and multi-client work remain separate.

## Retrospective

- Outcome: Pending.
- Documentation: owner and derived views will be synchronized owner-first after claim/implementation.
- Lessons: preserve local convergence and stable-stage validation from GOV-008.
