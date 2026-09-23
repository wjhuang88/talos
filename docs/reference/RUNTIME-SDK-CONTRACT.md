# talos-runtime SDK Support Contract

Created: 2026-06-30 (T13 of the four-month self-bootstrap plan)

This document defines the support boundary for embedding Talos as a Rust runtime. It is a
pre-1.0 contract: the surface is usable but not yet semver-stable. REL-002 gates the 1.0 promise.

## Supported Embedding Surface

Embedders should depend on **`talos-runtime`**. This contract covers `talos-runtime`'s own public
API plus the types it explicitly `pub use`-re-exports. I280 source development exposes the
provider, tool, permission, sandbox and session-event signature types through this facade,
so custom implementations need only one direct Talos dependency. These additive imports are
not included in published v0.10.0. Origin-crate paths remain compatible; optional built-in
providers may still come from `talos-provider`. Unlisted lower-level APIs retain their own
support boundaries. See [I280 migration](I280-RUNTIME-FACADE-MIGRATION.md).

### Builder and Handle (defined in `talos-runtime`)

| Type | Role | Stability |
|---|---|---|
| `RuntimeBuilder` | Configure and construct an embedded runtime | Pre-1.0 stable shape; method set may grow |
| `RuntimeHandle` | Interact with a running runtime (submit, events, shutdown) | Pre-1.0 stable shape; method set may grow |
| `RuntimeShutdownHandle` | Cloneable shutdown-only controller; starts or joins one bounded plan | I216 additive API |
| `ShutdownOptions` / `ActiveTurnPolicy` | Validated total timeout and active-turn policy | I216 additive API |
| `ShutdownReport` and shutdown outcome enums | Immutable redacted terminal shutdown projection | I216 additive API |
| `ShutdownFinalizerId`, `ShutdownFinalizerReport`, `ShutdownFinalizerOutcome`, `ShutdownFinalizerRegistryError` | Fixed runtime-owned finalizer projection and closed construction errors; no public callback registration | I217 additive API |
| `collect_until_turn_completed` | Helper to drain events until a turn finishes | Pre-1.0 |
| `RuntimeError` / `RuntimeResult<T>` | Error types for runtime operations | Pre-1.0 |
| `ApprovalHandler` | Trait embedders implement to bridge `Ask` decisions; defined in `talos-runtime` | Pre-1.0 |
| `RuntimeBuilder::shared_tools` | Explicitly opt into the shared Talos built-in tool contribution inventory when the `shared-composition` feature is enabled | I160 pre-1.0 additive API; not selected by `RuntimeBuilder::new()` |

### Re-exported Protocol Types (actual `pub use` in `talos-runtime`)

These are selected types `talos-runtime` re-exports. For the complete named export set,
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
| `RuntimeTurnCompletionStatus` / `TurnCompletionStatus` | `talos_core::session::TurnCompletionStatus` | The existing alias and new I280 bare name denote the same turn outcome type. |
| `ToolNature` | `talos_core::tool` | Risk classification: Read / Write / Execute / Network |
| `ToolProvenance` | `talos_core::tool` | Tool origin: `Native`, `McpRemote { server }`, or `Plugin { name, version, carrier }` (ADR-028) |
| `RuntimeHookRegistry` | `talos_plugin::HookRegistry` (re-exported under this alias) | Hook registry used by `RuntimeBuilder::hook_registry` |
| `RuntimeSkillIndex` | `talos_skill::SkillIndex` (re-exported under this alias) | Skill index used by `RuntimeBuilder::skill_index` |

### I280 facade signature closure (source development)

These canonical types are explicitly exported at the `talos_runtime` root in I280.
Their origin crates need not be direct dependencies for these embedding operations.

| Type / Trait | Canonical origin | Runtime SDK contract coverage |
|---|---|---|
| `LanguageModel`, `ProviderResult`, `Receiver`, `DecisionRequestLimits`, `ProviderProgress` | `talos-core` | Custom provider implementation and bounded request signatures |
| `AgentTool`, `ToolResult`, `ToolExecutionOutput`, `ToolExecutionAuthorization`, `ToolPermissionFacet` | `talos-core` | Custom tool execution and authoritative permission profiles |
| `Message`, `ContentPart`, `AssistantReasoning`, `ReasoningBlock` | `talos-core` | Initial history and typed message contents |
| `SessionEvent`, `TurnEventPayload` | `talos-core` | Typed progress and completion consumption |
| `ApprovalChoice` | `talos-core` | Enum returned by `ApprovalHandler::request_approval` |
| `PermissionRule` | `talos-permission` | A rule type (not a trait); accepted by `RuntimeBuilder::permission_rule` |
| `PermissionDecision`, `ResourceKind`, `GrantPreview` | `talos-permission` | Rule construction and bounded scoped approval preview |
| `SandboxProvider`, `SandboxConfig`, `SandboxResult`, `SandboxError`, `create_sandbox` | `talos-sandbox` | Custom sandbox implementation or canonical platform factory |
| `DurableSession`, `PersistencePolicy`, `SessionManager` | `talos-session` | Supported durable-session builder composition |
| `AgentError`, `SessionError` | `talos-agent`, `talos-session` | Canonical errors in runtime construction and durable-session signatures |
| `GrantPreviewFacet`, `GrantScope` | `talos-permission` | Inspect compiled scoped approval previews without reconstructing authority |
| `ToolContinuation` | `talos-core` | Typed continuation of an admitted tool execution |
| `StructuredSubmission`, `SubmissionItem`, `SubmissionKind`, `SubmissionSource`, `PendingSubmissionState`, `SubmissionReceiptDisposition`, `SubmissionRejectionReason` | `talos-core` | Structured submission and typed session queue projections |
| `BackgroundJobId`, `BackgroundJobState`, `BackgroundJobRequest`, `BackgroundJobPermit`, `BackgroundJobLauncher`, `LaunchedBackgroundJob`, `ToolExecutionAdmission` | `talos-core` | Background-job signature types for admitted tool execution |
| `BackgroundCleanupOutcome`, `BackgroundJobTerminalSummary`, `BackgroundOutputChunk`, `BackgroundOutputStream`, `BackgroundProcessControl`, `BackgroundProcessEvent`, `BackgroundProcessExit` | `talos-core` | Typed background-process control, output and terminal outcomes |

