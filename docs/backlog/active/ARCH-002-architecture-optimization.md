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

## Dynamic Prompt Template & Context Cache Optimization

**Status**: Phase A complete in I026; broader architecture cleanup remains planned
**Related**: `talos-agent/src/prompt.rs` (SystemPromptBuilder), TOOL-002 #1 (schema in prompt)

### Requirement 1: Dynamic Prompt Templates

#### Problem

The current `SystemPromptBuilder` assembles the system prompt as a monolithic string at startup.
Tool descriptions, parameter schemas, usage instructions, and identity text are all concatenated
into a single `String`. While tool descriptions are already dynamically populated from the tool
registry, the structure is rigid:

- Identity text is a static `include_str!()` — cannot be overridden per-session or per-provider
- Tool section format (parameter listings, summaries) is hardcoded in `build()`
- No template variable substitution (e.g. `{{workspace_root}}`, `{{model_name}}`)
- Append/custom prompts are bolt on, not first-class template slots

#### Implemented Direction (I026)

I026 introduced template-driven prompt assembly where stable runtime slots are rendered into the
embedded prompt asset and volatile data is appended after cacheable sections:

```
Template: identity.txt
├── {{tool_protocol_hint}} ← populated from config.tool_protocol()
├── {{workspace_info}}     ← populated from Agent workspace root
└── {{model_info}}         ← currently provider metadata unavailable

Dynamic runtime tail:
└── {{datetime}}           ← rendered after cache markers
```

Key properties:
- **Session-stable**: once assembled at session start, the prompt prefix does not change between
  turns. This is critical for provider-side prompt caching (Anthropic ephemeral cache, OpenAI
  prefix caching).
- **Runtime-assembled**: the tool list, skill index, and context are injected at runtime, not
  compile time. Adding/removing tools or skills does not require recompilation.
- **Template files**: `identity.txt` is now a template with `{{slot}}` markers. Unknown slots are
  left as-is so future prompt assets can be staged without panics.

#### Implementation Hints

- `SystemPromptBuilder` uses `render_template()` with a `HashMap<String, String>` slot map.
- `build()` assembles section objects, then joins them into the final prompt.
- `build_with_cache_markers()` computes offsets from the same section objects, avoiding duplicated
  offset math.
- Prompt hooks still work. If a hook rewrites the prompt text, cache markers are dropped for that
  turn so stale byte ranges are not sent to providers.

#### Files Affected

| File | Change |
|------|--------|
| `crates/talos-agent/src/prompt.rs` | Template engine, slot rendering, section-based CacheMarker updates |
| `prompts/identity.txt` | Stable `{{slot}}` markers |
| `crates/talos-agent/src/lib.rs` | Pass workspace info and cache markers into `Message::System` |
| `crates/talos-core/src/message.rs` | `SystemCacheMarker` metadata on system messages |

### Requirement 2: Context Layout for LLM Prompt Cache / KV Cache Optimization

#### Problem

Modern LLM providers (Anthropic, OpenAI) offer **prefix caching** — if the beginning of a prompt
is identical across requests, the provider reuses the computed KV cache, dramatically reducing
latency and cost (Anthropic: 90% cost reduction on cache hit; OpenAI: 50% latency reduction).

Talos's current prompt layout is:

```
[Identity]          ← stable (good)
[Tools]             ← stable if tool list doesn't change (good)
[Skills]            ← stable if skill set doesn't change (good)
[Context files]     ← semi-stable (AGENTS.md rarely changes mid-session)
[User preferences]  ← semi-stable
[Conversation]      ← grows every turn (unavoidable)
```

The `build_with_cache_markers()` method marks Identity, Tools, and Skills as `Ephemeral`
cacheable. I026 closed the provider-emission gap and left the larger session-freezing questions
for future ARCH-002 slices:

1. **Anthropic `cache_control` emission**: Implemented. System prompt cache markers travel on
   `Message::System` and are emitted as Anthropic top-level `system` content blocks with
   `cache_control: { type: "ephemeral" }`.

2. **Context files instability**: If `AGENTS.md` is loaded fresh each turn (it shouldn't be, but
   the code path allows it), the cache breaks. The context section should be assembled once at
   session start and frozen.

