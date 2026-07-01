# SESSION-003: Model Switch Context Marker

| Field | Value |
|-------|-------|
| Story ID | SESSION-003 |
| Priority | P2 |
| Status | Planned |
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

- [ ] Successful model switches add a system/context marker.
- [ ] Marker contains the new model and provider.
- [ ] Marker is persisted in session JSONL.
- [ ] Subsequent API request context includes the marker.
- [ ] Tests cover switch, persistence, and request-preview visibility.

## Required Reads

- [GitHub Issue #10](https://github.com/wjhuang88/talos/issues/10)
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-session/src/`
