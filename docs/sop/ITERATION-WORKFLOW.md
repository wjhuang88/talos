# SOP: Iteration Workflow

## Purpose

Define how to execute work during an active iteration.

## Daily Loop

### 1. Pick a Story

From the active iteration, select the next story:

1. Prefer stories whose dependencies are complete.
2. Prefer stories that unblock others.
3. Prefer stories that reduce risk (spikes, core infrastructure).

### 2. Implement

Follow `NEW-FEATURE.md` for feature work:

1. Read affected crate code and existing patterns.
2. Write tests first when possible (see `TESTING.md`).
3. Implement the minimum that satisfies acceptance criteria.
4. Run `cargo check --workspace`, `cargo clippy --workspace`, `cargo test --workspace`.
5. Fix all errors and warnings before proceeding.

### 3. Verify

For each story:

- [ ] All acceptance criteria pass
- [ ] `cargo test --workspace` exits 0
- [ ] `cargo clippy --workspace` has no warnings
- [ ] New public items have doc comments
- [ ] No `unwrap()` in library code
- [ ] No `unsafe` without an ADR in `docs/decisions/`
- [ ] `README.md` updated to reflect changes (features, usage, architecture)

### 4. Commit

Follow `GIT-WORKFLOW.md`:

- One commit per completed story.
- Reference the story ID in the commit message.
- Review `git diff --cached` before committing.

### 5. Update Status

Mark the story as "Done" in the backlog and iteration file.
Record any residual work or follow-up items.

## Completion

When all stories are done:

1. Run full verification: `cargo test --workspace && cargo clippy --workspace`
2. Update iteration file with execution results.
3. Write a brief retrospective: what worked, what didn't, lessons for `EVOLUTION.md`.
4. Record any decisions made during implementation in `docs/decisions/`.

## Doom Loop Prevention

If you find yourself:

- Re-reading the same files without making progress → stop, write down what's unclear, ask
- Trying 3+ approaches to the same problem → stop, record what failed, consult the architecture doc
- Adding features not in the iteration scope → stop, add to proposals, return to scope
