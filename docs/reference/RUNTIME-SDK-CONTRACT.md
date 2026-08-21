# talos-runtime SDK Support Contract

Created: 2026-06-30 (T13 of the four-month self-bootstrap plan)

This document defines the support boundary for embedding Talos as a Rust runtime. It is a
pre-1.0 contract: the surface is usable but not yet semver-stable. REL-002 gates the 1.0 promise.

## Supported Embedding Surface

Embedders should depend on **`talos-runtime`**. This contract covers `talos-runtime`'s own public
API plus the types it explicitly `pub use`-re-exports. Lower-level crate types that appear in
builder signatures but are NOT re-exported by the facade require a direct dependency on their
origin crate and are governed by that crate's own independent pre-1.0 support boundary (see
"Lower-level extension types requiring direct dependencies" below). Using a type from an external
project does not imply it is exported by the runtime facade.

### Builder and Handle (defined in `talos-runtime`)

| Type | Role | Stability |
|---|---|---|
| `RuntimeBuilder` | Configure and construct an embedded runtime | Pre-1.0 stable shape; method set may grow |
| `RuntimeHandle` | Interact with a running runtime (submit, events, shutdown) | Pre-1.0 stable shape; method set may grow |
| `RuntimeShutdownHandle` | Cloneable shutdown-only controller; starts or joins one bounded plan | I216 additive API |
| `ShutdownOptions` / `ActiveTurnPolicy` | Validated total timeout and active-turn policy | I216 additive API |
| `ShutdownReport` and shutdown outcome enums | Immutable redacted terminal shutdown projection | I216 additive API |
| `collect_until_turn_completed` | Helper to drain events until a turn finishes | Pre-1.0 |
| `RuntimeError` / `RuntimeResult<T>` | Error types for runtime operations | Pre-1.0 |
| `ApprovalHandler` | Trait embedders implement to bridge `Ask` decisions; defined in `talos-runtime` | Pre-1.0 |
| `RuntimeBuilder::shared_tools` | Explicitly opt into the shared Talos built-in tool contribution inventory when the `shared-composition` feature is enabled | I160 pre-1.0 additive API; not selected by `RuntimeBuilder::new()` |

### Re-exported Protocol Types (actual `pub use` in `talos-runtime`)

These are the types `talos-runtime` currently re-exports. The list is the public re-export set;
verify against `crates/talos-runtime/src/lib.rs` if the surface may have changed.

| Re-exported name | Source | Purpose |
|---|---|---|
| `AgentEvent` | `talos_core::message` | Streaming events during a turn (text delta, tool call, tool result, turn end) |
| `ToolCall` | `talos_core::message` | A tool call request from the model |
| `MessageToolResult` | `talos_core::message` | A tool execution result |
| `StopReason` | `talos_core::message` | Why the model stopped generating |
| `Usage` | `talos_core::message` | Token usage statistics |
| `ProviderError` | `talos_core::provider` | Provider error type |
| `ToolDefinition` | `talos_core::provider` | Provider-facing tool schema |
| `RuntimeTurnCompletionStatus` | `talos_core::session::TurnCompletionStatus` (re-exported under this alias) | Turn outcome: `Success`, `Cancelled`, or `Error`. NOTE: the public name is `RuntimeTurnCompletionStatus`; the underlying `TurnCompletionStatus` is not re-exported under that bare name. |
| `ToolNature` | `talos_core::tool` | Risk classification: Read / Write / Execute / Network |
| `ToolProvenance` | `talos_core::tool` | Tool origin: `Native`, `McpRemote { server }`, or `Plugin { name, version, carrier }` (ADR-028) |
| `RuntimeHookRegistry` | `talos_plugin::HookRegistry` (re-exported under this alias) | Hook registry used by `RuntimeBuilder::hook_registry` |
| `RuntimeSkillIndex` | `talos_skill::SkillIndex` (re-exported under this alias) | Skill index used by `RuntimeBuilder::skill_index` |

### Lower-level extension types requiring direct dependencies

These types appear in `RuntimeBuilder`/`ApprovalHandler` signatures but are **NOT** re-exported by
`talos-runtime`. An embedder who implements or constructs them must add a direct dependency on the
origin crate; that crate's own pre-1.0 support boundary applies, not this runtime SDK contract.

| Type / Trait | Direct crate dependency | Runtime SDK contract coverage |
|---|---|---|
| `LanguageModel` | `talos-core` | Trait; accepted by `RuntimeBuilder::provider`; not re-exported by the facade |
| `AgentTool` | `talos-core` | Trait; accepted by `RuntimeBuilder::tool`; not re-exported by the facade |
| `Message` | `talos-core` | Used by `RuntimeBuilder::initial_history`; not re-exported unless later added |
| `ApprovalChoice` | `talos-core` | Enum returned by `ApprovalHandler::request_approval`; not re-exported |
| `PermissionRule` | `talos-permission` | A rule type (not a trait); accepted by `RuntimeBuilder::permission_rule` |
| `SandboxProvider` | `talos-sandbox` | Trait; accepted by `RuntimeBuilder::sandbox` |

