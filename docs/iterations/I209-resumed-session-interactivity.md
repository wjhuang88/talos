# Iteration I209: Resumed Session Interactivity Under Provider Delay

> Document status: Planned / Unclaimed
> Planned date: 2026-08-17
> Objective: deliver TUI-051 so a resumed large Session remains responsive, exposes bounded
> provider retry progress and can cancel an active turn promptly.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #272 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | After #271 disposition and an exact-main reproduction checkpoint, establish an effective claim and branch only from its merge point or later current main. |

## Selected Story

- `TUI-051` - `docs/backlog/active/TUI-051-resumed-session-interactivity.md`

## Activation Gate

- #271/PROVIDER-005 is Complete, cancelled or otherwise explicitly dispositioned so provider-file
  overlap is not hidden.
- The exact loss point for resumed-turn `Esc` is reproduced across the TUI, bridge, actor and
  cancellation-token boundaries.
- Current Active, Review, Planned, Blocked and Paused work is inventoried and dispositioned.
- An effective target-branch Collaboration Claim exists before an implementation branch is made.

## Runnable Deliverable

A rebuilt `talos -c` demonstrates that a persisted large Session no longer consumes a full CPU
core while unchanged, shows bounded provider retry progress, and durably cancels a delayed resumed
turn through `Esc`, with deterministic regression tests, terminal-mode restoration and a
real-terminal trace.

## Scope

- Existing OpenAI-compatible dispatch/retry status projection and cancellation responsiveness.
- Generation-bound resumed structured-turn interruption.
- TUI history-projection caching/invalidation for large unchanged transcripts.
- Focused, workspace and real-terminal verification plus user-facing status/help documentation.

## Exclusions

- #271 UTF-8 transport decoding, timeout-default or retry-policy redesign.
- I200/#79 scroll semantics and I206/TUI-048 steering activation.
- Transcript deletion, persistence migration, compaction policy, public API changes or a broad TUI
  rendering refactor.

## Acceptance

- [ ] Delayed response headers and retry backoff expose bounded, changing status instead of a
      static indefinite `connecting...` label.
- [ ] `Esc` promptly cancels initial dispatch, retry backoff and first-packet wait after resume,
      with durable terminal-cancelled evidence.
- [ ] Unchanged large history reuses its projection; transcript/viewport/style/selection changes
      invalidate it correctly.
- [ ] A real persisted transcript comparable to the incident remains input-responsive without
      sustained full-core redraw work.
- [ ] Supported termination signals restore raw/alternate-screen terminal state before returning
      control to the shell.
- [ ] Locked focused/workspace checks, governance validators and `git diff --check` pass at the
      reviewed exact head.
- [ ] TUI help/troubleshooting, Issue #272, owner status and derived views are synchronized.

## Planned Validation

```bash
cargo test -p talos-provider --locked
cargo test -p talos-agent --locked
cargo test -p talos-cli --locked
cargo test -p talos-tui --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
COLLABORATION_VALIDATION_BASE=origin/main bash scripts/validate_collaboration_claims.sh .
git diff --check
```

## Current Disposition

I209 is Planned / Unclaimed in the upcoming task pool. It follows #271 disposition and precedes
I205 plus the existing long-running mainline sequence because the observed interaction failure can
block reliable execution and review in large Sessions. This ordering grants no implementation
authority.
