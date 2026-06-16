# ARCH-002: Architecture Optimization and Anti-Corruption Treatment

**Status**: Planned (needs analysis)  
**Priority**: P3  
**Depends on**: None (can proceed independently)  
**Related ADRs**: ADR-005 (L2 seam), ADR-009 (tool provenance), ADR-013 (boundary control)

## Problem

Talos has grown organically across 16 crates and 12+ iterations. Several boundary violations, type leakages, and coupling issues have accumulated. This story addresses architecture hygiene before the codebase grows further (TOOL-003 adds 7 new tools, each touching multiple crates).

## Scope

### In Scope

- Audit all crate boundaries for coupling violations
- Identify type leakage points where external library types bleed into internal APIs
- Identify missing anti-corruption layers (ACL)
- Propose refactoring slices with prioritization
- Document the target architecture state

### Out of Scope

- Actual code refactoring (separate iteration)
- New features
- Performance optimization

## Known Issues (from architecture survey)

### P0 — Blocks Clean Architecture

#### 1. `talos-mcp` depends on `talos-agent` (dead dependency)
- **File**: `crates/talos-mcp/Cargo.toml`
- **Issue**: MCP declares dependency on agent crate but source has zero `use talos_agent` — dead coupling
- **Fix**: Remove dependency

#### 2. `talos-rpc` coupled to concrete `Agent` (no trait abstraction)
- **Files**: `talos-rpc/src/methods/mod.rs:27`, `talos-rpc/src/server.rs:24`
- **Issue**: `MethodContext { agent: Arc<Agent> }` — concrete struct, not trait
- **Fix**: Define `AgentRuntime` trait in `talos-core`; RPC depends on trait

#### 3. Duplicate `ToolResult` types in `talos-core`
- **Files**: `talos-core/src/tool.rs:45` (no tool_use_id) vs `talos-core/src/message.rs:20` (has tool_use_id)
- **Issue**: Same name, different semantics, causes confusion
- **Fix**: Rename `message::ToolResult` → `MessageToolResult`

### P1 — Type Leakage

#### 4. `rmcp` types bleed through `talos-mcp` public API
- **Files**: `error.rs:50`, `client/adapter.rs:21`, `client/dispatcher.rs:29`, `client/facade.rs:12`, `server/handler.rs`, `server/permission.rs`
- **Issue**: `rmcp::ErrorData`, `rmcp::model::Tool` in public signatures
- **Fix**: Define wrapper types (`McpToolDescriptor`, `McpTransportError`)

#### 5. `rusqlite::Error` leaks in `talos-evolution` and `talos-session`
- **Files**: `talos-evolution/src/lib.rs:29`, `talos-session/src/sqlite.rs:26`
- **Fix**: Define crate-specific store error enums

#### 6. Duplicate `ToolDefinition` in `talos-agent/caching.rs`
- **File**: `talos-agent/src/caching.rs:45` (identical to `talos-core/src/provider.rs:31`)
- **Fix**: Use `talos_core::provider::ToolDefinition`

### P2 — God Module Decomposition

24 files exceed 500 lines. Top candidates:

| File | Lines | Proposed Split |
|------|-------|---------------|
| `talos-agent/src/lib.rs` | 2,568 | `turn_loop.rs`, `tool_execution.rs`, extract ~1000 lines test code |
| `talos-tui/src/app.rs` | 2,294 | `tool_display.rs`, `scrollback.rs`, `event_loop.rs` |
| `talos-cli/src/main.rs` | 2,002 | `registry.rs`, `provider_setup.rs`, `event_bridge.rs`, extract test fixtures |
| `talos-session/src/lib.rs` | 1,736 | Extract ~800 lines test code to `tests/` |
| `talos-skill/src/lib.rs` | 1,483 | `parser.rs`, `manager.rs`, `loader.rs`, extract tests |
| `talos-tools/src/lib.rs` | 863 | `bash.rs`, `read.rs`, `write.rs`, `edit.rs` (one file per tool) |

### P3 — Config Boundary

#### 7. `talos-mcp` imports `talos_config` types directly
- **File**: `talos-mcp/src/client/manager.rs:5`
- **Fix**: Define `McpClientConfig` in talos-mcp; convert at CLI composition root