### Extension types and traits used by embedders

Embedders typically implement or supply the following. Only `ApprovalHandler` is defined in
`talos-runtime` itself; the rest are lower-level types listed for orientation and require the direct
dependencies above.

| Type / Trait | Defined in | Supplied via |
|---|---|---|
| `LanguageModel` | `talos-core` | `RuntimeBuilder::provider` |
| `AgentTool` | `talos-core` | `RuntimeBuilder::tool` |
| `ApprovalHandler` | `talos-runtime` | `RuntimeBuilder::approval_handler` |
| `PermissionRule` | `talos-permission` | `RuntimeBuilder::permission_rule` (rule, not a trait) |
| `SandboxProvider` | `talos-sandbox` | `RuntimeBuilder::sandbox` |

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
5. **Publication gate.** `talos-agent` is a gate-before-publish crate (see the `talos-agent` entry
   in [CRATE-PUBLICATION-MATRIX](CRATE-PUBLICATION-MATRIX.md)). It is not on crates.io and may not
   be published until sandbox/tools dependency gates clear. Per
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

`submit` returning success means the command crossed the SDK admission fence and entered the
bounded Session queue; it does not mean the model turn completed. Once shutdown closes admission,
new submissions return `RuntimeError::RuntimeClosing` without enqueueing.

### Pattern 1a: Bounded Shared Shutdown

```rust,ignore
use std::time::Duration;
use talos_runtime::{ActiveTurnPolicy, ShutdownOptions};

let controller = handle.shutdown_controller();
let options = ShutdownOptions::new(
    Duration::from_secs(20),
    ActiveTurnPolicy::FinishCurrent {
        grace: Duration::from_secs(5),
    },
)?;

// Structured methods borrow their handles. Concurrent callers join the first
// valid plan and receive the same immutable redacted report.
let report = controller.shutdown(options).await?;
if !report.is_complete() {
    // Decide host policy from typed outcomes; the report contains no prompt,
    // tool, provider, path, credential, or arbitrary error text.
}

// The source-compatible consuming wrapper remains available. It uses a
// 30-second Interrupt plan and maps incomplete cleanup to ShutdownIncomplete.
handle.shutdown().await?;
```

`FinishCurrent` never admits or starts pending work during its grace. `Interrupt` uses the existing
Session cancellation and ADR-058 finalization path. Both policies share one total monotonic
deadline; cancelling one waiting caller does not cancel the runtime-owned shutdown driver. Dropping
the primary handle initiates the default plan without blocking, while dropping a controller is
inert. See [I216 Runtime Shutdown Migration](I216-RUNTIME-SHUTDOWN-MIGRATION.md).

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

### Pattern 2a: Explicit Shared Built-in Composition

The optional `shared-composition` feature provides the same built-in contribution selection used by
the Talos CLI. It is explicit and does not alter `RuntimeBuilder::new()` or bypass permission
evaluation:

```rust,ignore
let mut handle = RuntimeBuilder::new()
    .provider(provider)
    .workspace_root(workspace)
    .shared_tools()
    .approval_handler(approval_handler)
    .build()?;
```

The feature is not a coding preset: it selects tool instances only. Approval, permission rules,
sandbox selection, and caller overrides remain runtime concerns. `RuntimePreset::coding()` and
`SandboxFallbackPolicy` remain separate ARCH-031-C/I161 work.

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

Under ADR-052 the `talos-tools` default surface is **local read-only** (`file-read + search`). I159
implements compile-time opt-in features for file writes, document extraction, shell, Git,
network/web, image, and heavy code intelligence, plus a `coding` aggregate used explicitly by the
Talos CLI. These Cargo features make code available but grant no runtime permission.

Direct `talos-tools` consumers that relied on the former broad implicit default must select the
needed capability features, or `coding` when the full product-oriented set is intentional. The
future `RuntimePreset::coding()` remains owned by ARCH-031-C/I161 and is not implemented by I159.

## Pre-1.0 Change Policy

I216 marks `RuntimeError` non-exhaustive and adds `RuntimeClosing`, `AsyncRuntimeUnavailable`, and
`ShutdownIncomplete`. This is queued for the next minor release, not a patch release. Existing
external exhaustive matches must add a fallback arm; the repository's independent external fixture
compiles that migration shape. See the dedicated
[I216 migration note](I216-RUNTIME-SHUTDOWN-MIGRATION.md). No workspace version or release state is
changed by I216.

- **Additive changes** (new builder methods, new event variants, new handle methods) may land in
  any pre-1.0 release without a major version bump.
- **Breaking changes** to existing method signatures or type shapes require a new minor version
  and a migration note in the release changelog.
- **Removals** of public items require deprecation for at least one minor version cycle.
- The 1.0 stability promise is gated by [REL-002](../backlog/active/REL-002-v1-self-bootstrap-release-gate.md).
