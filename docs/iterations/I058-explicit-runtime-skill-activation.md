# Iteration I058: Explicit Runtime Skill Activation

> Document status: Review
> Published plan date: 2026-06-27
> Planned objective: Implement explicit Level 1 Skill body and bounded Level 2 reference activation
>   without leaking skill content into history or destabilizing prompt-cache behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a runnable Talos binary where `/skills activate <name>` makes the next provider
>   request include the selected Skill body, `/skills` reports the active state without dumping the
>   body, and activation has deterministic prompt-cache invalidation evidence.

## Published Baseline

### Non-Terminal Iteration Inventory

| Iteration | Current State | Disposition |
|---|---|---|
| I011 | Paused | Not reopened. |
| I018 | Planned | Baseline preserved; OBS slice was fulfilled through I047. |
| I019 | Review | Not reopened; memory follow-ups remain under MEM owners. |
| I020 | Review | Not reopened; exploration follow-ups remain under RES/WEBFETCH owners. |
| I028 | Planned | Deferred to the R27 permission-sensitive packet. |
| I047 | Review | Awaiting release workflow evidence; no release action in I058. |
| I048 | Planned | Baseline preserved; user-facing continuation delivered in I049. |
| I049-I057 | Review | Preserve execution records; append only if validation finds a regression. |
| R27 | In Progress | Owns high-risk oversight sequencing; I058 is R27/T2's implementation carrier. |

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SKILL-002` | Runtime Skill Activation | Ready after R27/T2 design pass | I033 complete; CMD-001 complete; ARCH-006 cache contract | Explicit Skill body/reference activation reaches provider context through a typed session owner with cache invalidation tests. |

### Readiness Decision

SKILL-002 does not require a new ADR because it does not change a public protocol or dependency
boundary. It implements the existing SKILL-001 residual through the accepted CMD-001 and ARCH-006
contracts.

The implementation owner is split deliberately:

- `talos-cli::skill_runtime` owns runtime discovery, SkillManager state, path-confined body/reference
  loading, activation budgets, and diagnostics.
- `talos-agent` owns model-visible activated Skill context through a typed prompt-builder field.
- Activated Skill content is part of the cacheable stable prefix after activation. Activation
  invalidates `cached_stable_prefix`; subsequent turns reuse the rebuilt prefix until activation
  changes again.
- Conversation/TUI command handling must route activation through a typed runtime/session operation,
  not by appending Skill content to chat history or scrollback.

### Scope

- Extend runtime skill state so it can load one active Level 1 `SKILL.md` body by name with a bounded
  byte/token budget.
- Add explicit command semantics for:
  - `/skills` to list Level 0 metadata and active state only.
  - `/skills activate <name>` to activate one discovered Skill.
  - `/skills reference <relative-path>` or equivalent bounded reference loading for the active Skill.
- Add path confinement for references: no absolute paths, no `..` escape, no symlink-following escape
  outside the active Skill directory.
- Inject activated body/reference content into provider context without writing the content into
  conversation history, scrollback history, exported visible transcript, or diagnostics.
- Invalidate and rebuild the stable prompt prefix when activated Skill context changes.
- Preserve existing Level 0 startup discovery behavior from I033.

### Non-Goals

- No automatic fuzzy activation.
- No loading all Skill bodies or references at startup.
- No arbitrary commands declared by Skill files.
- No plugin command or WASM integration.
- No SkillSidebar rendering.
- No network/resource loading from Skill files.

### Acceptance

- Given a workspace Skill named `review`,
  When the user runs `/skills activate review`,
  Then the next provider request contains the bounded Level 1 body and `/skills` reports `review`
  active without printing the body.

- Given an unknown Skill name,
  When the user runs `/skills activate missing`,
  Then active Skill state remains unchanged and the user receives a deterministic error.

- Given an active Skill with a reference file under its directory,
  When the user explicitly loads that reference,
  Then bounded reference content reaches model context and diagnostics show only path/status metadata.

- Given a reference path that is absolute, contains `..`, escapes through canonicalization, or
  exceeds the budget,
  When loading is attempted,
  Then Talos rejects or truncates deterministically without crashing.

- Given activated Skill context changes,
  When the next turn builds the provider prompt,
  Then the stable prefix cache is invalidated once, rebuilt deterministically, and remains stable
  across later turns until activation changes.

- Given transcript export or visible history,
  When a Skill is activated or a reference is loaded,
  Then the full Skill body/reference content is not dumped into scrollback-visible command output.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- Targeted `talos-skill` tests for reference path confinement and budget handling.
- Targeted `talos-agent` tests for activated Skill prompt section and stable-prefix invalidation.
- Targeted conversation/CLI tests for `/skills activate`, unknown Skill, and active diagnostics.
- Real `talos` binary request-preview scenario proving activated Skill content reaches provider
  context while diagnostics stay bounded.

### Documentation To Update

- `README.md`
- `README.zh-CN.md` if English README user behavior changes are mirrored there.
- `docs/backlog/active/SKILL-002-explicit-runtime-activation.md`
- `docs/backlog/active/SKILL-001-runtime-skill-activation.md` only for residual closure note.
- `docs/BOARD.md`
- `docs/tasks/2026-06-27-personal-oversight-high-risk-roadmap.md`

### Risks And Rollback

- Risk: Activation mutates prompt context but not the cached stable prefix.
  Rollback: keep `/skills activate` disabled and continue Level 0-only behavior.
- Risk: Reference loading permits path escape or oversized context.
  Rollback: ship Level 1 body activation only and defer Level 2 references.
- Risk: Command routing cannot safely mutate the live agent/session.
  Rollback: keep the iteration Planned and route the prerequisite through SESSION/CMD follow-up
  before implementation.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-27 | Planning | Created under R27/T2 after code inspection confirmed Level 0 runtime discovery exists, but Level 1/2 activation needs a typed session owner plus prompt-cache invalidation tests. |
| 2026-06-27 | Implementation | Added activated Skill prompt context in `talos-agent`, typed `SessionOp::SetSkillContext`, runtime activation/reference loading in `talos-cli`, typed `/skills activate` and `/skills reference` command routing in `talos-conversation`, and bridge handling that keeps full Skill content out of visible command output. |
| 2026-06-27 | Documentation | Updated `README.md`, `README.zh-CN.md`, SKILL-002, Product Backlog, Board, and the R27 long-task checkpoint to reflect explicit Skill activation behavior and remaining verification work. |
| 2026-06-27 | Validation | Full workspace format, check, clippy, tests, governance validation, and diff hygiene passed. Kept status Active because the real interactive binary request-preview proof remains outstanding. |
| 2026-06-27 | Runtime Evidence | Added inline real-binary request-preview regression for `/skills activate <name>`, proving activated Skill body content reaches provider context without relying on unit-only session proof. Moved I058 to Review. |

## Verification Evidence

- `cargo check -p talos-agent -p talos-conversation -p talos-cli -p talos-tui`
  - Result: passed.
- `cargo test -p talos-agent -p talos-conversation -p talos-cli skill -- --nocapture`
  - Result: passed.
  - Covered activated prompt section, request-preview propagation, runtime activation diagnostics,
    unknown Skill errors, reference confinement/budget behavior, and `/skills` command routing.
- `cargo test -p talos-agent set_skill_context_reaches_request_preview -- --nocapture`
  - Result: passed.
- `cargo test -p talos-cli conversation_loop_routes_skill_activation_to_session_op -- --nocapture`
  - Result: passed.
- `cargo clippy -p talos-core -p talos-agent -p talos-conversation -p talos-cli -p talos-tui -- -D warnings`
  - Result: passed.
- `cargo fmt --all -- --check`
  - Result: passed after applying `cargo fmt --all`.
- `cargo check --workspace`
  - Result: passed.
- `cargo clippy --workspace -- -D warnings`
  - Result: passed.
- `cargo test --workspace`
  - Result: passed.
- `scripts/validate_project_governance.sh .`
  - Result: passed with 0 warnings.
- `git diff --check`
  - Result: passed.
- `cargo test -p talos-cli --test skill_runtime_e2e -- --nocapture`
  - Result: passed.
  - Covered real `talos --inline --mock` binary flow: create workspace Skill, run
    `/skills activate review`, run `/mock-request`, and verify the activated Skill body appears in
    the provider request preview.

## Variance And Residuals

- Real binary proof uses inline mode because it can be scripted deterministically while still
  exercising the compiled `talos` executable, runtime Skill activation, typed session context, and
  provider request preview. TUI bridge behavior is separately covered by conversation-loop tests.
- No rollback has been needed.

## Retrospective

- Review state reached after adding real-binary evidence. Keep Complete gated on the normal review
  decision and any release packaging policy outside I058.
