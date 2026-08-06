# ARCH-034-R06: Conversation Engine Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F22 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | Not selected |
| Preserved behavior | ConversationEngine API, output ordering, commands, steering, and transcripts |

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
