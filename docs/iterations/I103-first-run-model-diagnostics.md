# Iteration I103: First-Run Model And Diagnostics

> Document status: In Progress
> Published plan date: 2026-07-07
> Planned objective: Execute Month 2 of the 2026-07-07 four-month developer operating plan by
> making first-run provider setup, model selection, and diagnostics usable for controlled trials.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a new developer can configure a standard provider, browse/select a model, and
> produce a redacted diagnostic report without editing source files.
> Activated: 2026-07-08

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D110 | MODEL-006/MC-001 | Planned | I102 closeout or explicit activation | Standard providers skip base-url prompts; custom providers require a URL. |
| D111 | MODEL-006 | Planned | D110 | Large model inventories are bounded and searchable in CLI/TUI surfaces. |
| D112 | VALIDATION-001/CONF-001 | Planned | D110 | Redacted diagnostics report config, provider, credential source, data dirs, and validation adapters. |
| D113 | Developer operating plan | Planned | D110-D112 | First-run docs and setup evidence are synchronized. |

### Scope

- Verify and polish `/connect` standard-provider and custom-provider flows.
- Keep model browsing responsive for large catalogs without reintroducing runtime `catalog.db`.
- Add or refine a redacted diagnostic path using existing config and validation services.
- Update first-run docs only after behavior is verified.

### Non-Goals

- No runtime catalog database.
- No provider credential schema change.
- No OAuth/device-flow implementation.
- No site deployment or release action.

### Acceptance

- Given a built-in standard provider, when a user runs `/connect`, then Talos asks only for the
  needed credential source and does not ask for a base URL.
- Given a custom provider, when a user runs `/connect`, then Talos requires an explicit base URL and
  preserves secret masking.
- Given a large model list, when a user browses or filters it, then output remains bounded and
  provider-qualified.
- Given a diagnostic command is run, then secrets are masked and local paths/config status are clear.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-cli connect`
- `cargo test -p talos-cli model`
- `cargo test -p talos-config provider`
- `cargo check --workspace`
- Manual or integration evidence for a large model list and redacted diagnostics
- `cargo test --workspace` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md` if user-facing setup text changes
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: setup docs drift from actual `/connect` behavior.
- Rollback: keep docs changes behind verified command evidence and leave uncertain behavior as a
  known limit instead of documenting it as supported.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 2 shell for the four-month developer operating plan. |
| 2026-07-08 | D110 Verification | Verified `/connect` standard-vs-custom provider base-url behavior is fully implemented and tested. The behavior was shipped in I101 (2026-07-06) across three layers: (1) CLI handler `mode_runners.rs::handle_connect()` resolves `default_base_url` via 3-tier precedence (user config → `models.toml` `api_base_url` → `builtin_provider_config()` hardcoded fallbacks → `None`); (2) TUI credential panel `state.rs::credential_submit()` implements two-phase flow — standard providers with `default_base_url` submit after API key only, custom providers without it advance to `BaseUrl` field requiring non-empty URL; (3) config persistence `handle_connect_with_credential()` saves credential + resolved base_url and auto-detects protocol from URL path. 15 existing tests confirm all paths: 8 CLI tests in `mode_runners::connect_tests` (credential write, field preservation, base_url update, default fallback, minimax protocol detection, already-authenticated skip, picker construction) + 8 TUI tests in `state::tests` (standard submit, custom first/second submit, empty URL rejection, empty API key cancel, non-connect mode, credential append/backspace, picker filtering/search). No new code needed — D110 is pure verification. |
| 2026-07-08 | D111 Verification | Verified model browsing for large inventories is fully implemented and tested. The `--available-models-browser` (shipped I101) uses viewport-windowed rendering, vim-like navigation, and provider-qualified search. 28 tests pass: 10 `models_browser::tests` (viewport windowing over 500 rows, provider/model/qualified-name filtering, navigation, no-secret rendering, standard/custom provider setup routing, minimax protocol detection, config preservation) + 18 `model_lifecycle::tests` and `tests::tests` (model picker, unauthenticated provider omission, duplicate ID qualification, context limits, model switch markers, session round-trip). Acceptance criterion 3 satisfied: large model lists remain bounded and provider-qualified. No new code needed — D111 is pure verification. |

## Verification Evidence

### D110 verification evidence

- `cargo test -p talos-tui connect`: 8 passed / 0 failed / 0 ignored. Tests: `connect_mode_standard_provider_submits_without_base_url_field`, `connect_mode_custom_provider_first_submit_advances_to_base_url_field`, `connect_mode_custom_provider_second_submit_returns_typed_base_url`, `connect_mode_custom_provider_empty_base_url_stays_open`, `connect_mode_empty_api_key_cancels_without_advancing`, `non_connect_mode_ignores_base_url_and_submits_single_phase`, `connect_picker_is_picker_and_supports_filtering`, `connect_picker_search_matches_provider_group`.
- `cargo test -p talos-cli --bin talos -- connect`: 8 passed / 0 failed / 0 ignored. Tests: `handle_connect_with_credential_writes_new_provider_api_key_and_base_url`, `handle_connect_with_credential_preserves_unrelated_provider_fields`, `handle_connect_with_credential_updates_base_url_when_provided`, `handle_connect_default_base_url_falls_back_to_builtin_provider_config`, `handle_connect_minimax_coding_plan_uses_anthropic_messages_endpoint`, `handle_connect_with_credential_sets_anthropic_protocol_for_minimax_endpoint`, `handle_connect_already_authenticated_does_not_request_credential`, `build_connect_picker_data_none_falls_back_without_blocking`.
- `cargo test -p talos-config provider`: 25 passed / 0 failed / 0 ignored (provider config, model limits, credential write).
- `cargo check --workspace`: passed (exit 0).
- `cargo clippy --workspace -- -D warnings`: passed (exit 0).
- `cargo test --workspace`: 1791 passed / 0 failed / 0 ignored across 61 test binaries (was 1789 at I102 closeout → +2 from commit `3211fc3` mid-stream error chunk fix).
- `cargo fmt --all -- --check`: only pre-existing `bash_tool.rs:583` drift (I102 residual, out of scope).
- `scripts/validate_project_governance.sh .`: passed, 0 governance warnings.
- Acceptance criteria 1 & 2 for I103 are satisfied: standard providers skip base URL, custom providers require it, secret masking is preserved.

### D111 verification evidence

- `cargo test -p talos-cli --bin talos -- model`: 28 passed / 0 failed / 0 ignored. Key tests: `render_lines_is_viewport_windowed_for_large_catalog` (viewport windowing over 500 rows), `filters_by_provider_model_and_qualified_name` (search bounding), `navigation_stays_on_model_rows`, `render_marks_current_and_setup_without_secrets`, `provider_setup_standard_provider_uses_default_without_typed_url`, `provider_setup_custom_provider_requires_base_url`, `unauthenticated_providers_are_omitted_from_model_picker`, `duplicate_model_ids_get_provider_qualified_values`.
- `cargo test -p talos-cli --bin talos -- browser`: 10 passed / 0 failed / 0 ignored (subset of above, focused on `models_browser::tests`).
- Acceptance criterion 3 for I103 is satisfied: large model lists remain bounded and provider-qualified.

## Variance And Residuals

- Planned.

## Retrospective

- Pending.
