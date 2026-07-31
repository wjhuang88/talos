# SOP: Git Workflow

## Purpose

Define commit, branching, and PR conventions for the Talos project.

Task ownership and the required governance-before-implementation sequence are defined by
[`AGENT-COLLABORATION.md`](AGENT-COLLABORATION.md). This SOP must not be used to bypass that claim
gate.

## Commit Rules

### Before Committing

1. **Review the staged diff**: `git diff --cached`
2. **Verify**: No secrets, no unintended changes, no debug code
3. **Check**: Does every changed line trace to a requirement?
4. **Run**: `cargo check --locked --workspace && cargo clippy --locked --workspace -- -D warnings && cargo test --locked --workspace`

For workspace or release validation, prefer `./scripts/release_preflight.sh` so local and CI
checks cannot drift. The pinned toolchain is defined in `rust-toolchain.toml`.

When governance files change, also run:

```bash
scripts/validate_project_governance.sh .
```

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
docs/E2-S1-claim-sq-eq-turn-loop
feature/E2-S1-sq-eq-turn-loop
fix/E3-S1-symlink-escape
```

### Workflow

For a newly claimed Issue or task item:

1. Complete the preflight in `AGENT-COLLABORATION.md`.
2. Create a governance-only claim branch from the current target branch.
3. Submit and merge the governance claim PR. An open claim PR does not reserve the task.
4. Refresh the target branch after the claim merges.
5. Create the implementation branch from the claim merge commit or a later target-branch commit
   containing the effective claim.
6. Implement and commit with Story or iteration ID references.
7. Run full verification before the implementation PR.
8. Create the implementation PR linking the source Issue, owner document, and governance claim PR.
9. After implementation merges, submit a separate governance closure update citing the existing
   implementation commit as completion evidence.

Do not create the implementation branch, commit or push implementation changes, change production
dependencies for the task, or open a draft implementation PR before the governance claim is present
on the target branch.

Read-only investigation and disposable uncommitted experiments are permitted before claim merge,
but they do not establish ownership and must not be represented as implementation progress.

For work that is already effectively claimed on the target branch, begin at step 4 after verifying
the claim remains valid and no overlapping implementation PR exists.

## PR Rules

- A governance claim PR and its implementation PR are separate changes.
- A governance claim PR contains no production implementation or implementation tests.
- A pending governance claim PR is not an effective reservation.
- An implementation PR must reference the backlog Story or iteration ID, source Issue when one
  exists, owner document, and merged governance claim PR.
- The implementation PR normally leaves delivery status at `Review`; it cannot use its unmerged
  implementation commit as `Completion Commit` evidence.
- Governance closure occurs after implementation merge and cites an already-existing target-branch
  implementation or merge commit.
- All CI checks must pass.
- No merge without review (at least one reviewer for security-sensitive code).
- Sandbox/permission changes require explicit security review sign-off.
