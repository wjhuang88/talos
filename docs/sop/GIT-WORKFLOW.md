# SOP: Git Workflow

## Purpose

Define commit, branching, review, and PR conventions for the Talos project.

Task ownership, adoption, exceptions, authorization, and the governance-before-implementation
sequence are defined by [`AGENT-COLLABORATION.md`](AGENT-COLLABORATION.md).

## Commit Rules

### Before Committing

1. **Review the staged diff**: `git diff --cached`
2. **Verify**: No secrets, no unintended changes, no debug code
3. **Check**: Does every changed line trace to a requirement or an allowed bounded-maintenance
   exception?
4. **Run**: `cargo check --locked --workspace && cargo clippy --locked --workspace -- -D warnings && cargo test --locked --workspace`

For workspace or release validation, prefer `./scripts/release_preflight.sh` so local and CI checks
cannot drift. The pinned toolchain is defined in `rust-toolchain.toml`.

When governance files change, also run:

```bash
scripts/validate_project_governance.sh .
scripts/validate_collaboration_claims.sh .
```

### Commit Messages

Format: `type(scope): description (#story-id) [model:<model-name>]`

- `(#story-id)` may be omitted for project-level or bounded single-PR maintenance with no owner.
- `[model:<model-name>]` is required when an Agent authored or co-authored the commit.

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
feat(agent): implement SQ/EQ turn loop (#E2-S1) [model:gpt-5.6-thinking]
fix(sandbox): prevent symlink escape (#E3-S1) [model:gpt-5.6-thinking]
docs(reference): correct broken architecture link [model:gpt-5.6-thinking]
```

### Commit Hygiene

- One logical change per commit. No mixed concerns.
- Never commit secrets.
- Never force-push to `main`.
- Never move or force-push a release tag. Correct source and use a new patch tag.
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

### Governed Workflow

For new governed work:

1. Complete `AGENT-COLLABORATION.md` preflight.
2. Create a governance-only claim branch from the current target branch.
3. Open a Draft claim PR to obtain its number.
4. Finalize the owner record on that branch with `Claim State: Claimed`, the real PR number,
   bounded Work Slice, and authorization fields.
5. Run both governance validators and exact-head CI.
6. Repeat the merge-time compare-and-swap preflight.
7. Merge using an allowed authorization path.
8. The same merge establishes claim and activation; refresh the target branch.
9. Create the implementation branch from that merge commit or a later target commit.
10. Converge implementation, tests, documentation and owner Review state locally.
11. Push and open one stable stage candidate only after the local checkpoint passes.
12. After implementation merge, close with existing implementation evidence; when safe, combine
    that owner-first closeout with the next non-overlapping atomic claim+activation PR.

An open claim PR does not reserve the task. Proposed `Claimed` content is ineffective until it
exists on the target branch.

Do not create the implementation branch, commit/push implementation, change production dependencies,
or open a draft implementation PR before the effective claim, except for an authorized emergency.

Read-only investigation and disposable uncommitted experiments are allowed before claim merge but do
not establish ownership.

For work already effectively claimed on the target branch, begin at step 8 after verifying the
claim, Work Slice, and absence of overlapping PRs.

### Bounded Maintenance And Existing-PR Follow-Ups

A bounded single-PR exception or reviewer-only follow-up follows the applicability rules in
`AGENT-COLLABORATION.md`. The PR description must name the exception and explain why it does not
change behavior, API, security, dependencies, release authorization, persistent data, owner state,
or scope.

### Emergency Workflow

Emergency changes may implement before the normal claim merge only with maintainer authorization and
the minimum emergency record required by `AGENT-COLLABORATION.md`. Reconcile governance within two
business days after containment.

## Authorization And Review

Use one of these merge paths:

- **Independent review**: preferred for normal work and mandatory for security-sensitive, sandbox,
  permission, process-hardening, or explicitly protected scope.
- **Single-maintainer merge**: allowed when no independent reviewer is available and exact-head CI,
  `validate_project_governance.sh`, and `validate_collaboration_claims.sh` pass; the PR records the
  reason and has no unresolved blocking review feedback.
- **Direct commit**: allowed only when repository policy explicitly permits it and a maintainer
  records reason and validation. It is not a normal review bypass.
- **Emergency override**: limited to the emergency conditions and reconciliation requirements in
  `AGENT-COLLABORATION.md`.

The PR author does not approve their own PR under the single-maintainer path. Security-sensitive code
still requires independent security review unless the emergency path explicitly applies.

## Merge-Time CAS Checklist

Immediately before merging a claim PR:

- [ ] Branch includes the latest target branch or GitHub reports it cleanly mergeable.
- [ ] Target owner still has no conflicting effective claimant.
- [ ] No new overlapping claim or implementation PR exists.
- [ ] Responsible Actor and Work Slice still match owner truth.
- [ ] Dependencies and activation gates remain satisfied.
- [ ] Governance Claim PR field matches the actual PR.
- [ ] Exact-head CI and both governance validators passed.
- [ ] Authorization mode and evidence are complete.
- [ ] No unresolved blocking review feedback remains.

Any changed item invalidates the previous review/validation snapshot and requires refresh.

## PR Rules

### Governance Claim PR

- Governance-only: owner creation/correction, claim record, inventories, Board, and directly required
  governance validation changes.
- No production implementation, implementation tests, speculative dependencies, or generated
  implementation artifacts.
- Draft until the actual PR number is backfilled and the proposed claim is complete.
- Must pass the merge-time CAS checklist.

### Implementation PR

- References Story/iteration/task ID, source Issue when present, owner document, and merged claim PR.
- Implements only the recorded Work Slice.
- Records validation and residuals.
- Is first pushed only after the local stable-candidate checkpoint; GitHub CI is stage validation,
  not an edit-by-edit loop.
- Normally leaves delivery state at `Review`; an unmerged commit cannot be Completion Commit evidence.
- Uses the existing PR for batched reviewer corrections. Substantive new heads require new exact-
  head evidence, but not a new PR.

### Governance Closure

- Occurs after implementation merge.
- Cites an already-existing target-branch implementation or merge SHA.
- Synchronizes owner, inventories, Board, and Issue in that order.
- May share the next non-overlapping claim+activation PR only when the previous Completion Commit
  already exists and all previous owner/derived/Issue state is closed before the next activation.

### General

- All required CI checks must pass.
- Security-sensitive changes require explicit security review sign-off unless an emergency override
  is recorded.