### Positive Findings (what's done right)

- `talos-core` has zero internal dependencies — clean foundation
- `talos-conversation` / `talos-tui` separation is clean
- `AgentTool`, `LanguageModel`, `SandboxProvider` traits properly abstracted
- Error types use `thiserror` consistently
- `talos-mcp/src/client/facade.rs` shows intentional ACL awareness (incomplete but right direction)

## Open Questions

1. **Provider abstraction**: Should the `LanguageModel` trait return `Pin<Box<dyn Stream>>` instead of `mpsc::Receiver`? More idiomatic async Rust, but breaking change.

2. **Tool result types**: Should `ToolResult` evolve from `String` content to an enum (`Text(String) | Image(ImageData) | Json(Value)`)? Or keep String + optional images field?

3. **Permission engine trait**: Should `PermissionEngine` become a trait? Currently concrete, but MCP server and TUI mode need different behavior.

4. **Event protocol**: Should `AgentEvent` be split into provider-level events and agent-level events? Currently one enum serves both.

5. **Config isolation**: Should each crate define its own config struct, with `talos-config` providing a central loader that maps to per-crate configs?

6. **talos-cli decomposition**: Should `main.rs` be split into modules (registry, provider setup, event bridge), or extracted into a new `talos-app` crate?

## Proposed Approach

### Phase 1: Documentation Validation (this story, prerequisite)

Before any refactoring, validate that all project documentation accurately reflects the current codebase:

- **ADR consistency**: Each ADR's stated decisions match actual implementation
- **ARCHITECTURE.md accuracy**: Crate list, dependency graph, data flow diagrams match reality
- **AGENTS.md rules**: Hard constraints, coding behavior, task router entries are current
- **Backlog accuracy**: Story statuses, dependencies, acceptance criteria reflect actual state
- **Iteration notes**: Completed iterations have accurate evidence and residual work recorded
- **EVOLUTION.md**: Lessons learned are current and traceable to real incidents
- **SOP accuracy**: Procedures match actual workflow
- **README**: Features, usage examples, tech stack reflect current capabilities
- **BOARD.md**: Derived view matches owner docs

Output: Validation report with discrepancies found and fixes applied.

### Phase 2: Architecture Audit (this story)

Run comprehensive coupling analysis:
- Crate dependency graph validation
- Public API surface audit per crate
- Type leakage inventory
- File size / responsibility inventory
- Trait vs concrete type usage audit

Output: Prioritized refactoring backlog with specific slices.

### Phase 3: Anti-Corruption Layers (separate iteration)

Implement ACLs at key boundaries:
- Provider → Agent event translation
- Config → per-crate config DTOs
- Tool result structured types

### Phase 4: God Module Decomposition (separate iteration)

Split the three large files into focused modules.

### Phase 5: Trait Abstractions (separate iteration)

Introduce traits where concrete types create tight coupling.

## Acceptance Criteria

### Phase 1: Documentation Validation
- [ ] All ADRs validated against actual implementation
- [ ] ARCHITECTURE.md crate list, dependency graph, data flow verified
- [ ] AGENTS.md hard constraints and task router current
- [ ] Backlog story statuses and dependencies accurate
- [ ] Iteration evidence and residual work recorded
- [ ] EVOLUTION.md lessons current and traceable
- [ ] SOP procedures match actual workflow
- [ ] README features and usage examples current
- [ ] BOARD.md derived view matches owner docs
- [ ] Discrepancy report produced, fixes applied

### Phase 2: Architecture Audit
- [ ] Comprehensive coupling audit document produced
- [ ] Each identified issue has: severity, affected crates, proposed fix, estimated effort
- [ ] Dependency graph validated (no circular or unexpected dependencies)
- [ ] Type leakage inventory complete
- [ ] God module decomposition plan with proposed file splits
- [ ] Prioritized refactoring backlog created (separate stories for each slice)
- [ ] Target architecture state documented

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Refactoring breaks working features | Medium | High | Each slice ships independently with full test coverage |
| Over-engineering boundaries | Medium | Medium | Only add ACLs where coupling causes real problems |
| Breaking public API changes | Low | High | Semver-bound crates need migration plans per ADR |
