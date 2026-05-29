# SOP: Change Control

## Purpose

Define how to handle requirement changes after an iteration has started.

## Principles

- Changes are expected. The process exists to make them visible, not to prevent them.
- Every change must be evaluated for impact before implementation.
- No silent scope expansion.

## Process

### 1. Identify the Change

When a requirement changes mid-iteration:

1. Write down the change request clearly.
2. Identify which stories are affected.
3. Classify the change:

| Classification | Meaning | Action |
| --- | --- | --- |
| **In-scope correction** | Fixes a misunderstanding of existing scope | Update the story, implement |
| **Scope addition** | New work not in the original iteration | Evaluate, defer or add |
| **Scope reduction** | Work no longer needed | Remove story, mark as cancelled |
| **Priority shift** | Same work, different order | Reorder stories |
| **Blocker** | External dependency changed | Mark blocked, record in iteration |

### 2. Assess Impact

For scope additions:

- Which crates are affected?
- Does it create new dependencies?
- Does it affect the iteration timebox?
- Does it conflict with Hard Constraints in `AGENTS.md`?

### 3. Decide

- **Defer** — Add to backlog for a future iteration (most common for additions)
- **Accept** — Add to current iteration with updated scope and acceptance criteria
- **Reject** — Record reason in iteration notes

### 4. Update

- Update the iteration file with the change decision.
- Update affected stories in the backlog.
- If the change invalidates completed work, record what needs rework.

## Rules

- Scope additions larger than size S must be deferred unless explicitly approved.
- Changes to Hard Constraints require a decision record in `docs/decisions/`.
- Changes that affect crate public APIs require a decision record.
