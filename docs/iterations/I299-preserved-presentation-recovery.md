# Iteration I299: Preserved Presentation Recovery

> Document status: Active (proposed; ineffective until #698 merges)
> Published plan date: 2026-10-10
> Planned objective: recover effective ownership and deliver the preserved C0 presentation candidate.
> MVP deliverable: truthful process summaries and readable live TUI activity, queued bodies and tool results.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | C0 preserved TUI activity, queue, edit/Todo result presentation and process-tool summaries; associated regression repairs only. |
| Claimed At | 2026-10-10 |
| Source Issue | None |
| Governance Claim PR | #698 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized serial implementation, stable publication, independent Agent review and gated merge; explicitly permitted non-overlapping concurrency with I293/I294. No separate maintainer reviewer is available; exact-head CI, Agent review, validators and CAS remain required. |
| Implementation PR | Not started |
| Last Updated | 2026-10-10 |
| Handoff / Release Condition | Deliver only after effective claim, local convergence, native acceptance and exact-head gates; retain incomplete work in the handoff ledger. |

PR #698 proposes Active / Claimed atomically; neither is effective until this finalized
governance-only record reaches main. Implementation starts from that merge or later target commit.

## Published Baseline

### Scope And Historical Recovery

The coordinator is [the session handoff](../tasks/2026-10-10-session-todo-handoff.md).
Historical implementation commits `0a0622c0` and `3af78f04` precede an effective target-branch
claim. Preserve them on `recovery/session-handoff-20261010`; this record does not retroactively
authorize them, invoke an emergency waiver or make local main delivered code. Rebuild the
implementation candidate after the governance merge, as in I289's recovery precedent.

- Preserve live activity animation and queued message body visibility under constrained height.
- Preserve the small body bullet and process action/job_id/cursor/max_bytes/wait_ms summaries.
- Preserve edit soft red/green backgrounds through CJK wrapping and normal/head/tail output.
- Deliver display-only Todo status glyphs and active highlighting without rewriting user titles,
  descriptions, arbitrary checkboxes, error output or durable raw results.
- Repair the independent review finding: multiline user text can spoof Todo rows. Text prefixes,
  list headers or UUID-shaped suffixes alone are not trusted presentation provenance.

### Non-Goals

No tool execution, permission, sandbox, storage or process-hardening change; no new public API
without a separately accepted ADR/migration contract. No MODEL-014 implementation, compaction,
Git repair, validation/agile tools, memory-limit parameter, Search or Auto locale change. The seven
original handoff requirements remain separately scheduled, not completed by C0. No release.

### Acceptance

- Queued bodies remain readable alongside pending tool animation, including narrow/short layouts.
- Process summaries show the supported arguments without changing execution or private evidence.
- Edit background fills cover each projected visual row; original logical text remains unchanged.
- Todo titles/descriptions remain unchanged, including multiline text, forged headers and suffixes;
  status glyph/highlight behavior remains correct for trustworthy item status presentation.
- Normal/head/tail rendering, error and unrelated-tool cases retain their correct behavior.
- Native terminal acceptance confirms animation, constrained-height queue visibility, tinted CJK
  wrapping/selection and Todo appearance. Automated tests alone do not close these observations.

### Planned Validation And Documentation

- Focused locked TUI tests and process-tool summary tests; include multiline spoofing regressions.
- Standard `./scripts/release_preflight.sh` with pinned Rust 1.99.0 and locked workspace checks.
- Both governance validators; exact-head remote CI, independent Agent review and merge-time CAS.
- Update `README.md` and `README.zh-CN.md` for observable TUI presentation; synchronize this owner,
  the coordinator, iterations index and Board owner-first.

### Selection Inventory And Parallel Boundary

| Owner | State at selection | Disposition |
|---|---|---|
| I293 / #696 | Active / Claimed | Other session; Search unchanged; explicit non-overlap authorization. |
| I294 | Active / Claimed | Other session; provider evidence unchanged; explicit non-overlap authorization. |
| I290 / I291 / #682 | Review | Retain Auto locale acceptance and ownership; no takeover. |
| I277 | Review; human checks Deferred | Retain Desktop device/accessibility residuals. |
| I249 | Planned | Keep dependency pilot deferred. |
| I164 | Paused | Keep superseded layout target paused. |
| I296 / I297 / I298 | Terminal owners; claims Closed | Preserve their completion evidence; no reuse of closed claims. |

No other Active, Review, Planned or Blocked iteration header found at selection against
`e58e26783e2f48416b871a394b17ac79b642bb1f`. Story owners remain separately authoritative;
DEPENDENCY-003-A is retained without taking over its scope. Shared derived views use owner-first
union updates. Refresh all owners and open PRs again before merge.

### Risks And Rollback

False Todo recognition can corrupt presentation; broad transcript/API changes exceed this slice.
Resolve provenance conservatively and seek an ADR if required, rather than guessing from text.
Retain historical commits and revert only this bounded implementation if regression occurs.

## Verification Evidence

Historical candidate `0b396c48865b74e4cf3e7d0f711ccb208f264afd` passed full standardized
preflight on 2026-10-10 with debug-free, single-job, non-incremental profiles. Independent Agent
review identified the Todo defect above; it is not approval. Rebuilt candidate requires fresh gates.

## Completion Evidence

Pending. No target-branch implementation or native acceptance is claimed.
