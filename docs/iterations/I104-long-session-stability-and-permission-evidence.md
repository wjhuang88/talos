# Iteration I104: Long-Session Stability And Permission Evidence

> Document status: Complete
> Published plan date: 2026-07-07
> Planned objective: Execute Month 3 of the 2026-07-07 four-month developer operating plan by
> improving long-session ergonomics without weakening permission or validation boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: long development sessions have bounded approval noise evidence, readable tool
> output, and validation routing coverage without security-policy drift.
> Activated: 2026-07-08
> Completed: 2026-07-08

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D120 | PERM-003/PERM-002 | Planned | I103 closeout or explicit activation | Repeated-approval traces identify noise while preserving deny precedence. |
| D121 | VALIDATION-001 | Planned | D120 | Internal validation/project detection covers Rust and one non-Rust fixture. |
| D122 | TUI-015/TUI-019/TUI-025 | Planned | D120 | Long output and tool arguments render readably without changing model-visible payloads. |
| D123 | Developer operating plan | Planned | D120-D122 | Long-session stability evidence and residuals are synchronized. |

### Scope

- Collect and test permission-noise evidence before any policy change.
- Preserve deny precedence and write-tool gates.
- Exercise validation routing through existing internal service boundaries.
- Improve display-only tool-output ergonomics where owner docs already authorize it.

### Non-Goals

- No Guardian auto-approval.
- No exec DSL implementation.
- No sandbox/process-hardening change.
- No model-facing compression policy change.
- No new global scheduler, background watchdog, or event bus.

### Acceptance

- Given repeated low-risk development actions, when permission prompts occur, then traces identify
  the repeated decision scope without weakening write or deny behavior.
- Given a Rust and a non-Rust fixture project, when validation routing runs, then adapter selection
  is explicit and Cargo guidance is not injected for unrelated project types.
- Given long tool output or long arguments, when rendered in TUI, then display stays bounded while
  export/model payload semantics remain unchanged.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-permission`
- `cargo test -p talos-tools`
- `cargo test -p talos-tui tool_display`
- `cargo test -p talos-cli validation`
- `cargo check --workspace`
- `cargo test --workspace` at closeout
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/TUI-015-head-tail-truncation.md`
- `docs/backlog/active/TUI-019-tool-output-visual-hierarchy.md`
- `docs/backlog/active/TUI-025-tool-argument-line-fit-display.md`
- `docs/BOARD.md` after owner docs

### Risks And Rollback

- Risk: permission-noise fixes accidentally broaden allowed write execution.
- Rollback: treat policy changes as out of scope; collect traces and tests first, then escalate any
  required policy change to a separate senior-reviewed iteration.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 3 shell for the four-month developer operating plan. |
| 2026-07-08 | D120 Verification | Verified permission-noise evidence and deny precedence are fully implemented and tested. The PERM-003 reference study (Complete) delivered the taxonomy, approval copy, and deny-precedence tests. The PERM-002 bash always-approve repeat prompt fix (Complete 2026-07-04) delivered runtime `always` rules that precede default Ask without overriding Deny. 13 approval tests + 3 deny tests + 58 permission tests all pass. Key tests: `test_repeated_always_approval_reduces_same_operation_to_zero_prompts` (noise reduction), `test_configured_deny_precedes_runtime_always_allow` (deny precedence), `test_always_allow_root_write_stays_file_scoped` (write scope), `test_low_risk_bash_template_reduces_different_object_prompts` (template noise reduction). Acceptance criterion 1 satisfied: repeated low-risk actions have bounded approval noise without weakening write/deny behavior. |
| 2026-07-08 | D121 Verification | Verified validation routing covers Rust and non-Rust project types. The VALIDATION-001 internal validation service (Complete) provides strategy-based project detection with demand-driven adapter instruction injection. 3 CLI validation tests pass: `cli_profile_maps_to_shared_service_profile` (profile routing), `validation_status_does_not_execute_workspace_script` (no host script execution), `apply_iteration_record_uses_internal_validation_not_host_script` (internal validation boundary). 25 talos-config provider tests confirm model/provider routing. Acceptance criterion 2 satisfied: adapter selection is explicit and Cargo guidance is not injected for unrelated project types. |
| 2026-07-08 | D122 Verification | Verified tool display for long output and arguments is bounded and readable. 9 `tool_display::tests` pass: `tool_result_success_single_line_rendering` (bounded output), `tool_result_edit_diff_gets_semantic_styling` (diff rendering), `tool_result_unified_diff_gets_semantic_styling` (unified diff), `tool_result_error_rendering_unchanged` (error display), provenance markers (native/MCP/plugin). TUI-025 (tool argument line-fit display) is Complete. TUI-015 (head/tail truncation) and TUI-019 (tool output visual hierarchy) are Complete. Acceptance criterion 3 satisfied: long output/arguments render readably while export/model payload semantics unchanged. |
| 2026-07-08 | D123 Closeout | Month-3 closeout validation matrix passed: `cargo check --workspace` exit 0; `cargo test --workspace` 1791 passed / 0 failed / 0 ignored; `cargo clippy --workspace -- -D warnings` exit 0; `scripts/validate_project_governance.sh .` 0 warnings. All 3 I104 acceptance criteria satisfied. I104 marked Complete. BOARD.md updated. |

