# talos-runtime SDK Support Contract

Created: 2026-06-30 (T13 of the four-month self-bootstrap plan)

This document defines the support boundary for embedding Talos as a Rust runtime. It is a
pre-1.0 contract: the surface is usable but not yet semver-stable. REL-002 gates the 1.0 promise.

## Supported Embedding Surface

Embedders should depend on **`talos-runtime`** and the types it re-exports from `talos-core`.
These are the only types covered by this contract.

### Builder and Handle

| Type | Role | Stability |
|---|---|---|
| `RuntimeBuilder` | Configure and construct an embedded runtime | Pre-1.0 stable shape; method set may grow |
| `RuntimeHandle` | Interact with a running runtime (submit, events, shutdown) | Pre-1.0 stable shape; method set may grow |
| `collect_until_turn_completed` | Helper to drain events until a turn finishes | Pre-1.0 |
| `RuntimeError` / `RuntimeResult<T>` | Error types for runtime operations | Pre-1.0 |

### Re-exported Protocol Types (from `talos-core`)

| Type | Purpose |
|---|---|
| `AgentEvent` | Streaming events during a turn (text delta, tool call, tool result, turn end) |
| `ToolCall` | A tool call request from the model |
| `MessageToolResult` | A tool execution result |
| `StopReason` | Why the model stopped generating |
| `Usage` | Token usage statistics |
| `TurnCompletionStatus` | Turn outcome: `Success`, `Cancelled`, or `Error` |
| `ToolNature` | Risk classification: Read / Write / Execute / Network |
| `ToolProvenance` | Tool origin: `Native`, `McpRemote { server }`, or `Plugin { name, version, carrier }` (ADR-028) |
| `ApprovalChoice` | User decision: `ApproveOnce`, `AlwaysApprove`, `Deny` |
| `Message` | Conversation message types |
| `ToolDefinition` | Provider-facing tool schema |

### Traits Embedders Implement

| Trait | Crate | Purpose |
|---|---|---|
| `LanguageModel` | `talos-core` | Provider adapter for LLM streaming |
| `AgentTool` | `talos-core` | Custom tool definition |
| `ApprovalHandler` | `talos-runtime` | Permission-gated tool approval |
| `PermissionRule` | `talos-permission` | Allow/deny/ask rules |
| `SandboxProvider` | `talos-sandbox` | Optional sandbox for isolation |

### Re-export boundary

Only the types re-exported **by `talos-runtime` itself** (plus the `talos-core` protocol types it
re-exports) are covered by this SDK contract. The extension traits above are listed for orientation:
`LanguageModel` and `AgentTool` live in `talos-core` and ARE reached through the facade; but
`PermissionRule` and `SandboxProvider` require depending on `talos-permission` / `talos-sandbox`
directly when an embedder implements them. Those lower-level published crates have their own
independent pre-1.0 public APIs; depending on them directly is supported by those crates' own
support boundaries, not by this runtime SDK contract.

## Implementation Surface (NOT Supported)

The following are internal implementation details. Embedders should NOT depend on them directly:

| Crate / Type | Why Not Supported |
|---|---|
| `talos-agent` constructors | The turn-loop implementation crate; its API may change without notice. Use `RuntimeBuilder` instead. |
| `talos-session` internals | Session storage internals (TLOG durable format, archival, SQLite index) are not a public embedding API. This excludes only session INTERNALS — the published `talos-session` crate retains its own independent pre-1.0 public API. |
| `AppServerSession` | The actor that drives the conversation loop; managed by `RuntimeHandle`. |
| `talos-tui` | Product UI; not a reusable library. |
| `talos-cli` library types | Binary package; library API is explicitly unsupported (binary-only per T06). |
| `talos-evolution` | Product-specific learning; not externally reusable yet. |

## Direct-Use Caveats for `talos-agent`

If an embedder has a compelling reason to use `talos-agent` directly (bypassing `talos-runtime`):

