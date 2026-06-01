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
- [ ] **End-to-end runtime evidence** recorded (see gate below)

### 3a. End-to-End Runtime Acceptance Gate (MANDATORY)

> Originating lesson: I008 passed all unit tests and every acceptance box was checked,
> yet the feature was a no-op at runtime because the library was never wired into the
> binary. Passing unit tests is **necessary but not sufficient**.

A story that changes observable behavior may be marked Done **only** when its capability
is exercised through the actual `talos` binary, not just isolated unit tests:

- [ ] The feature is reachable from a real run path (`talos ...` / TUI), not only from `#[test]`.
- [ ] There is at least one test or recorded manual transcript that drives the feature through
      the binary and asserts the user-visible result.
- [ ] Newly added library types are referenced by non-test runtime code
      (a `never used` / `never constructed` warning on a feature's core type is a **gate failure**).
- [ ] The evidence (command + observed output, or integration-test name) is pasted into the
      iteration file's Verification section.

If the runtime path is intentionally out of scope for this story, the story is **library-only**:
say so explicitly, and register the integration work as a residual item — do not mark the
behavior-facing parent story Done.

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
2. Confirm the End-to-End Runtime Acceptance Gate (3a) passed for every behavior-facing story;
   an iteration whose deliverable is not runnable end-to-end is **not** Complete — mark it Review.
3. Update iteration file with execution results, including the runtime evidence.
4. Update `docs/iterations/README.md` "Current Iterations" table state.
5. Write a brief retrospective: what worked, what didn't, lessons for `EVOLUTION.md`.
6. Record any decisions made during implementation in `docs/decisions/`.
7. Update `.agent-governance/manifest.yaml` `last_audited_at`.

## Doom Loop Prevention

If you find yourself:

- Re-reading the same files without making progress → stop, write down what's unclear, ask
- Trying 3+ approaches to the same problem → stop, record what failed, consult the architecture doc
- Adding features not in the iteration scope → stop, add to proposals, return to scope
