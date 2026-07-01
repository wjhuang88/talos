# SESSION-003: Model Switch Context Marker

| Field | Value |
|-------|-------|
| Story ID | SESSION-003 |
| Priority | P2 |
| Status | Complete |
| Source | [GitHub Issue #10](https://github.com/wjhuang88/talos/issues/10) |
| Relates To | SESSION-001, CMD-001 |

## Requirement

When `/model` switches the active model, persist a system/context marker so subsequent model
requests can see the switch boundary and current model identity.

## Scope

- Add a model-switch marker after successful model rebuild.
- Include new provider/model and, when available, previous provider/model.
- Persist the marker into session history.
- Ensure later request previews include the marker.

## Non-Goals

- No change to provider selection policy.
- No replay or rewriting of older messages.

## Acceptance Criteria

- [x] Successful model switches add a system/context marker.
- [x] Marker contains the new model and provider.
- [x] Marker is persisted in session JSONL.
- [x] Subsequent API request context includes the marker.
- [x] Tests cover switch, persistence, and request-preview visibility.

## Execution Notes

- 2026-07-01: Implemented in I076/T106. Successful model rebuilds now append a system marker containing previous and new provider/model identity, inject it into the rebuilt agent history, and persist it to session JSONL after commit.
- 2026-07-01: Moved to Complete during I076/T109 closeout after full workspace validation passed.

## Verification Evidence

- 2026-07-01: `cargo test -p talos-cli model_switch_marker` passed: 3 tests.
- 2026-07-01: `cargo test -p talos-cli` passed: 95 unit tests and 8 integration tests.
- 2026-07-01: `cargo check --workspace` passed.
- 2026-07-01: `cargo clippy -p talos-cli -- -D warnings` passed.
- 2026-07-01: `cargo test --workspace` passed during I076/T109 closeout.
- 2026-07-01: `scripts/validate_project_governance.sh .` passed with 0 warnings during I076/T109 closeout.

## Required Reads

- [GitHub Issue #10](https://github.com/wjhuang88/talos/issues/10)
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-session/src/`