1. **No stability promise.** The `talos-agent` API changes as the turn loop evolves. Pin an exact
   version and expect breaking changes between minor versions.
2. **No SDK documentation.** `talos-agent` docs describe implementation, not a supported contract.
3. **Migration path.** If a `talos-agent` pattern becomes popular, it will be promoted into
   `talos-runtime` with a proper API. File an issue before depending on an internal constructor.
4. **Permission boundary.** Direct `talos-agent` use bypasses the `RuntimeBuilder` permission
   wrapping. The embedder is responsible for installing permission rules and approval handlers.
5. **Publication gate.** `talos-agent` is a gate-before-publish crate (see
   [CRATE-PUBLICATION-MATRIX](CRATE-PUBLICATION-MATRIX.md) row 13). It is not on crates.io and may
   not be published until sandbox/tools dependency gates clear. Per
   [ADR-052](../decisions/052-sdk-publication-and-composition-boundary.md) it will be published as
   an **implementation dependency only** (route A: `talos-sandbox` → `talos-tools` → `talos-agent`
   → `talos-runtime`), never promoted to a second supported SDK entrypoint.

## Embedding Patterns

### Pattern 1: Minimal Turn Loop

```rust,ignore
use talos_runtime::RuntimeBuilder;
// provider: Arc<dyn LanguageModel>

let mut handle = RuntimeBuilder::new()
    .provider(provider)
    .workspace_root(".")
    .build()?;

handle.submit("Hello, what can you do?")?;
while let Some(event) = handle.next_event().await {
    // inspect event
}
handle.shutdown()?;
```

### Pattern 2: Custom Tool + Approval

```rust,ignore
let mut handle = RuntimeBuilder::new()
    .provider(provider)
    .tool(Arc::new(MyTool {}))
    .approval_handler(Arc::new(MyApprovalHandler {}))
    .build()?;
```

Without an approval handler, `Ask` decisions are **denied** by default. Always provide an
`ApprovalHandler` for headless embedding unless all registered tools are read-only.

For the Talos snapshot-aware file-tool set, construct one shared registry-backed group and register
all four tools so writes and deletes invalidate read snapshots consistently:

```rust,ignore
let (read, write, edit, delete) =
    talos_tools::snapshot_aware_file_tools(workspace_root.clone());
let mut handle = RuntimeBuilder::new()
    .provider(provider)
    .workspace_root(workspace_root)
    .tool(Arc::new(read))
    .tool(Arc::new(write))
    .tool(Arc::new(edit))
    .tool(Arc::new(delete))
    .approval_handler(approval_handler)
    .build()?;
```

The snapshot handle is Runtime-memory-only. It reaches the active model but is removed from runtime
events, hook observations, approval presentation, returned durable messages, transcript, and TLOG.
Hooks that leave the sanitized projection unchanged do not disturb the active model payload; a hook
that rewrites it intentionally replaces the private payload and may trigger a recoverable re-read.
Rebuilt runtimes must read again before an anchored edit. Legacy `ReadTool::new` and
`EditTool::new` remain available without snapshot behavior.

### Pattern 3: Prompt Customization

- `custom_prompt(str)` — **Replaces** the default Talos system prompt entirely.
- `append_prompt(str)` — **Appends** domain-specific instructions to the default prompt.
- Both can compose: `custom_prompt` sets the base, `append_prompt` adds to it.

### Pattern 4: Request Preview

```rust,ignore
handle.preview_request("What would you send for this?")?;
// Collect events — TurnCompleted.final_text contains the serialized request
// without making an actual API call.
```

## Permission Model Summary

| Tool Nature | Default Behavior | With Approval Handler |
|---|---|---|
| Read | Auto-allowed | Not called (no need) |
| Write / Execute / Network | `Ask` → denied without handler | Handler decides per call |
| Hybrid (multi-facet) | Most restrictive facet wins | Each facet evaluated |

