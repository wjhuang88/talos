# Iteration I046: Architecture, Structure, And Governance Repair

> Document status: Planned
> Published plan date: 2026-06-25
> Planned objective: Repair the model/config lifecycle regressions and reduce the structural
>   pressure introduced around I045 before new product features are selected.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `cargo test --workspace` passes again, inline `api_key` policy is recorded and
>   documented, duplicate model IDs resolve by `(provider, model_id)`, and model-switching code is
>   extracted out of the CLI mode runner into testable units.

## Published Baseline

### Non-Terminal Iteration Inventory

| Iteration | State | Disposition |
|---|---|---|
| Now | No active iteration | I046 remains Planned until explicitly activated. |
| I036 Research Consolidation | Planned | Unchanged; research-only work is not selected into I046. |
| I028 Delayed/Scheduled Tasks | Planned | Unchanged; feature work deferred until repair is complete. |
| I018 Observability and Prompt Assets | Planned | Partially superseded by I045 for log rotation/prompt assets; no new scope here. |
| I019 Layered Memory Foundation | Planned | Unchanged; depends on stable config/model/runtime boundaries. |
| I020 Exploration Library | Planned | Unchanged. |
| Blocked/Paused I011 S2 Provider Plugin Architecture | Paused | Unchanged; provider plugin architecture is not reopened here. |

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `I046-S1` Validation Baseline Repair | I045 closeout correction | Ready | Current failed `cargo test --workspace` evidence | Workspace tests pass and I045/manifest no longer claim stale validation. |
| `I046-S2` Model Identity Boundary | MODEL-004/MODEL-005 follow-up | Ready | Duplicate catalog IDs introduced in I045 | Model lookup, override, picker, and CLI selection use `(provider, model_id)` semantics without string-only ambiguity. |
| `I046-S3` Local Inline API Key Boundary | CONF-001 follow-up | Ready | User requirement: TOML `api_key` must be supported | ADR and docs state that inline `api_key` is allowed in private local config and masked everywhere else. |
| `I046-S4` Model Lifecycle Structure Cleanup | ARCH-011 watchlist | Ready | `mode_runners.rs` model-switch duplication | Model picker data and model transition preparation move out of `mode_runners.rs` into focused, tested modules. |
| `I046-S5` Documentation Synchronization | I045 closeout correction | Ready | S1-S4 decisions and fixes | README, config reference, backlog/iteration owners, and board agree with current behavior. |

### Scope

- Fix the stale model-limit test that still uses removed catalog IDs.
- Make model catalog merge and active-model resolution provider-aware.
- Preserve support for inline `providers.<name>.api_key` in `~/.talos/config.toml`.
- Record the inline-key security boundary in a new ADR or equivalent decision owner.
- Update AGENTS/config reference/README to describe inline keys, masking, and env-var guidance.
- Extract model picker data construction and model-switch session rebuild logic from
  `crates/talos-cli/src/mode_runners.rs`.
- Remove duplicated provider/session/agent rebuild logic between direct model switch and
  credential-assisted model switch.
- Add focused tests for duplicate model IDs, provider-qualified selection, inline-key round-trip,
  and masked display behavior.
- Correct stale completion evidence in I045 and derived board/manifest summaries where applicable.

### Non-Goals

- Do not implement new providers, new model marketplace behavior, or dynamic provider loading.
- Do not reopen provider plugin architecture or MODEL-003 reasoning/thinking support.
- Do not replace TOML inline keys with an OS keychain in this iteration.
- Do not broadly decompose unrelated large files such as `talos-tui/src/scrollback.rs` unless a
  local edit in this iteration directly requires it.
- Do not change permission/sandbox behavior except to preserve existing gates while refactoring.

### Acceptance

- Given the current workspace
  When `cargo test --workspace` is run
  Then it exits 0 with no unexpected ignored tests.

- Given duplicate built-in entries such as `zhipu/glm-5.2` and `zai/glm-5.2`
  When a user selects or configures a provider-qualified model
  Then Talos activates the intended provider and only that provider's model metadata/overrides.

- Given a user stores `providers.anthropic.api_key` in local `~/.talos/config.toml`
  When Talos loads and saves config
  Then the key is preserved in the local file and never printed by config display commands.

- Given a user runs `talos --config-list` or `talos --config-get providers.anthropic.api_key`
  When the config contains an inline key
  Then the displayed value is masked.

- Given `/model` opens the picker
  When the user selects a ready model or completes provider credential setup
  Then the same tested transition path prepares and commits the new session runtime.

