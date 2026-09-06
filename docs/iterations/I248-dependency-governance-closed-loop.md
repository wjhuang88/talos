# Iteration I248: Dependency Governance Closed Loop

> Document status: Active (proposed; ineffective until #496 merges)
> Published plan date: 2026-09-06
> Planned objective: Deliver the complete #474 dependency baseline, audit, SOP, and live handoff loop.
> Baseline rule: preserve this scope; a different outcome requires a new iteration.
> MVP deliverable: Bash and PowerShell produce equivalent actionable reports from the same committed baseline and a live registry snapshot, with a generated bounded upgrade candidate and no automatic dependency mutation.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session |
| Work Slice | Full #474 governance mechanism: baseline, cross-platform audit, SOP, live handoff; no dependency upgrade |
| Claimed At | 2026-09-06 |
| Source Issue | #474 |
| Governance Claim PR | #496 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Draft PR #496 proposes atomic claim+activation; ineffective until target-main merge; independent review pending |
| Implementation PR | Not started |
| Last Updated | 2026-09-06 |
| Handoff / Release Condition | Merge #496, then local convergence and one stable candidate |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| DEPENDENCY-001 | #474 | Refinement / Unclaimed | release, testing, collaboration and doc-check SOPs | One usable dependency-governance workflow |

### Scope

- Script-owned accepted baseline with source commit and timestamp.
- Direct external dependency collection distinguishing manifest requirements, every lock resolution, baseline, latest stable, users, and policy signals.
- Bash and PowerShell parity for human, JSON and Markdown output, including offline/registry-unavailable behavior.
- Generic aggressive-latest-stable upgrade SOP with isolation, validation, blocker, rollback and baseline-advancement rules.
- Live audit evidence and one generated bounded candidate proposal; no automatic upgrades.

### Non-Goals

- No Cargo.toml/Cargo.lock/version changes, workspace dependency centralization, helper runtime, public API change, or concrete dependency upgrade.

### Acceptance

- Same fixture input yields semantically identical ordered Bash/PowerShell JSON and golden human output.
- Duplicate resolved versions, pre-1.0 minor risk, prerelease, yanked, deprecated, security, baseline drift, upstream drift, malformed input and unavailable registry are distinct outcomes.
- Live report records source SHA/time and produces a candidate Story without mutating the baseline or repository dependencies.

### Planned Validation

- Fixture/golden and mutation tests; Bash/PowerShell parity on supported platforms.
- Explicit live audit with recorded network result.
- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`.
- Both governance validators and `git diff --check`.

### Documentation To Update

- `docs/reference/DEPENDENCY-BASELINE.md`, `docs/sop/DEPENDENCY-UPGRADE.md`, README usage for audit commands, backlog/Board/Issue state.

### Risks And Rollback

- Risk: parser or registry semantics produce false freshness or hide a resolution.
- Rollback: revert the implementation candidate; accepted baseline advances only after a merged governed upgrade.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-06 | Planning | I248 replaces the invalidly widened local I247 attempt; claim remains ineffective until target-main merge. |

## Verification Evidence

- Pending claim and implementation.

## Completion Evidence

- Completion Commit: pending

## Variance And Residuals

- I247 local branch is rejected evidence and remains preserved for audit only.

## Retrospective

- Outcome: pending
