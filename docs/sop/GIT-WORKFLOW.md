# SOP: Git Workflow

## Purpose

Define commit, branching, and PR conventions for the Talos project.

## Commit Rules

### Before Committing

1. **Review the staged diff**: `git diff --cached`
2. **Verify**: No secrets, no unintended changes, no debug code
3. **Check**: Does every changed line trace to a requirement?
4. **Run**: `cargo check --locked --workspace && cargo clippy --locked --workspace -- -D warnings && cargo test --locked --workspace`

For workspace or release validation, prefer `./scripts/release_preflight.sh` so local and CI
checks cannot drift. The pinned toolchain is defined in `rust-toolchain.toml`.

### Commit Messages

Format: `type(scope): description (#story-id) [model:<model-name>]`

- `(#story-id)` may be omitted for project-level changes with no associated story.
- `[model:<model-name>]` is required when an Agent authored or co-authored the commit, identifying the AI model used.

Types:

| Type | When |
| --- | --- |
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `refactor` | Code restructuring without behavior change |
| `docs` | Documentation changes |
| `test` | Adding or updating tests |
| `chore` | Build, CI, tooling changes |

Scopes: crate name (`core`, `agent`, `tools`, `sandbox`, `permission`, `provider`, `session`,
`skill`, `plugin`, `mcp`, `config`, `cli`, `rpc`) or `workspace` for cross-crate changes.

Examples:

```
feat(agent): implement SQ/EQ turn loop (#E2-S1)
fix(sandbox): prevent symlink escape in bwrap (#E3-S1)
refactor(provider): extract streaming trait (#E2-S3)
docs(reference): add crate dependency graph
test(core): add proptest for message serialization (#E1-S2)
chore(workspace): set up CI pipeline (#E1-S5)
```

### Commit Hygiene

- One logical change per commit. No mixed concerns.
- Never commit secrets. Check for API keys, tokens, passwords.
- Never force-push to `main`.
- Never move or force-push a release tag. If a tag workflow fails, correct the source and use a new
  patch version/tag.
- Keep commits atomic and reorderable.

## Branching

### Branch Names

```
{type}/{story-id}-{short-description}

# Examples:
feature/E2-S1-sq-eq-turn-loop
fix/E3-S1-symlink-escape
```

### Workflow

1. Create branch from `main`.
2. Implement and commit with story ID references.
3. Run full verification before PR.
4. Create PR with description linking to the story.
5. Squash or rebase based on review preference.

## PR Rules

- PR description must reference the backlog story ID.
- All CI checks must pass.
- No merge without review (at least one reviewer for security-sensitive code).
- Sandbox/permission changes require explicit security review sign-off.