- Given an Agent reads AGENTS.md, README, config reference, I045, and I046
  When it reasons about `api_key`, model IDs, and current validation status
  Then those documents do not contradict each other.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo test -p talos-config`
- `cargo test -p talos-cli`
- `cargo test -p talos-tui`
- Manual or scripted CLI check proving `--config-list` and `--config-get providers.<name>.api_key`
  mask an inline key.
- Manual or scripted TUI/conversation check proving provider-qualified duplicate model selection
  reaches the intended provider.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `AGENTS.md`
- `README.md`
- `docs/reference/config.reference.toml`
- `docs/decisions/README.md`
- New decision record for local inline API key boundary, if not already present.
- `docs/iterations/I045-product-readiness-model-lifecycle-observability.md`
- `docs/iterations/I046-architecture-structure-governance-repair.md`
- `docs/BOARD.md`
- `.agent-governance/manifest.yaml` if status notes or validation claims are updated.

### Risks And Rollback

- Risk: Provider-qualified model selection changes persisted config semantics.
  Rollback: Keep unqualified model IDs accepted where unique; only require provider qualification
  for duplicates.

- Risk: Refactoring model switching breaks first-run onboarding.
  Rollback: Preserve the existing public `SessionLifecycleRequest` flow and extract behind it
  before changing behavior.

- Risk: Inline-key policy is implemented in docs but missed in output surfaces.
  Rollback: Treat display masking tests as required before closure; leave implementation partial
  until all known display paths are covered.

- Risk: Governance repair expands beyond the iteration.
  Rollback: Register unrelated stale docs as residuals instead of broad rewriting.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-25 | Planning | Created from post-I045 review. Current workspace is ahead of `origin/main` by two commits (`a8cd614`, `0734eae`) and has no active iteration on `docs/BOARD.md`. |
| 2026-06-25 | Activation | I046 activated. S1: fixed two failing tests — `test_model_limits_from_builtin_and_custom_providers` (stale `gpt-4.1` → `gpt-4.1-2025-04-14`, switched to `resolve_model_limits()`) and `test_session_picker_accept_resume_default_command` (I045 `PanelItemAction` refactor lost `/resume` fallback for empty command). `cargo test --workspace` passes. S2: made model identity provider-aware — added `find_model_by_provider`/`models_with_id`, fixed `resolve_model_limits`/`all_models`/`set_active_model` fallback, added 9 tests for duplicate model IDs. S3: custom `Debug` impls masking `api_key` on `ProviderConfig`/`Credentials`/`CredentialResponseData`, added `api_key` case to `config_get_dotted`, masking tests, ADR-023. S4: extracted model lifecycle module. S5: fixed stale `config.reference.toml` (`skip_serializing` → persisted), updated AGENTS.md constraint #3 and task router, updated decisions README. |

## Verification Evidence

- `cargo check --workspace` — clean (2026-06-25)
- `cargo clippy --workspace -- -D warnings` — clean (2026-06-25)
- `cargo test --workspace` — all pass, 0 failures (2026-06-25). Notable new tests:
  - `test_resolve_model_limits_provider_aware_for_duplicate_ids` — zhipu/zai glm-5.2 resolves correctly
  - `test_set_active_model_errors_on_ambiguous_bare_id` — bare `glm-5.2` errors with provider list
  - `test_set_active_model_provider_qualified_resolves_correctly` — `zai/glm-5.2` → zai provider
  - `test_all_models_user_override_matches_by_provider_and_id` — override hits zai, not zhipu
  - `test_provider_config_debug_masks_api_key` / `test_config_debug_masks_provider_api_keys` / `test_credentials_debug_masks_keys`
  - `mask_secrets_masks_api_key_lines` / `is_secret_key_detects_api_key_paths` / `config_get_dotted_returns_api_key_value`
  - `model_lifecycle::tests::*` — 4 tests for extracted picker data construction
- `scripts/validate_project_governance.sh .` — 0 warnings (2026-06-25)

## Variance And Residuals

- I046 is a repair iteration, not new product behavior. It intentionally groups validation,
  structure, and governance corrections because they share the same I045 model/config boundary.
- Residual watchlist outside this iteration: broader decomposition of `talos-tui/src/scrollback.rs`,
  `talos-tui/src/app.rs`, large test files, and future provider plugin architecture.

## Retrospective

- Outcome: Complete. All 5 stories (S1-S5) delivered. `cargo test/clippy/governance` all pass.
- Documentation: ADR-023 created, AGENTS.md/README/config.reference.toml updated, I045 stale evidence corrected.
- Lessons: I045 closed with stale `cargo test --workspace` claims because the catalog-update commit (`0734eae`) landed AFTER the I045 closeout check. Future iterations that update shared datasets (models.toml) must re-run workspace tests after the dataset change, not just after the feature commit.