The single-direct-dependency promise covers custom provider/tool/approval/sandbox implementations,
typed event consumption and runtime composition. For durable sessions it covers
`SessionManager::with_dir`, `SessionManager::create_or_open_session`, `DurableSession`
composition and `DurableSession::read_messages`. Other management methods reachable through
`SessionManager` retain the original session crate's independent contract; their presence on
the canonical type does not promise a complete facade dependency closure.

Likewise, language-provider, hook and evaluator extension APIs are not claimed as complete
single-dependency extension ecosystems. Existing named exports remain available, but adapters
using those broader extension surfaces may need their origin crates. Re-exported admission,
grant and background-job types do not grant execution authority or bypass permission checks.

### Extension types and traits used by embedders

Embedders typically implement or supply the following. Only `ApprovalHandler` is defined in
`talos-runtime` itself; the canonical lower-level types below are available through its
explicit facade exports in I280.

| Type / Trait | Defined in | Supplied via |
|---|---|---|
| `LanguageModel` | `talos-core` | `RuntimeBuilder::provider` |
| `AgentTool` | `talos-core` | `RuntimeBuilder::tool` |
| `ApprovalHandler` | `talos-runtime` | `RuntimeBuilder::approval_handler` |
| `PermissionRule` | `talos-permission` | `RuntimeBuilder::permission_rule` (rule, not a trait) |
| `GrantPreview` | `talos-permission` | `ApprovalHandler::request_scoped_approval` |
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
5. **Publication does not establish SDK support.** `talos-agent` is published as an
   implementation dependency. Under [ADR-052](../decisions/052-sdk-publication-and-composition-boundary.md),
   registry availability does not promote it to a second supported SDK entrypoint.

## Embedding Patterns

### Pattern 1: Minimal Turn Loop

```rust,ignore
use talos_runtime::{RuntimeBuilder, SessionEvent, TurnEventPayload};
// provider: Arc<dyn LanguageModel>

let mut handle = RuntimeBuilder::new()
    .provider(provider)
    .workspace_root(".")
    .build()?;

handle.submit("Hello, what can you do?").await?;
while let Some(event) = handle.next_event().await {
    if matches!(event, SessionEvent::TurnEvent {
        payload: TurnEventPayload::Completed { .. }, ..
    }) { break; }
}
handle.shutdown().await?;
```

`submit` returning success means the command crossed the SDK admission fence and entered the
bounded Session queue; it does not mean the model turn completed. Once shutdown closes admission,
new submissions return `RuntimeError::RuntimeClosing` without enqueueing.

### Session-Owned Background Process Controls

When a live `AppServerSession` is constructed, the model-visible `process` tool is registered for
that session. It can inspect or cancel only jobs admitted by the same session's existing permission
pipeline. The supported actions are `read`, `status`, `list`, and `cancel`.

`process(read)` returns bounded output events and a byte cursor. Pass the returned `next_cursor` to
the next read; a cursor may resume inside one output chunk, and `dropped_before` reports output
evicted by the fixed retention bound. `wait_ms` is a bounded long-poll hint, not a reason to busy-
poll. `max_bytes`, wait duration, retained output, and the number of retained terminal jobs are
all hard bounded. `process(cancel)` is idempotent and uses the supervisor's existing termination
and reap path. Unknown or foreign job identifiers fail closed without metadata disclosure.

This control surface does not attach to arbitrary PIDs, persist across restarts, provide stdin/PTY
control, add scheduling or autonomous follow-up turns, or change Windows background support.

For an intentionally long-running `bash` or one-command `exec` invocation, set the explicit
`background: true` input. The start result contains an opaque `job_id`; use the `process` tool
with `status`, `read` (passing each returned `next_cursor`), `list`, or `cancel`. Use foreground
execution for finite commands whose output is needed immediately. Do not infer background intent
from shell syntax such as `&`, `nohup`, `Start-Job`, or `-d`, and do not busy-poll `read`.
The CLI and TUI render a bounded job summary while retaining the structured result for the model;
foreground tool output keeps its existing projection.

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