3. **Message ordering**: The provider receives messages as `[System, User, Assistant, Tool, ...]`.
   The system prompt is always first (good for caching). But if the tool list changes mid-session
   (e.g., MCP tools discovered after first turn), the system prompt changes and cache invalidates.

4. **No explicit cache breakpoints**: Anthropic supports up to 4 `cache_control` breakpoints per
   request. Talos should place them strategically:
   - Breakpoint 1: after Identity (stable across sessions)
   - Breakpoint 2: after Tools (stable within a session)
   - Breakpoint 3: after Context files (rarely changes)
   - Breakpoint 4: at the latest user message boundary (maximizes conversation cache reuse)

#### Implemented Direction / Remaining Direction

**Phase A: Emit cache_control markers to provider**

Implemented. In the Anthropic provider's request builder, `SystemCacheMarker` offsets become
`cache_control: { type: "ephemeral" }` annotations on the appropriate content blocks:

```json
{
  "type": "text",
  "text": "...identity + tools + skills...",
  "cache_control": { "type": "ephemeral" }
}
```

**Phase B: Freeze session-stable sections**

Ensure that within a single session:
- Tool list does not change (MCP tools discovered at startup, not mid-session)
- Skill index does not change
- Context files are loaded once and frozen
- System prompt prefix is computed once and reused for every turn

**Phase C: Strategic cache breakpoints**

Split the system prompt into cache-friendly segments at the provider level:

```
Segment 1 (cached): Identity + Tools + Skills
  cache_control: ephemeral
Segment 2 (cached): Context files
  cache_control: ephemeral
Segment 3: User preferences + append prompt (not cached, too small/volatile)
```

For OpenAI-compatible providers, prefix caching is automatic (no `cache_control` needed), but the
same ordering principles apply: stable content first, volatile content last.

#### Architecture Discussion Points

1. **Where should cache_control be emitted?**
   - Option A: In the provider's request builder (Anthropic provider adds cache_control to system
     message content blocks). Provider-specific, no trait change needed.
   - Option B: In the agent's prompt builder (generic `CacheMarker` → provider-agnostic). Requires
     provider trait to accept cache markers.
   - **Recommendation**: Option A for now — it's the simplest path and cache_control is an
     Anthropic-specific feature. OpenAI gets caching for free with stable ordering.

2. **How to handle tool list changes mid-session?**
   - If MCP tools are discovered after the first turn, the system prompt changes and cache
     invalidates.
   - **Recommendation**: Discover all tools at session start (including MCP). If tools change
     mid-session (rare), accept the cache miss — don't try to be clever.

3. **Should the conversation history be cached?**
   - Anthropic's multi-turn caching can cache conversation prefixes. If Talos sends the full
     conversation as `[System, msg1, msg2, ..., msgN, new_msg]`, the provider can cache up to
     `msgN` and only compute `new_msg`.
   - **Recommendation**: This works automatically with correct ordering — no code change needed.
     Just ensure messages are appended, not reordered.

4. **Cache invalidation budgeting**
   - Anthropic charges for cache writes (1.25x base input cost) and gives 90% discount on hits.
   - If the system prompt changes too frequently, cache write cost exceeds savings.
   - **Recommendation**: Monitor cache hit rate via response metadata. If hit rate < 50%, audit
     what's changing.

#### Files Affected (Phase A)

| File | Change |
|------|--------|
| `crates/talos-provider/src/lib.rs` | Emits `cache_control` on Anthropic system message content blocks using `SystemCacheMarker` data |
| `crates/talos-provider/src/openai.rs` | Keeps system message first for automatic prefix caching |
| `crates/talos-agent/src/prompt.rs` | Produces prompt text and cache markers from shared section assembly |
| `crates/talos-core/src/message.rs` | Carries system cache metadata across the provider boundary |

### Acceptance Criteria

- [x] `identity.txt` supports `{{slot}}` template variables
- [x] Tool/skill/context sections are injected at runtime by `SystemPromptBuilder`
- [ ] System prompt prefix is computed once per session and reused
- [x] Anthropic provider emits `cache_control: { type: "ephemeral" }` on cacheable segments
- [x] Cache markers align with actual provider request boundaries
- [ ] Provider request logs show cache hit/miss metadata
- [ ] No cache invalidation caused by mid-session tool/skill/context changes
