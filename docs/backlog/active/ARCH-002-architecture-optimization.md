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

## Known Issues (Preliminary)

### 1. Type Leakage

External crate types appearing in internal public APIs:

| External Type | Where It Leaks | Impact |
|---------------|---------------|--------|
| `serde_json::Value` | Tool parameters, tool results, provider messages | Every crate depends on serde_json; internal types should wrap or constrain usage |
| `reqwest::Response` | Provider implementations | HTTP client details leak through provider trait boundaries |
| Provider-specific SSE types | Agent receives `AgentEvent` which originated as provider SSE parsing | Provider parsing logic mixed with event semantics |

### 2. Crate Boundary Concerns

- `talos-cli/src/main.rs` is ~2000 lines — acts as God module (registry builders, approval handlers, event loops, provider construction, session management all inline)
- `talos-tui/src/app.rs` is ~2300 lines — owns TUI state, event loop, stream rendering, tool call display, scrollback, approval, debug — too many responsibilities
- `talos-agent/src/lib.rs` is ~2500 lines — owns turn loop, hooks, permission, tool execution, text filtering, doom detection, session actor

### 3. Missing Abstractions

- `ToolCallDisplay` and `ToolResultDisplay` are defined in `talos-conversation` but the TUI reaches into their fields directly — no rendering trait
- Provider trait returns `mpsc::Receiver<AgentEvent>` — ties the agent to a specific channel implementation
- `PermissionEngine` is passed as `Arc<PermissionEngine>` concrete type, not `dyn PermissionEngineTrait`
- `AppServerSession` mixes session state management with turn forwarding logic

### 4. Anti-Corruption Opportunities

| Boundary | Current State | ACL Needed? |
|----------|--------------|-------------|
| Provider → Agent | AgentEvent directly carries provider-specific fields | Yes — translate provider events to internal domain events |
| Agent → Conversation Engine | AgentEvent passed through as-is | Mostly clean, but text filter logic split across agent and provider |
| Conversation Engine → TUI | UiOutput enum is clean | Clean — this is the best boundary |
| Config → Provider | Config types passed directly to provider constructors | Yes — provider should receive a provider-specific config DTO |
| Tools → Agent | ToolResult is a plain struct with String content | Needs ImageData support (TOOL-003) — opportunity to add structured result types |

### 5. God Module Decomposition Candidates

| File | Lines | Proposed Split |
|------|-------|---------------|
| `talos-cli/src/main.rs` | ~2000 | Extract: `registry.rs`, `approval.rs` (exists), `provider_setup.rs`, `event_bridge.rs` |
| `talos-tui/src/app.rs` | ~2300 | Extract: `tool_display.rs`, `scrollback.rs`, `event_loop.rs` |
| `talos-agent/src/lib.rs` | ~2500 | Extract: `turn_loop.rs`, `tool_execution.rs`, `text_filter.rs` |

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
