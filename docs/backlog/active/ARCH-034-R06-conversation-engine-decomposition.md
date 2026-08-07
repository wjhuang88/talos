# ARCH-034-R06: Conversation Engine Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F22 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | I175 (Planned; Claim PR #154) |
| Preserved behavior | ConversationEngine API, output ordering, commands, steering, and transcripts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Private slash-command dispatch and transcript/extension projection source decomposition behind the current `ConversationEngine` facade, preserving public paths, turn/steering state ownership, command semantics, exact output text/order, transcript formats, plugin/skill behavior, and extension snapshot contents. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #154 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if the split requires any public API, state-ownership, command/output, transcript, extension snapshot, plugin/skill, dependency, or behavior change. |

## Problem And Boundary

`talos-conversation/src/engine.rs` combines turn state, agent event projection, slash commands,
steering custody, transcript export, and extension snapshots in 1,651 production lines.

## Scope

- Extract private slash-command and transcript/extension projection helpers.
- Keep `ConversationEngine` state ownership and all public methods at their current paths.

## Exclusions

- No command, output text, queue semantics, plugin/skill behavior, or public API change.

## Acceptance And Validation

- Command helpers do not own mutable turn state; engine transitions remain centralized.
- Exact `UiOutput` sequences and transcript formats remain covered by existing tests.
- Locked workspace, public API, governance, and diff checks pass.

## Rollback / Residual

Revert if output/state equivalence is not exact. New commands require a feature story.
