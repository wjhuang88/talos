# Iteration I184: TUI-046-A Native Selection Policy

> Document status: Planned
> Published plan date: 2026-08-10
> Planned objective: decide and document a predictable native text-selection/copy interaction for Issue #134 while preserving ADR-054's Alternate Screen and application-owned transcript boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an Accepted ADR-054 contract amendment (or replacement decision) plus a runnable cross-terminal validation plan that makes the TUI-046-B implementation boundary unambiguous.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-10 |
| Work Slice | Implement only I184/TUI-046-A: establish the native-selection versus mouse-capture contract, validate the causal interaction on the selected terminal matrix, and amend or replace ADR-054 with the explicit TUI-046-B gate; no Rust implementation or TUI-046-B authority. |
| Claimed At | 2026-08-10 |
| Source Issue | #134 |
| Governance Claim PR | #186 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent exact-head review is requested for this PR's final head and is recorded in PR #186; the approved SHA is backfilled at closeout. Shared-account natural-person disclosure required. Claim is proposed and remains ineffective until PR #186 merges. |
| Implementation PR | Not started |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | Finalize the claim on the target branch before editing ADR-054; TUI-046-B remains blocked until this decision is Accepted. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. The claim is ineffective until the
finalized record is merged into `main`.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-046-A | TUI-046 / Issue #134 | Ready | I183 Complete; current TerminalSession/input inventory | Accepted native-selection versus mouse-capture contract and ADR-054 amendment, with explicit TUI-046-B gate. |

### Scope

- Reproduce the current mouse-capture/native-selection conflict on the maintainer terminal and one materially different terminal/platform.
- Decide whether capture is disabled by default, explicitly configurable, gesture-overridden, or replaced by a bounded application-owned selection path.
- Amend ADR-054 or record a replacement decision, preserving Alternate Screen, transcript ownership, keyboard history navigation, privacy boundaries and `/copy` semantics.
- Define TUI-046-B's exact implementation, restoration, test and real-terminal acceptance contract.

### Non-Goals

- No Rust implementation, mouse-selection model, clipboard/export redesign or `/copy` behavior change.
- No TUI-042/#79 fix, permission/provider/session/runtime change, or abandonment of Alternate Screen.
- No TUI-046-B claim or implementation authorization.

### Acceptance

- Given the current lifecycle/input inventory, the decision identifies the causal interaction between mouse capture, native selection and wheel history without assuming Alternate Screen is the cause.
- Given the selected terminal matrix, the decision records observed default selection, override gestures, wheel behavior, redraw behavior and restoration results without overstating unsupported environments.
- ADR-054 or a replacement decision explicitly defines the default policy, keyboard history preservation, privacy boundary and the gate for TUI-046-B.
- The owner, Board and backlog state agree that TUI-046-B is Blocked until this decision is Accepted.

### Planned Validation

- Focused code/lifecycle inventory under `crates/talos-tui` and ADR-054 review.
- Recorded manual matrix on the maintainer's primary terminal and one materially different platform terminal. Every row names the exact terminal and version, OS/platform version, multiplexer and version or `none`, Talos SHA, gesture, wheel behavior, redraw condition, restoration result and copied-text observation; include Alacritty/Windows Terminal/macOS Terminal or iTerm2/tmux where applicable.
- `git diff --check`, `scripts/validate_project_governance.sh .`, `bash scripts/validate_collaboration_claims.sh .`, exact-head CI and independent architecture review.

### Documentation To Update

- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md` or a replacement decision.
- `docs/backlog/active/TUI-046-native-text-selection-copy.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, `docs/iterations/README.md`, and `.agent-governance/manifest.yaml`.
- Issue #134 with the exact decision/claim/validation links.

### Risks And Rollback

- Risk: selecting a policy that restores native selection but silently breaks wheel history or redraw stability.
- Rollback: keep ADR-054 unchanged and leave TUI-046-B Blocked until a narrower, independently reviewed contract is available.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-10 | Selection | I183/AG-7 completed at `edf903aa96574043294923ad60b0cefe9730f8c4`; no other active implementation iteration remains. TUI-046-A is selected as the P0 decision slice, pending effective claim. |

## Verification Evidence

- Pending effective claim and exact-head review.

## Completion Evidence

- Not applicable while Planned. A later closure must cite an already-existing decision/implementation evidence SHA.

## Variance And Residuals

- TUI-046-B remains Blocked until TUI-046-A is Accepted.
- Authorization Evidence review-state linkage is an existing validator/auditability gap owned by
  unclaimed GOV-004; I184 does not modify the collaboration validator.

## Retrospective

- Pending execution.
