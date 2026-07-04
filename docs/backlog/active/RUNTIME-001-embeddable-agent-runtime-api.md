# RUNTIME-001: Embeddable Agent Runtime API

**Status**: Complete (pre-1.0 facade)
**Priority**: P2
**Created**: 2026-06-28
**Source**: User request to reuse Talos runtime capabilities from other projects
**Depends on**: `talos-agent` turn-loop stability; `talos-core` protocol stability; permission
pipeline stability; session protocol cleanup; ADR-024

I093 activation note (2026-07-04): selected for self-bootstrap readiness audit only. The current
status remains "Complete (pre-1.0 facade)", not a stable 1.0 SDK guarantee.

I093 A13 readiness result (2026-07-04): `docs/reference/REL-002-READINESS-REPORT-2026-07-04.md`
keeps RUNTIME-001 at pre-1.0. Minimum gaps before REL-002 qualification are Talos-primary edit
loop evidence, autonomous validation evidence capture, git/issue sync policy, and stable SDK
surface classification.

I095 activation note (2026-07-04): selected for runtime validation evidence only. This does not
change the "Complete (pre-1.0 facade)" status and does not authorize arbitrary shell policy
expansion, scheduled execution, Guardian auto-approval, exec DSL, hidden pass/fail, release claim,
tag, publish, or permission-default changes.

## Problem

Talos's core turn loop is mostly isolated in `talos-agent`, and CLI/TUI crates do not own the
runtime. That is a good internal boundary, but it is not yet a clean dependency boundary for other
Rust projects that want to bring their own product features, UI, session surface, or interaction
model while reusing Talos's agent runtime.

Without an explicit embeddable runtime API, external projects must assemble low-level pieces
directly: provider trait objects, tool registries, permission engines, sandbox providers, prompt
configuration, session actors, and event channels. That makes the API easy to misuse, couples
consumers to Talos product internals, and weakens the runtime's long-term semver story.

## Scope

- Define a stable runtime-facing API for embedding Talos in another Rust project.
- Provide a conservative `RuntimeBuilder` / `RuntimeHandle` style facade around the existing
  `talos-agent` and session actor seams.
- Keep external projects free to implement their own CLI, TUI, web UI, commands, persistence,
  product features, and interaction model.
- Preserve the existing permission and sandbox pipeline as the safe default for write, execute,
  and network-capable tools.
- Make runtime events and commands serializable and suitable for local RPC or in-process consumers.
- Move product/debug affordances that are currently inside the core turn loop, such as
  `/mock-request`, behind explicit APIs or higher-level caller-owned command handling.
- Replace CLI-shaped runtime configuration such as `print_mode` with product-neutral runtime
  policy fields.
- Document which crates and types are public SDK surface and which remain Talos product internals.

## Non-Goals

- No new UI or command surface under this story.
- No remote transport implementation; remote session protocol remains `REMOTE-001`.
- No plugin runtime implementation; WASM plugin architecture remains `PLUGIN-001`.
- No rewrite of the turn loop unless a narrow compatibility issue requires it.
- No weakening of permission, sandbox, provenance, hook, or secret-display boundaries for embedder
  convenience.
- No public semver promise for every existing `talos-agent` internal type.

## Candidate Slices

1. **API audit and boundary ADR**
   - Classify current `talos-core`, `talos-agent`, `talos-provider`, `talos-tools`,
     `talos-permission`, and `talos-sandbox` public types as SDK surface, internal surface, or
     transitional surface.
   - Decide whether the facade lives in a new `talos-runtime` crate or a `talos-agent::runtime`
     module.
   - **Done 2026-06-28**: ADR-024 selects a dedicated `talos-runtime` facade crate, keeps
     `talos-agent` as the turn-loop implementation crate, keeps foundational protocol/trait types
     in `talos-core`, and requires protocol cleanup before SDK stability.

2. **Runtime facade**
   - Add a builder that accepts provider, tools, workspace root, permission policy, sandbox policy,
     memory provider, skill index, and prompt customization without exposing every internal
     construction detail.
   - Expose a handle that can submit turns, interrupt turns, receive typed events, and shut down.
   - **Done 2026-06-28, first slice only**: `talos-runtime` now provides a minimal
     `RuntimeBuilder` / `RuntimeHandle` facade with safe-by-default tool wrapping,
     caller-supplied permission rules, provider/tool injection, typed session events, and
     mock-provider embedder tests. This is not yet the stable 1.0 SDK surface.

3. **Protocol cleanup**
   - Fix nested event serialization so runtime events can round-trip cleanly across RPC boundaries.
   - Replace `SessionConfig::print_mode` with product-neutral runtime policy.
   - Convert `/mock-request` from a magic user-message command into an explicit diagnostic API or
     caller-owned command.
   - **Done 2026-06-28**: `SessionEvent::AgentEvent` now serializes as a nested `event` payload,
     `SessionConfig` uses product-neutral `RuntimePolicy`, and request preview runs through
     explicit `SessionOp::PreviewRequest` / `RuntimeHandle::preview_request()` instead of
     `talos-agent` parsing `/mock-request`.