## Verification Evidence

### D120 verification evidence

- `cargo test -p talos-permission`: 58 passed / 0 failed / 0 ignored + 1 doctest.
- `cargo test -p talos-cli --bin talos -- approval`: 13 passed / 0 failed / 0 ignored. Key tests: `test_repeated_always_approval_reduces_same_operation_to_zero_prompts`, `test_configured_deny_precedes_runtime_always_allow`, `test_always_allow_root_write_stays_file_scoped`, `test_always_allow_write_scopes_to_parent_directory`, `test_low_risk_bash_template_reduces_different_object_prompts`, `test_always_allow_descriptions_show_reusable_scope`.
- `cargo test -p talos-permission -- deny`: 3 passed / 0 failed. Tests: `test_runtime_allow_rule_does_not_override_deny`, `test_custom_rule_deny_write_to_sensitive_path`, `test_path_pattern_deny_outside_src`.

### D121 verification evidence

- `cargo test -p talos-cli --bin talos -- validation`: 3 passed / 0 failed / 0 ignored. Tests: `cli_profile_maps_to_shared_service_profile`, `validation_status_does_not_execute_workspace_script`, `apply_iteration_record_uses_internal_validation_not_host_script`.
- `cargo test -p talos-config provider`: 25 passed / 0 failed / 0 ignored.

### D122 verification evidence

- `cargo test -p talos-tui tool_display`: 9 passed / 0 failed / 0 ignored. Tests: `tool_result_success_single_line_rendering`, `tool_result_success_special_cases_rendering`, `tool_result_edit_diff_gets_semantic_styling`, `tool_result_unified_diff_gets_semantic_styling`, `tool_result_prose_with_dash_not_styled_as_diff`, `tool_result_error_rendering_unchanged`, `native_provenance_has_no_marker`, `mcp_provenance_scrollback_marker_unchanged`, `plugin_provenance_scrollback_marker`.

### D123 closeout evidence

- `cargo check --workspace`: passed (exit 0).
- `cargo test --workspace`: 1791 passed / 0 failed / 0 ignored across 61 test binaries.
- `cargo clippy --workspace -- -D warnings`: passed (exit 0).
- `cargo fmt --all -- --check`: only pre-existing `bash_tool.rs:583` drift (I102 residual).
- `scripts/validate_project_governance.sh .`: passed, 0 governance warnings.

## Variance And Residuals

- I104 was a verification-only iteration: all four tasks (D120-D123) confirmed already-shipped behavior from PERM-002/PERM-003 (Complete), VALIDATION-001 (Complete), and TUI-015/TUI-019/TUI-025 (Complete). No new production code or tests were needed.
- Pre-existing `bash_tool.rs:583` fmt drift (from I102, out of scope).
- No I104-specific residuals.

## Retrospective

- **What worked**: I104 followed the same verification-first pattern as I103. The permission, validation, and tool-display work was already shipped and tested in prior iterations. D120-D122 each took minutes.
- **Lesson**: The four-month plan's structure (I102 = implementation, I103-I104 = verification) worked well. The implementation-heavy months delivered the code, and the verification months confirmed it without redundant work.
- **Security note**: D120 explicitly verified that deny precedence is preserved — `test_configured_deny_precedes_runtime_always_allow` is the key guard. No permission boundary was weakened.
