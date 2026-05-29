# SOP: New Feature Implementation

## Purpose

Define the process for implementing a new feature during an active iteration.

## Process

### 1. Understand the Story

Before writing any code:

1. Read the story's acceptance criteria in the backlog.
2. Identify which crates are affected.
3. Read existing code in those crates to understand patterns.
4. Check `docs/reference/ARCHITECTURE.md` for crate boundaries and trait definitions.
5. State your assumptions. If uncertain, ask.

### 2. Design the Interface

For features that affect public APIs:

1. Define the trait or type signature first.
2. Consider: does this need dynamic dispatch (`dyn Trait`) or is `impl Trait` sufficient?
3. Check: does this introduce a new dependency between crates? If yes, is it warranted?
4. If the design affects Soft or Assumption constraints → create an ADR.

### 3. Write Tests

Follow `TESTING.md`:

1. Write failing tests that define the acceptance criteria.
2. For protocol/parsing code → use `proptest` for property-based testing.
3. For async code → use `tokio::test` with proper timeout handling.

### 4. Implement

Rules:

- Implement the minimum that satisfies acceptance criteria.
- Use `thiserror` for library crate errors, `anyhow` for binary crate errors only.
- Never `unwrap()` in library code. Use `?` and proper error types.
- Never suppress type errors with `as any` equivalents (`std::mem::transmute`, unchecked casts).
- All `unsafe` requires an ADR in `docs/decisions/`.
- All public items get `///` doc comments.

### 5. Verify

Before marking the story done:

```bash
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

All must pass with zero errors.

### 6. Commit

Follow `GIT-WORKFLOW.md`:

- One logical change per commit.
- Conventional commit with story ID.
- Review staged diff before committing.

## Crate-Specific Guidelines

### Adding a Tool (talos-tools)

1. Implement the `AgentTool` trait.
2. Register in the tool registry.
3. Add unit tests for the tool logic.
4. Add permission rules if the tool modifies files.

### Adding a Provider (talos-provider)

1. Implement the `LanguageModel` trait.
2. Handle streaming via the event protocol.
3. Add compatibility tests with mock responses.
4. Document API quirks in the provider module.

### Adding a Permission Rule (talos-permission)

1. Security-sensitive. Review against escape vectors.
2. Test with adversarial inputs.
3. Ensure rules are evaluated in the correct order.
4. Document the rule in the permission module docs.
