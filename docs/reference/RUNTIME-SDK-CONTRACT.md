# talos-runtime SDK Support Contract

Created: 2026-06-30 (T13 of the four-month self-bootstrap plan)

This document defines the support boundary for embedding Talos as a Rust runtime. It is a
pre-1.0 contract: the surface is usable but not yet semver-stable. REL-002 gates the 1.0 promise.

## Supported Embedding Surface

### I316 process input migration (next minor only)

The direct `talos-tools` public structs `BashInput`, `ExecInput` and `ExecStep`
gain `memory_limit_mib: Option<u64>`. Existing Rust struct literals must add
`memory_limit_mib: None` to retain default/request inheritance, or
`memory_limit_mib: Some(4096)` for an explicit 4096 MiB policy. JSON callers omit
the field for inheritance; explicit `null` is rejected, as are non-integers and
values outside 64..=65536. A step inheriting an explicit request remains explicit.
Do not serialize `None` as JSON null when constructing tool requests.

This is a source-breaking pre-1.0 minor migration under ADR-098, not authorized
for a 0.10 patch. Windows now enforces the default per-process committed-memory
policy; Unix omission remains best-effort and explicit setup failure refuses
payload startup. No API consumer should interpret these limits as RSS or an
aggregate process-tree quota. Native delivery gates remain tracked by I316.

On Unix, the additive `talos-sandbox::CommandCapture` and
`OwnedCommandOperation::execute_captured` API retain bounded streams while
continuing to drain output. Supply a fresh capture instance for each invocation;
even `Ok(Output)` requires inspecting `timed_out`. Timeout output is returned only
after confirmed cleanup. Shell and exec expose their aggregate cleanup owner via
`AgentTool::cleanup_receipt`; cancellation callers must seal/wait that owner during
host shutdown rather than treating a dropped invocation as cleanup confirmation.

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
| `RuntimeHandle::set_skill_context` / `clear_skill_context` | Queue an activated-skill context change for subsequent provider requests; rejected with `SessionEvent::Error` while a turn is active; not persisted (ADR-092) | I303 pre-1.0 additive API |
| `RuntimeBuilder::skill_read_tool` | Opt-in read-only `skill_read` tool over bound skills with confinement, 2 MiB file cap and 48 KiB chunks; caller tool of the same name wins (ADR-092) | I303 pre-1.0 additive API; not registered by default |
| `RuntimeBuilder::shared_tools` | Explicitly opt into the shared Talos built-in tool contribution inventory when the `shared-composition` feature is enabled | I160 pre-1.0 additive API; not selected by `RuntimeBuilder::new()` |

`shared_tools()` preserves built-in identity for its internally constructed instances,
without granting execution permission. Registration follows builder call order: the
last `shared_tools()` or `tool()` registration of a name wins. Caller tools never
inherit built-in identity from a replaced tool. Cleanup owners from replaced
instances remain retained for runtime shutdown. The coding preset only fills
missing tool names and preserves explicit registrations.

### Project validation tools (I315 merged, next minor)

Shared composition binds project tool discovery to `workspace_root`. A governance
manifest, or any two canonical Board/iteration-index/backlog files, enables the
read-only `agile_summary`; project size and Active status are not prerequisites.
Use it for explicit progress/backlog questions or status needed for accepted work,
not ordinary chat or repeated refreshes without a task need. Language-project
markers independently enable `validate_project`; recognizing a language does not
invent an execution adapter for it. Cargo-only projects expose validation without
agile summary. Missing prerequisites remain explicit unavailable evidence.

Validation input is exactly `mode: plan|run` and
`profile: governance|i076|workspace`. Plan has Read facets and no host launch;
run has concrete Execute facets and normal permission admission. Cargo run may
execute build scripts, fetch dependencies and write artifacts. Neither tool
exposes owner/Board/Issue mutations, activation, publication or release operations.
Descriptions guide model selection; they are not deterministic intent routing.
Runtime shutdown retains the adapter cleanup owner. Caller replacement tools must
retain their own equivalent lifecycle guarantees and do not inherit trust.

Run uses `PreparedNormalToolInvocation`: the dispatcher binds input, provenance,
complete Execute facets and plan identity, then consumes the exact admission
before creating the executor. Bare `AgentTool::execute` refuses run. Evidence
states actual admission/consumption without inferring grant scope or approver.
Custom validation factories must return an operation-local cleanup receipt; a
missing or panicked receipt refuses execution and reports cleanup unconfirmed.
The prepared receipt is retained before dispatch; waiting before execution closes
the unstarted operation, while cancellation after publication waits only for that
operation. Aggregate shutdown ownership remains required as well.