Before the actor is joined, shutdown observes actor-owned durable reconciliation and then runs the
build-time frozen Talos-owned finalizer registry once in fixed order. Every entry shares the
original total deadline; its own cap can shorten but never extend that deadline. The report exposes
only fixed code-owned identifiers and typed `Completed`, `Failed`, `Panicked`, `TimedOut`, or
`NotRunDeadline` outcomes. There is no supported public API for registering arbitrary callbacks,
plugins, identifiers, or error text. The current default runtime composition has no resource
finalizers, so `ShutdownReport::finalizers()` is empty unless reviewed Talos-owned composition code
installs one in a later governed change. See
[I217 Runtime Finalizer Migration](I217-RUNTIME-FINALIZER-MIGRATION.md).

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
sandbox selection, and caller overrides remain runtime concerns. The existing
`RuntimePreset::coding()` and `SandboxFallbackPolicy` provide separate composition controls.

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

### Scoped approval and runtime lifetime

`RuntimeBuilder` defaults to `PermissionMode::Headless`. Interactive embedding applications
must explicitly select `.permission_mode(PermissionMode::Interactive)` and supply an approval
handler. The mode is re-exported by `talos-runtime`; selecting it grants no permissions.
Existing callers keep the headless default without source changes.

`.auto_assistance(true)` enables model-assisted review only when a manual approval handler is
available as fallback; it does not implicitly switch the permission mode. Shell eligibility
still follows the permission context and policy. If the managed workspace lease cannot be
created, the manual handler remains available and `auto_report_sink` receives an `unavailable`
report with reason `workspace_lease_unavailable_manual_approval_retained`. The failure must
not silently remove the user's approval path.

`RuntimeBuilder::approval_handler` receives `Ask` decisions through
`ApprovalHandler::request_scoped_approval`. Its `GrantPreview` is compiled by `talos-permission`
from the authoritative tool profile; handlers should render that preview instead of reconstructing
scope from raw arguments. The default method delegates to `request_approval`, so existing handlers
remain source-compatible.

- `ApprovalChoice::ApproveOnce` produces non-stored authority consumed at one official adapter
  admission.
- `ApprovalChoice::AlwaysApprove` installs a first-class in-memory Session grant. It is separate
  from `PermissionRule`, is never serialized, and belongs only to the `RuntimeHandle` built by that
  `RuntimeBuilder::build()` call.
- A new runtime, including one built around the same durable transcript, starts with an unrelated
  empty grant store. Durable resume does not restore grants.
- Every matching Session grant binds complete tool provenance and all compiled facets. Policy deny
  and restriction state are rechecked at the final admission fence; clearing or changing relevant
  state before admission invalidates pending authority.

Direct consumers that need to construct or inspect first-class grant state must depend on
`talos-permission`. See
[I219 Scoped Grant Migration](I219-PERM006B-SCOPED-GRANT-MIGRATION.md) for the v0.9+ source and
schema migration.

## Explicit Composition And Sandbox Policy (ADR-052)

[ADR-052](../decisions/052-sdk-publication-and-composition-boundary.md) defines these existing
composition APIs. I280 changes their import closure, not their execution or authorization policy.

### Caller-selected sandbox fallback

`create_sandbox()` returns a managed platform provider on macOS/Linux. Its additive
`SandboxProvider::cleanup_receipt()` capability returns a cloneable `SandboxCleanupReceipt`,
also re-exported by `talos-runtime`. Runtime captures this receipt before executing tools,
waits before publishing turn completion, and registers a bounded shutdown finalizer.
Receipt failure cannot be reported as confirmed cancellation or successful shutdown.

The public `SeatbeltSandbox` / `BubblewrapSandbox` unit types and their constructors remain
source-compatible. Direct unit providers and existing custom providers default to no receipt;
`None` means unconfirmed through this capability, not proof of cleanup. Embedders needing
managed lifecycle evidence should use `create_sandbox()`. Before directly awaiting a receipt,
stop admitting commands and drop/finish their execution futures. The receipt does not itself
cancel execution. ADR-082 covers ordinary descendants remaining in the owned Unix process group,
not deliberate group escape. No permission or sandbox-fallback policy is changed.

When sandbox isolation is unavailable, the SDK exposes an explicit, caller-selected policy
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

`RuntimeBuilder::new()` stays minimal and composition-first; an explicit, overridable preset can
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
`RuntimePreset::coding()` is a separate opt-in composition API, not an implicit tool default.

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
### Prompt customization authority

`append_prompt` contributes domain guidance without replacing Talos runtime-owned sections.
`custom_prompt` remains an explicit raw identity override for compatibility; callers using it
accept responsibility for restoring any identity guidance they intentionally replace. Hook
contributions are additive prompt modifications and must not be used to erase runtime safety or
protocol sections.