`PermissionRule` entries are evaluated before the engine's default fallback. Rules can `Allow`,
`Deny`, or `Ask` for specific tools, paths, or operation types.

## Planned Additions (ADR-052 — Not Yet Implemented)

[ADR-052](../decisions/052-sdk-publication-and-composition-boundary.md) decided the following SDK
surface additions. They are **design commitments, not shipped APIs**; this section is a forward
contract and MUST NOT be read as "already available." Each lands through ARCH-031 slices under
iteration governance, and this document is updated to the "Supported" tables above only when the
implementation commit exists.

### Caller-selected sandbox fallback

When sandbox isolation is unavailable, the SDK will expose an explicit, caller-selected policy
instead of silently choosing a product default:

```rust,ignore
pub enum SandboxFallbackPolicy {
    Deny,             // reject sandbox-required execution when isolation is unavailable (default)
    Ask,              // route the unsandboxed fallback decision through the approval mechanism
    AllowUnsandboxed, // caller explicitly accepts direct execution for that runtime
}
```

- Default is `Deny` (omission is fail-closed).
- `talos-sandbox` remains policy-neutral (typed availability/errors only; no runtime/UI policy).
- **Orthogonal to permission policy:** `AllowUnsandboxed` never grants any tool/path/execute/network
  permission. Normal permission evaluation (rules, tool natures, `Deny` precedence) still runs in
  full; the fallback only decides whether execution may continue when isolation is unavailable.
- **`Ask` is a distinct, scoped approval:** it MUST carry an identifiable sandbox-fallback
  reason/context to the approval layer (not the same meaning as a normal tool-permission approval);
  authorization is scoped to at least the current invocation/runtime (never an implicit permanent
  allowance); with no approval handler it MUST fail closed (equivalent to `Deny`); a normal
  `AlwaysApprove` tool-permission rule MUST NOT auto-permanently-allow unsandboxed execution.
- Replacing any existing sandbox boolean/implicit fallback follows the pre-1.0 change policy below,
  with a migration note and, where practical, one minor cycle of deprecated compatibility.

### Official coding preset

`RuntimeBuilder::new()` stays minimal and composition-first; an explicit, overridable preset will
reproduce Talos-owned coding defaults without copying internal registry construction:

```rust,ignore
let runtime = RuntimeBuilder::new()
    .preset(RuntimePreset::coding())
    .provider(provider)
    .workspace_root(workspace)
    .sandbox_fallback(SandboxFallbackPolicy::Ask)
    .build()?;
```

- The preset is explicit, inspectable, and overridable; it never hides write/execute/network
  actions from the permission pipeline.
- **Precedence:** explicit caller configuration (permission rules, sandbox policy, tool selection)
  overrides preset defaults. A preset MUST NOT override or weaken an explicit `Deny`, a permission
  rule, or a sandbox requirement. A preset only provides default composition and gains NO additional
  authorization capability.
- It must construct through the same shared registry and safety pipeline as the product CLI so CLI
  and SDK share tool registration, permission defaults, and session semantics (verified by tests,
  not documentation alone).

### `talos-tools` default surface

Under ADR-052 the SDK's default tool surface is **local read-only** (file-read + search). Write,
shell, git, network/web, image, and heavy code-intelligence families are opt-in via explicit
features or the coding preset. Embedders that need the former broad set must select features or the
preset explicitly. This changes default transitive dependency/capability behavior and will be listed
in release notes when it lands.

## Pre-1.0 Change Policy

- **Additive changes** (new builder methods, new event variants, new handle methods) may land in
  any pre-1.0 release without a major version bump.
- **Breaking changes** to existing method signatures or type shapes require a new minor version
  and a migration note in the release changelog.
- **Removals** of public items require deprecation for at least one minor version cycle.
- The 1.0 stability promise is gated by [REL-002](../backlog/active/REL-002-v1-self-bootstrap-release-gate.md).