[ADR-097](../decisions/097-validation-host-execution-api.md) removes conversation's
public synchronous `run_validation_plan` in the next pre-1.0 minor, not a 0.10
patch. Internal-only callers migrate to `run_governance_validation`; host checks
await `run_validation_plan_with_executor` with an admitted, bounded executor and
retained cleanup ownership. Synchronous embedders own a Tokio runtime at their
application boundary; conversation creates no hidden runtime or process runner.
The trait does not enforce third-party executor correctness or grant permissions.
I315 implementation merged in #847 as `add669403734d4e2469eb479939066f57027c74d`.
Exact-head CI `41849101811` passed all eight jobs, including native Windows;
standard preflight passed both SDK modes. Independent API/process-security review
and merge CAS passed. See the I315 owner for fixture and platform limits; this
evidence does not authorize a 0.10 patch release of the breaking API migration.

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
| `RuntimeSkillIndex` | `talos_skill::SkillIndex` (re-exported under this alias) | Level 0 skill index used by `RuntimeBuilder::skill_index` (name/description only) |
| `RuntimeSkill`, `RuntimeSkillLoader` | `talos_skill::Skill`, `talos_skill::SkillLoader` | Parsed skills for `RuntimeBuilder::skill_read_tool` and activation |
| `RuntimeSkillActivation`, `RuntimeSkillReadLimits` | `talos_skill::SkillActivation`, `talos_skill::SkillReadLimits` | Bounded, confined Level 1/2 content for `RuntimeHandle::set_skill_context` (ADR-092) |

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

Durable commits preserve assistant reasoning metadata only when
`PersistencePolicy.persist_reasoning` is explicitly enabled (default: disabled),
after the existing credential redaction. Recovery search does not index that
metadata. Same-turn retries compare the filtered metadata as well as role/content.
Older commits that lost reasoning remain readable and are not backfilled: retrying
such a committed turn with reasoning now returns `DurableTurnConflict` rather than
silently accepting different persisted state. Treat the existing committed outcome
as authoritative; do not change turn IDs and repeat effects merely to bypass this
conflict.

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

### Pending-Journal Startup Recovery

Recovery of a genuinely absent pending-submission sidecar returns an empty inbox without creating
directories, SQLite, WAL or schema. Paths containing `..` conservatively retain original
directory/database initialization, since missing-directory creation may resolve them to an
existing journal. Existing directory entries (including final symlinks) retain
the original SQLite recovery path; malformed/dangling parent paths and inspection failures remain
errors. This is an absence snapshot under the store guard, not a cross-instance custody fence.
A later structured submission still commits its journal before acknowledgement, and a subsequent
read must recover that acknowledged work. Existing-journal schema, Running custody reconciliation,
WAL/FULL durability and configured deadlines are unchanged.

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

### Pending Journal Pause During Shutdown

Pausing unstarted custody under the existing store guard returns zero without creating storage
when the pending journal is genuinely absent. This empty result describes an operation snapshot;
it is not a cross-instance admission fence. The Runtime admission/start arbiter still owns closing.
Later normal admission by another store instance remains durable and recoverable.

Existing journals and ambiguous ParentDir paths retain the original immediate transaction,
schema validation, accepted_pending UPDATE, commit, WAL and FULL behavior. Running records are not
relabeled. Invalid parents, permission failures, malformed journals, unsupported schemas and
invalid links remain errors; an existing final symlink retains the SQLite route. Actor terminal
and durable-reconciliation receipts retain their existing meaning. This removes unnecessary
initialization, without promising a universal shutdown latency or changing any deadline.

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

### Opt-in frame-aware browser host (I295 / WEB-007-F)

See [I295 migration and dependency boundaries](I295-BROWSER-HOST-MIGRATION.md).
Browser host types are supplied by an explicit `talos-tools` dependency with `network` enabled;
they are not all re-exported by the Runtime facade. This is a local candidate, not a release claim.

The frame-aware browser v2 contract is an explicit host composition, not part of the default
Runtime inventory. A trusted host creates a `BrowserHost` with a bound `BrowserExecutor`, records
its tab/frame lifecycle state, and registers `ManagedBrowserTool` only when it can route original
argument bytes through the prepared invocation and browser-specific permission resolver. The
default resolver denies; generic tool-name, Network, path, session, or workspace grants do not
authorize browser calls. An approved invocation is one-shot and bound to its exact normalized
request, executor, document/frame epochs, and expiry.

Hosts must treat `BrowserExecutor` as a trusted implementation boundary: it must perform
document-bound actions and reject navigation or lifecycle changes before mutation or observation
release. A deterministic example is available as
`crates/talos-tools/examples/i295_browser_host.rs` and can be run with:

```text
cargo run -p talos-tools --locked --features network --example i295_browser_host
```

The example proves child-frame discovery, exact authorization, one action, and stale-reference
rejection; it is not native Chromium/CDP evidence. Standalone MCP does not advertise or execute
v2 until its raw-ingress duplicate-key proof is available. Existing tools and default Runtime
composition remain unchanged.


## Provider protocol migration (PROVIDER-008-A, unreleased)

Embedders using the optional built-in provider/config APIs must account for the ADR-093 pre-1.0
minor boundary: `ProviderProtocol` and `CatalogProviderProtocol` add `OpenAIResponses` and become
non-exhaustive; downstream matches need a catch-all arm. `ModelConfig` adds optional `base_url`;
explicit struct literals need that field or `..Default::default()`. These are origin-crate APIs,
not new runtime facade re-exports. Custom `LanguageModel` composition remains the existing contract.