4. **Embedding proof**
   - Add a minimal non-CLI integration test or example showing another Rust crate using the runtime
     facade with a mock provider and custom tool.
   - **Done 2026-06-28**: `talos-runtime` tests cover response streaming, safe default write
     denial, explicit write allow-listing, initial history, and explicit request preview.

## Acceptance Criteria

- [x] A decision record or proposal defines the embeddable runtime boundary and public SDK surface.
- [x] External consumers can build a safe runtime without depending on `talos-cli` or `talos-tui`.
- [x] Runtime construction has a single documented happy path that defaults to permission-aware
      execution.
- [x] A consumer can submit a turn, stream events, interrupt a turn, and shut down without using
      Talos product UI code.
- [x] Runtime events and commands have tested serialization round-trips suitable for RPC or
      cross-crate integration.
- [x] CLI/TUI/debug-only behavior is not triggered by hidden magic strings inside the core runtime.
- [x] Existing Talos CLI/TUI behavior remains compatible through adapter code.
- [x] User-facing and developer-facing docs explain how to embed the runtime and which APIs are
      semver-supported.

## Validation

- `cargo test -p talos-core -p talos-agent`
- Targeted serialization tests for runtime commands/events.
- A minimal embedder example or integration test using a mock provider and custom tool.
- Workspace `cargo test --workspace` before closing an implementation iteration.
- Governance validation if backlog, architecture, or SDK documentation changes.

## Execution Notes

- 2026-06-28: Added the `talos-runtime` crate to the workspace with `RuntimeBuilder`,
  `RuntimeHandle`, permission-aware tool wrapping, and `collect_until_turn_completed()`.
- 2026-06-28: Verified embedder behavior with mock-provider tests:
  `runtime_streams_mock_response`, `runtime_denies_ask_tools_by_default`,
  `runtime_allows_tool_when_rule_allows_write`, and `runtime_accepts_initial_history`.
- 2026-06-28: The facade slice initially left protocol cleanup and SDK hardening open:
  command/event serialization coverage, product-neutral session policy, `/mock-request`
  diagnostic extraction, semver surface documentation, and a fuller embedder example.
- 2026-06-28: Completed protocol cleanup for the pre-1.0 facade: `SessionOp::PreviewRequest`,
  `RuntimePolicy`, round-trippable `SessionEvent::AgentEvent { event }`, and CLI-owned
  `/mock-request` parsing.
- 2026-06-28: README and README.zh-CN now document the embedding boundary: `talos-runtime` plus
  re-exported `talos-core` protocol/trait types are the public pre-1.0 surface; lower-level
  `talos-agent` constructors remain implementation surface unless documented otherwise.
- 2026-06-30: GitHub issue #4 closed the interactive approval gap for embedders:
  `talos-runtime` now exposes `ApprovalHandler` and `RuntimeBuilder::approval_handler(...)`.
  Missing handlers still deny `Ask` decisions, `ApproveOnce` executes one call, and
  `AlwaysApprove` installs in-memory allow rules for the current runtime without writing user
  configuration.
- 2026-06-30: GitHub issue #5 added runtime prompt customization for embedders:
  `RuntimeBuilder::custom_prompt(...)` replaces the default Talos identity and
  `RuntimeBuilder::append_prompt(...)` appends product-specific instructions before the session
  actor starts. Runtime request-preview coverage proves `initial_history` is no longer needed as an
  ineffective prompt workaround.

## Required Reads

- `docs/reference/ARCHITECTURE.md`
- `docs/backlog/active/ARCH-030-remaining-production-root-residual-register.md`
- `docs/backlog/active/AGENT-001-standard-agent-protocol-support.md`
- `docs/backlog/active/AGENT-002-dotagents-protocol-support.md`
- `docs/backlog/active/REMOTE-001-remote-session-protocol.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/decisions/005-app-server-session-boundary.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/decisions/009-tool-provenance.md`
- `docs/decisions/021-tool-call-protocol-architecture.md`
- `docs/decisions/024-embeddable-runtime-api-boundary.md`
- `crates/talos-core/src/`
- `crates/talos-agent/src/`
- `crates/talos-provider/src/`
- `crates/talos-permission/src/`
- `crates/talos-sandbox/src/`

## Open Questions

1. Should the public facade be a new `talos-runtime` crate or a module inside `talos-agent`?
2. Should the first stable API be in-process Rust only, or should it intentionally mirror the
   future `REMOTE-001` command/event protocol?
3. Which existing `talos-agent` configuration mutators should remain public after the facade lands?
4. How much of Skill, memory, and hook configuration should be first-class in v1 versus adapter-only?