`Config::provider_protocol()` reports the effective runtime protocol; the resolver also exposes
provenance. Explicit Chat remains the rollback for confirmed catalog defaults, with no automatic
cross-wire retry. Follow the [full protocol migration](PROVIDER-PROTOCOLS.md) and
[unreleased notes](../releases/unreleased-provider008a.md). No patch release or publication is
claimed by this source-development contract.

## Unreleased I311 configuration literal migration

Consumers using `talos_config::ModelConfig` directly must account for its new optional
`reasoning_supported` field. See [the configuration migration](MODEL-CAPABILITY-INFERENCE.md#rust-configuration-migration)
for source compatibility and the required next pre-1.0 minor release boundary.

## Unreleased I312 manual compaction migration

Direct `talos-agent` consumers must migrate the deprecated legacy
`Compactor::{compact, apply_collapse, apply_autocompact, manual_compact}` methods
to the Runtime/Session operation below. The first three now always return
`CompactionError::CompactionFailed`; `manual_compact` returns unchanged input and
`CompactionStatus::Failed`. They perform neither provider calls nor partial
preprocessing, even for empty/fitting inputs. Keeping the signatures does not
preserve behavior: this is a next-minor migration under ADR-096, not a patch fix.
Bare history/provider references cannot establish request/privacy authority.
Deterministic Compactor utilities remain caller-owned transformations and cannot
authorize dispatch or replacement of durable history.

`SessionOp::Compact { session_generation, operation_id }` and matching
`CancelCompact` expand the public Rust
operation enum. Exhaustive consumers must add a handler (or an explicit unsupported
response). Ship this change only with the next pre-1.0 minor release per ADR-096.
The Runtime facade exposes `RuntimeHandle::compact(operation_id)` / `cancel_compact(operation_id)` and re-exports
`CompactState`/`CompactOutcome`/`CompactFailureReason`. Await transient `SessionEvent::CompactStatus` for
the result; successful enqueue is not successful compaction. IDs are monotonic per
runtime generation; one operation may be pending, 64 terminal receipts are retained,
and duplicate retained IDs return the prior result with `replayed: true`.

Compaction preserves raw history and emits no user/assistant turn. An active turn
defers manual execution until a complete tool-batch boundary or turn completion.
The Actor holds continuation permission while preparing and applying the candidate.
An unchanged continuation reuses its observed request; new steering input creates
a different request. Observation denial or interruption aborts that continuation
without retrying its hook. Interrupt/shutdown cancel pending work.
Status contains bounded enums and numeric estimates, never checkpoint text.
Exhaustive `CompactOutcome` matches must also handle `FailedWithEstimate { before,
reason }` and `CancelledWithEstimate { before }`. `FailedWithEstimate` retains the
whole-request estimate and a `CompactFailureReason` (`Preparation`, `Provider`,
`Timeout`, or `CommitRejected`); `Failed` means no estimate is available. These
failures preserve the prior projection and do not expose provider error strings.
The `CancelledWithEstimate` outcome retains an already measured request count;
`Cancelled` means no count is available. New operations never inherit a previous
operation's measurement; receipt replay retains the original completed value. The
external SDK fixture exhaustively matches and JSON-round-trips every terminal
outcome and failure category, and exercises below-threshold compaction before an
ordinary turn in both dependency modes. Its constructed failure cases verify the
public contract, not execution of each failure path or long-history semantic retention.

Automatic attempts emit `SessionEvent::AutomaticCompactStatus` directly, including
initial request preparation before an ordinary turn starts. The Runtime re-exports
`AutomaticCompactState` and `AutomaticCompactFailure`; variant tags and typed
failure categories are bounded reason codes, with whole-request token estimates.
Consumers should display these as transient progress, reject old generations and
continue waiting for the ordinary turn's terminal event. Automatic status is not a
manual operation receipt and does not complete the turn or write transcript text.
Cancellation before the shared Runtime checkpoint commit gate prevents installation;
cancellation after Applied does not undo that projection. Provider error strings,
summary content and image bytes are never included in these status events.

### Pending I313 recovery receipt compatibility

The local I313 implementation adds version-two recovery receipts for tools that
explicitly permit a bounded recovery view different from their ordinary persisted
result. The Session persistence policy and private/redaction filters still apply;
this does not enable model history search by default. Equal views continue using
version-one digest-only receipts. A version-one-only reader refuses a receipt set
containing version two; it can still read the original transcript. Rollback may
therefore make those tool facts unavailable for model recovery, without deleting
raw history. See the I313 owner for current implementation and validation status.

### Model checkpoint tool registration (I313, next minor)

RuntimeBuilder registers `compact_context` and `history_search` by default.
Applying a checkpoint replaces the covered older prefix. The suffix starting
with the oldest of the ten most recent user messages remains verbatim when
that boundary exists. Carry forward active goals, constraints and unresolved
work from the covered prefix using eligible source references.
`history_search` discovery returns eligible checkpoint source UUIDs. Read their
bounded text with `mode: "source_detail"`, the snapshot, and a source UUID as
`reference`; cite that same UUID in `compact_context`. Durable `search`/`detail`
references belong to a different reference domain and cannot be checkpoint sources.
Source detail currently projects user and public assistant text only, excluding
reasoning, tool arguments, system/context text, tool results and multimodal content.
Unsupported sources or exhausted projection budgets return an explicit error; there
is no raw-text fallback. Each detail is at most 8192 UTF-8 bytes with `truncated`,
and cached text payload per snapshot is capped at 2 MiB. Reads revalidate current
source and permission authority; a configured unavailable durable host cannot fall
back to memory disclosure. Invalid snapshot/source submissions preserve valid
issued state for correction; admission reserves a single operation through its
asynchronous response and terminal receipt.

`history_search.query` is plain text, not an FTS expression. Linear durable history
uses a safely quoted, tokenized phrase; archive recovery uses case-insensitive text
containment. Punctuation is never evaluated as an FTS operator. Empty queries retain
the existing bounded browse behavior; pagination remains bound to the original text.
The model query limit remains 1024 UTF-8 bytes. `SessionIndex::recovery_text_candidates`
escapes that bounded input internally; `recovery_candidates` retains its raw FTS
syntax and original 1024-byte limit.
A caller registration using either reserved name makes construction fail; it is
not silently replaced. Low-level Agent hosts explicitly enable these tools.
Permission checks still govern calls. Model compaction requires Actor admission;
history queries require prepared durable storage and a discovered snapshot.
Unsupported storage returns Unavailable. Durable discovery may return
`compact_available: false` with no compaction sources while permitting history
search; this does not authorize compaction of an unsupported source range.

Observe `SessionEvent::ModelCheckpointTerminal` for the final result of each
admitted model operation, including when no provider continuation consumes its
receipt. Correlate by session, generation and model operation ID. Model and manual
operation IDs are separate namespaces: a model terminal must not complete a
pending manual `Compact` command or a whole turn. The event contains only
`CompactOutcome` metadata; it is not a raw transcript entry. Existing consumers of
the non-exhaustive SessionEvent enum may ignore the additive event; status-aware
hosts should handle it and discard stale-generation events.

Unix host preparation can resume initial history verification from completed
pages saved in the existing trusted durable-bindings store. The disposable search
index cannot supply this provenance. Changed source versions require verification
again; this is not protection against modification of the trusted host database.
`DurableSession` root branch IDs are now stable for a durable UUID; discard cached
random root IDs from earlier opens, while retaining original transcript/entry IDs.
The opaque scan receipt permits exact-record cleanup only for its issuing handle;
it does not authorize history queries. The Windows implementation
restores saved progress only after bounded content revalidation under a new retained
lease. Metadata equality cannot certify unchanged content across released leases:
native testing observed a same-length overwrite with identical version fields.
Restoration rereads the saved prefix; each step stays bounded but restart does not
skip that I/O. Until verification finishes, no saved progress grants query authority.
I313 production-hook native Windows validation and full acceptance passed at
implementation `322574bc9707492ea6e9d5232794f73e687071d9` (PR #837), with
exact-head CI `41608342151` and independent API/privacy/storage/security review.
These are deterministic regressions, not live-model quality probes.
Cancellation cleanup is bounded best effort; a
persistent bindings-store failure may retain a receipt and emit a warning.

Unix and Windows durable hosts can call `RuntimeHandle::fork_durable(external_id)` at an idle
Actor boundary. It returns a fresh child UUID, preserves the committed raw prefix
and keeps the original runtime active; existing target identities and busy Actors
are rejected. Open the child through the same durable root for a separate runtime.
The operation does not inherit search-index or checkpoint authority. Interrupt,
shutdown and dropping the receipt request cooperative cancellation; publication
may win that race, so reconcile the target identity before retrying. Kernel I/O
has no hard abort guarantee. Startup performs bounded intent recovery; unresolved
intents are retained and reported, and lock files intentionally remain. The Windows
implementation uses local-volume checks, write-through file creation and
file flush before publication. It does not provide directory fsync or promise
Unix-equivalent namespace durability after power loss. Native Windows crash,
lock/cancellation and Actor fork/reopen tests passed in final exact-head run
`41608342151` before PR #837 merged as `322574bc9707492ea6e9d5232794f73e687071d9`.
Earlier candidate `5a80fe914` / run `41403698582` is historical evidence.

Exhaustive consumers must handle additive `SessionOp::ForkDurable` and
`SubmissionRejectionReason::SessionBusy` in the next minor migration (ADR-096).
ForkDurable requires an in-process receipt channel; serialized wire requests are
unsupported and emit an error. SessionBusy is a temporary refusal, not closure.


### CLI startup history restoration (I313)

TUI, interactive and inline startup restore Session messages using bounded pages
on blocking workers. Ctrl-C cancels startup; corrupt/unavailable persisted history
returns an error instead of silently starting with no history. No file is rewritten.
The new SessionMessageScan API reuses message/tool pairing across pages and supports
legacy JSONL message envelopes, but emits no durable search provenance.
Host message restoration also accepts historical headerless TLOG records beginning
with `E\t`; strict history authority readers still require the creation header.
Empty persisted sessions restore as empty, and their first append writes the header.

Each host restoration step reads at most 2 MiB and decodes at most 8192 entries,
with cooperative cancellation and a CLI two-second step deadline. A single large
message or metadata record accumulates across reads; allocation follows actual
captured file bytes, never a declared length alone. Total restored history and the
pending record occupy linear memory; decoding one complete record is synchronous.
Blocked OS I/O may retain resources after caller cancellation. Malformed TLOG
headers/records, invalid UTF-8 and unsupported source volumes fail explicitly.
Model history indexing/disclosure retains its separate 2 MiB record/page limits;
host restoration does not create model recovery authority. Incomplete final TLOG records and invalid JSONL lines are omitted
as before. SessionStore implementers default to no paged encoding; opt into a
built-in encoding only when it matches the actual store. There is no whole-file
fallback. `/resume` and model switching use the same paged message/tail snapshot.
The tail is the last successfully decoded entry, not proof of the final physical
bytes; skipped JSONL or incomplete TLOG suffixes do not become activation evidence.
Failed resume leaves the original command route active. Model switching retains
the existing retirement/activation fence and retries the exact committed marker
without duplicating it. Dropping a transition read cancels its worker cooperatively,
without adding another Ctrl-C handler. CLI fork stages raw bytes in bounded pages
from that same source version and installs without overwriting an existing child.
Dropping an unpublished fork cleans its transcript, sidecar, index and lineage;
cleanup failures require storage recovery. Filesystems without hard-link support
fail explicitly. This does not add process-crash atomicity or directory-fsync
guarantees. The unreleased recovery candidate binds the host-selected CLI TLOG in
place for permitted user/assistant history_search projections. It retains source seals,
permission generation and opaque references; old tool results still lack recovery
receipts and remain unavailable. Exact-overlap archive recovery preserves original
record IDs and parents, refuses ambiguous or changed sources, and subjects restored
messages to normal request admission and source-bound checkpoint folding. This does
not certify real-provider summary quality or native platform acceptance.

Startup `--fork`, REPL and TUI publish archived or receipt-bound children through
`publish_history_fork`, preserving original records with private origin evidence.
Direct `Session::fork` refuses count-placeholder and receipt-bound histories;
it cannot establish durable child ownership. Ordinary non-archived forks retain
the legacy path. Receipt-backed publication and restart reconciliation are local
candidate implementations; complete host/native acceptance and delivery remain open.
See [ADR-105](../decisions/105-selected-session-history-recovery.md) and the
[recovery owner](../tasks/session-archive-recovery-emergency.md) for candidate evidence
and remaining review/deployment gates.


The unreleased selected-history candidate adds
`DurableRecoverySource::try_lock_for_recovery_read` for bounded synchronous host
reads coordinated with writes through the same Session clone family. The guard is
not content/disclosure authority and must not span provider or tool awaits. Windows
prepared History releases its restrictive source handles while idle and verifies
the original prefix digest again for each read. Archive inventory changes still
revoke evidence. Independently reopened owners and external writers are not covered
by this mutex; source verification and restrictive handles remain required. See
ADR-105's Windows live-history lease correction for limits and pending native gates.

Windows built-in atomic transcript replacement coordinates with recovery reads by
acquiring nontruncating write access to an existing target through publication.
It therefore requires target-file write permission, even if directory replacement
permission alone would permit rename. Readonly/write-denied or reparse targets
are refused without changing original bytes. Sharing conflicts have a bounded wait;
ordinary access-denied errors are not generally retried. This does not replace host
single-writer ownership or make multiple source paths an atomic snapshot.

The unreleased candidate preserves the existing TLOG header, record encoding and
blank separators during durable turn finalization. It validates the existing
records, copies their original bytes, and encodes only the new turn and terminal
marker. An incomplete or invalid existing tail fails without replacing the source;
finalization does not repair that tail. Host-produced records larger than a model
history page remain supported. This currently adds one full raw-transcript buffer
alongside decoded history and does not establish a total memory bound. Receipt publication and compaction migration are separate mechanisms described below.

`SessionStore::replace_with_retained_suffix` is an additive extension point with
a default implementation using the store's own atomic replacement. Built-in TLOG
compaction overrides it to preserve retained record bytes and atomically publish
the new head. A `.tlog` extension alone does not select that implementation; custom
stores keep their own encoding and write behavior. This prevents re-encoding from
creating conflicting IDs between retained records and archives, while receipt/locator custody and keep0 discovery use the separate recovery
protocol described below. Raw suffix preservation alone is not recovery authority.

Archive publication writes a create-new temporary sibling, syncs its contents, then
publishes the final archive name with a no-overwrite hard link before replacing
the head. On Unix the containing directory is synced after publication. Existing
archive destinations are never replaced. Soft compatibility constraint: the
archive filesystem must support hard links; otherwise compaction fails while the
original head remains intact. Replacement trigger: a reviewed portable no-replace
publication primitive with equivalent durability tests. Windows currently has
file-content sync but no proven directory durability. Failure can leave a temporary
alias or complete archive for recovery; this is not an archive/head/receipt crash
recovery protocol or completion of archived-fork support.


Zero-retention compaction of a UUID-backed TLOG now records a private source locator
in `history-bindings.sqlite` after archive sync and before head replacement. It binds
the exact first placeholder record, original logical tail, and archive name/digest.
Recovery follows only locators activated by actual first records, validates exact
source bytes and linear new suffixes, and rejects cycles or changed/missing evidence.
Cold append validates the complete bounded source chain before using its original
tail; it never adopts the synthetic placeholder as the new parent. Existing legacy
zero-retention histories without a locator remain unavailable rather than guessed.
This does not authorize cross-session inheritance or old tool results.
`SessionEntry::is_compaction_placeholder` is an additive descriptive predicate,
not a receipt or ownership check. Host history preparation checks the sealed first
page even when resuming an old completed index cursor, so keep0 markers cannot
silently become empty linear history. Search/detail require validated archive
sources; missing sources leave history unavailable with fixed diagnostic reasons.

Locator reads are read-only/no-follow, use the recovery deadline in SQLite's progress
handler, and check cancellation before/after queries and between rows. Cancellation
inside one SQLite step is currently deadline-bounded, not immediate. Each owner has
at most 256 locators; canonical archive names/digests and a 512-byte original-tail ID
limit bound values copied into Rust. Exceeding these limits fails before head
replacement. Compaction still relies on host single-writer ownership. Process-death
tests cover registration-before-head and published-head restart. Identity custody,
incarnation checks and receipt-backed fork publication/replay implement identity
reuse/deletion and fork inheritance boundaries described below. Native platform and
power-loss durability acceptance remain pending. The feature is an unreleased
recovery candidate; these mechanisms do not establish deployment acceptance.

New UUID TLOG compactions in the recovery candidate use reserved `s-owned-<UUID>` archive
names and an immutable private publication intent committed before file visibility.
Absent ownership is an error for these names, never legacy fallback. Foreign-owned
archives are excluded even after their owner head is deleted; prepared evidence
rechecks all observed ownership. Legacy archive names retain existing proof rules.
Publication intent alone grants no inherited fork authority; inheritance requires
the receipt and incarnation checks described below. Power-loss guarantees and
compatibility with old writers compacting materialized children are not established.

The recovery candidate adds `Session::begin_message_scan_through(entry_id)` and
`SessionMessageScan::finish_with_copy_length()`. These are host raw-copy helpers,
not fork receipts or model-search authority. The scan validates its entire captured
source, rejects absent/duplicate target IDs, and returns the original record-end
byte offset alongside the selected messages/tail. A publisher must truncate its
same-snapshot copy to that offset and sync before publication; ordinary scans still
return the complete captured length. Compaction placeholders remain detectable even
after the selected cutoff, so this API does not enable inherited archived branches.

### Unreleased Gemini invocation migration (I310 / ADR-094)

The next pre-1.0 minor adds `GoogleGemini` protocol variants and optional
`ProviderConfig::gemini`; exhaustive struct literals must initialize it or use defaults.
`ProviderInvocationEvent` becomes non-exhaustive and gains `TransientReplay`; downstream
matches need a catch-all arm. Never serialize, log or persist the private carrier.
`LanguageModel::stream_with_replay_context` has a default that refuses private attachments
and delegates ordinary requests. Existing providers need not implement private replay.

The live AppServerSession Actor owns browser signed-history custody across submissions.
Standalone Agent execution only retains it for that execution. Reopen, durable fork and
generation replacement do not inherit custody. Missing or modified bindings fail before HTTP;
callers must start a fresh conversation rather than replay a durable privacy projection as
original signed content. Native, Compat and TalosStrict share this rule. Browser-registered
textual routes also retain signed ordinary tool frames, so the same continuity limit applies.
Custody is bounded to 32 blocks / 16 MiB. This is an unreleased candidate contract, not a patch
release compatibility claim or live-vendor acceptance. See PROVIDER-PROTOCOLS for configuration.

### Unreleased JSONL direct-store append migration (ADR-105)

JsonlSessionStore::append_entry now opens one read+append handle to inspect the
final byte. A nonempty legacy file without a final newline receives a separator
before the new record, preserving its original bytes and making raw-prefix fork
continuation readable on reopen. Direct-store hosts using append-only file ACLs
must grant the writer read access before upgrading, or retain an explicitly
append-only custom SessionStore. Talos does not change ACLs automatically. A read
or seek failure is returned before writing, with no blind append fallback. Normal
Session callers already require reads for their parent tip. Existing lease and
partial-write semantics remain; separator plus record is not an atomic transaction.

`SessionMessageScan::finish_with_record_copy_length()` returns the end of the last
retained decoded record from the same validated capture. It excludes skipped trailing
JSONL/incomplete TLOG bytes without a second read or unbounded tip preselection.
Selected scans still stop at the selected record; an empty scan returns zero and no
tail. Startup, REPL and TUI fork staging all use this boundary so skipped trailing
bytes cannot prevent the child from continuing. Original bytes through that boundary
are retained exactly; the source remains untouched, and invalid interior JSONL lines
are not rewritten. Ordinary `finish_with_copy_length()` retains physical EOF for
callers that need a complete captured copy. Neither cutoff grants archive inheritance
authority.

### Unreleased identity-lock storage permission change

Built-in UUID-named JSONL/TLOG reads and writes now acquire an operation-local
identity lock before checking retirement. Hosts must provide a writable transcript
parent and read/write access to permanent `.identity-locks/<canonical UUID>` files.
A first read can create missing parent directories and the lock anchor even when
the transcript is absent; it creates neither transcript nor custody database.
Readonly media or read-only directory grants no longer suffice for these built-in
reads. Migrate by provisioning the private lock directory/anchors with host access
or copying input into a host-owned writable session directory; there is no unlocked
fallback. Non-UUID archive names retain their existing source-read checks.
Lock contention is nonblocking and returns unavailable; callers retry operations,
never assume failed retirement succeeded. Lock anchors must never be deleted or
replaced, including during session cleanup. Same-UID external lock/directory
replacement is outside the cooperative custody boundary. Shared identity locks
exclude retirement but do not serialize writers; existing host runtime ownership
and transcript write locks remain mandatory. Native Windows acceptance remains pending. Archived-fork publication is
implemented locally as described below; delivery is not yet complete.

`DurableRecoverySource::lock_identity_for_recovery(deadline)` and
`RecoveryPrefixSeal::lock_identity_for_recovery(deadline)` return an owned opaque
operation guard. Keep it alive through synchronous query result validation; do not
store it with idle cached history or hold across provider/tool awaits. The guard
excludes managed identity retirement only, not concurrent writes, source changes
or permission revocation. Prefix/receipt and permission checks remain required.
Talos History queries acquire this guard per invocation, including archived
search/detail; releasing a query result releases its guard, so later delivery
still requires the host execution/permission fence.

Materialized fork compatibility (local implementation, not delivery evidence):
built-in TLOG append now reads the leading format marker before choosing its write
path, so append requires read access as well as write access. Legacy headerless
TLOG files remain accepted. Receipt-bound materialized heads use validated atomic
append to preserve their exact inherited bytes and reject duplicate IDs or invalid
suffix parents before publication. This imposes the existing atomic publication
requirements on those heads; arbitrary append-only permissions are insufficient.
Built-in receipt-bound compaction preserves the envelope and exact retained raw
records, including repeated keep0/keepN compaction. Cold Session append resolves
the logical tail through owned archives; a placeholder never becomes its parent.
Before archive intent/publication, the writer checks the shared recovery input
budget including the old head plus the new archive and, for compressed archives,
its decoded bytes. This conservative crash-window reservation can refuse a
compaction even if the eventual smaller head would fit. Keep0 head replacement
also requires the locator to match the verified archive digest.
Receipt-bound message scans validate a complete bounded recovery inventory on the
first step, then decode logical originals in entry pages and revalidate custody
and sources at finish. The first step uses the shared64MiB inventory bound, not
legacy single-page I/O. Raw copying does not authorize receipt-bound children;
startup, REPL and TUI use the receipt publisher for archived/envelope sources,
including selected original prefixes where the host exposes a cutoff. Automatic pending-publication preservation and published metadata replay have
the bounded coverage and residuals described below.


Local archived-fork publication API (CLI integration implemented; not yet delivered):
`Session::publish_history_fork` requires a matching retained SessionRuntimeLease,
a fresh absent same-directory canonical UUID.tlog child, its final branch identity,
an optional selected original entry, deadline and cancellation predicate. It
reserves the child identity, captures proven raw history and publishes the private
receipt. It returns the selected tail. Host runtime/index/lineage activation is
still required. On error preserve pending artifacts and do not retry with the
same UUID or blindly delete an uncertain committed child. The runtime lease is
cooperative; independent SDK writers that do not honor it are outside that fence.

`Session::quarantine_pending_history_fork` requires the child's matching runtime
lease and a stopped/joined publisher. It preserves failed pending history outside
the recognized transcript namespace, permanently retires its UUID, and refuses
published receipts. It does not reconcile index/runtime metadata automatically.

Fork publication requires both source and child stores to declare TLOG encoding;
the child must also declare the tlog extension. Custom stores opting in must honor
the built-in TLOG/envelope read/write and custody contracts. A JSONL writer under
a .tlog path is refused before child identity reservation.


Local legacy REPL fork lifecycle: `/fork` owns a pending operation before accepting
another input. Ctrl+C cancels the host receipt wait; the Actor observes receiver
closure and cooperatively stops its publisher while retaining its lease/busy state.
Host completion does not prove that the Actor publisher has joined. Successful
publication requests a runtime switch: the parent Actor is shut down, its scheduler
is cancelled/joined, and the Actor is joined before rebuilding child tools,
permissions, history and runtime. The input channel is retained; runtime events
carry a generation so delayed parent events cannot affect the child. A cancelled
operation that already published a child retains it without switching. Rebuild
failure exits interactive mode with the retained session ID and resume command;
it does not restart the parent automatically. Automatic crash/metadata reconciliation is described below;
production-terminal acceptance remains outstanding.

Startup pending-fork preservation: `SessionManager::new` invokes
`reconcile_pending_history_forks(deadline)` before index and orphan maintenance.
The custom-directory constructor remains side-effect-free; hosts may call the
method explicitly. It discovers canonical heads with validated private pending
intents, obtains both parent and child runtime leases, rechecks custody and
quarantines interrupted heads. Busy owners are skipped, never interrupted.
Published children are excluded. Returned count is not proof of exhaustive
reconciliation; errors, deadline and the 256-entry per-directory recovery bound
can defer work. Missing-head intents and temporary artifacts remain evidence.
`.fork-unpublished` protects same-identity sidecars from automatic orphan cleanup.
Published-child metadata replay follows ordinary index cleanup, as described below.

Receipt branch restoration: SessionManager reopening uses the validated private
receipt's child branch UUID before reading entries. This persists across keep0,
keepN and parent removal. `Session::fork` rejects receipt-bound heads even when
materialized; use `publish_history_fork` or the Actor's `ForkHistory` operation to
create an independently recoverable child. The branch's in-memory root field keeps
the Session constructor convention; no original root-entry identity is invented.

Local runtime snapshot candidate: newly captured fork receipts are version2 and
include the committed source runtime identity (or explicit absence) before
publication. Version1 history remains readable but cannot supply this snapshot.
`restore_history_fork_runtime` requires child runtime ownership and validated source
evidence; it fills missing generation-zero identity, refuses conflicting initial
metadata and preserves later activation state. Automatic published-child discovery invokes this primitive and separately replays
lineage; this primitive alone does not repair indexes. Fork snapshot reads and
restoration use nonblocking local/SQLite locks and the
caller deadline for SQLite VM work. Snapshot reads never initialize schema or
create an absent sidecar. Restore commits schema/generation/identity together;
connection initialization may still create sidecar files before a later error.
Filesystem I/O remains cooperative, and a final source-validation error can follow
a committed metadata write: preserve artifacts and retry idempotently.

Current startup integration supersedes earlier unconnected-candidate notes:
`reconcile_published_history_forks(deadline)` runs after pending preservation and
ordinary index cleanup in `SessionManager::new`. It validates version2 receipt history under child runtime
ownership, restores initial identity and replays exact lineage through bounded
index initialization/registration. Busy children and version1 receipts are skipped;
malformed custody may defer the remaining pass with an error. Partial commits remain
retryable, and returned count includes consistent children. Search transcript
indexing still runs as the separate existing reconcile_index step, whose work is
not covered by this recovery deadline. Production-host acceptance is outstanding.
Startup ordering correction: pending preservation runs first, then ordinary index
cleanup, then published runtime/lineage replay, then orphan maintenance. Replaying
after stale parent-index deletion prevents cleanup from erasing recovered lineage.

### Host startup runtime ownership transfer

Hosts preparing persistent identity/history before Actor startup may acquire a
`SessionRuntimeLease` first and move it into `run_with_runtime_lease`. The Actor
checks the lease against its configured persistence transcript before recovery
or execution; a different filename/format or absent persistence refuses startup.
Ownership remains continuous across host preparation and Actor execution. Ordinary
`run` still acquires ownership at startup, so its callers must separately address
any earlier metadata preparation. REPL uses the transfer. Owned startup history
restoration validates the supplied lease and avoids reacquiring it for archive
recovery, including keep0 heads.

Prepared hosts may alternatively call `adopt_runtime_lease` after configuring
persistence, then use ordinary `run`. Adoption validates the transcript immediately
and refuses replacing existing ownership; `run` revalidates before execution.
TUI initial startup, resume, new-session preparation and ordinary/receipt-backed fork use
this path. Shared startup/REPL fork metadata preparation holds a child lease until
it returns; subsequent runtime construction reacquires before further preparation.
Same-session model replacement reacquires after old Actor retirement, verifies
both durable generation and exact activation, and retains ownership through
marker/history preparation into the new Actor. Acquisition failure refuses further
preparation; this is not an uninterrupted old-to-new lease transfer. Ordinary TUI
fork owns the child before installation and transfers that lease to the Actor.
Pre-transfer errors roll back once after releasing ownership; post-transfer
cancellation and commit/publication errors retain the child for recovery.
Ordinary fork source fencing uses the Actor capture described below; full native
terminal acceptance remains open.

Receipt-backed host fork metadata consumes the child's immutable receipt identity,
not a later read of the parent's runtime state. A receipt with explicit absence
inherits no runtime identity; normal host model selection may still choose one.
TUI persists that effective selection under child ownership before runtime build,
as startup/REPL activation does. Receipt restoration and new activation selection
are distinct operations; absence does not prohibit future child model selection.

Ordinary interactive REPL/TUI forks now obtain CaptureOrdinaryFork from the source
Actor. History bytes and optional identity come from that fixed capture, not later
parent reads; TUI/REPL publish a fresh child under child ownership. Capture excludes
turns/compaction, and receiver loss cancels its worker. Once synchronous staging
starts it completes before cancellation is observed. REPL retains a completed child
without switching when cancellation is observed. TUI's staging guard rolls back an
ordinary child before Actor ownership transfer; after transfer, cancellation or
publication failure retains the child for recovery. This is cooperative ownership and
does not fence external or older writers. Native terminal acceptance remains open.
