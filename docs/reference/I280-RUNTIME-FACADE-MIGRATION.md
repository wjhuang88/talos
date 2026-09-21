# I280 Runtime Facade Migration

Status: source development; these additive exports are not included in published v0.10.0.
No release or version bump is authorized by I280.

The supported embedding entrypoint is `talos-runtime`. A custom provider/tool consumer
needs no other direct Talos dependency. Third-party dependencies such as `async-trait`,
`tokio` and `serde_json` remain explicit in the consumer's manifest. Use a checkout
containing I280 until a release explicitly includes it; `talos-runtime = "0.10.0"`
does not provide the new paths.

| Previous origin import | Facade import |
|---|---|
| `talos_core::provider::{LanguageModel, ProviderResult, Receiver}` | `talos_runtime::{LanguageModel, ProviderResult, Receiver}` |
| `talos_core::message::{Message, AgentEvent, ToolCall, Usage}` | `talos_runtime::{Message, AgentEvent, ToolCall, Usage}` |
| `talos_core::session::{SessionEvent, TurnEventPayload, TurnCompletionStatus}` | `talos_runtime::{SessionEvent, TurnEventPayload, TurnCompletionStatus}` |
| `talos_core::tool::{AgentTool, ToolResult, ToolNature}` | `talos_runtime::{AgentTool, ToolResult, ToolNature}` |
| `talos_core::ApprovalChoice` | `talos_runtime::ApprovalChoice` |
| `talos_permission::{PermissionRule, PermissionDecision, ResourceKind, GrantPreview}` | Same names under `talos_runtime` |
| `talos_sandbox::{SandboxProvider, SandboxConfig, SandboxResult, SandboxError}` | Same names under `talos_runtime` |
| `talos_agent::AgentError`, `talos_session::SessionError` | `talos_runtime::{AgentError, SessionError}` |
| `talos_permission::{GrantPreviewFacet, GrantScope}` | `talos_runtime::{GrantPreviewFacet, GrantScope}` |
| `talos_core::tool::ToolContinuation` | `talos_runtime::ToolContinuation` |
| Named `talos_core::submission` protocol types | `StructuredSubmission`, `SubmissionItem`, `SubmissionKind`, `SubmissionSource`, `PendingSubmissionState`, `SubmissionReceiptDisposition`, `SubmissionRejectionReason` at the facade root |
| Named `talos_core::background_job` signature types | Explicit background-job admission, launch, state, control, output and terminal types at the facade root; see the SDK contract table |

These are explicit re-exports of the canonical types, not wrappers or duplicate protocols.
Legacy imports and the `RuntimeTurnCompletionStatus` alias remain valid. Remove an origin
dependency only after migrating all uses; unrelated APIs from that crate are not automatically
part of the facade contract. `talos-provider` remains an optional convenience for built-in
providers, not a required SDK dependency.

The durable-session promise is bounded to `SessionManager::with_dir` and
`create_or_open_session`, `DurableSession` composition and `read_messages` (with
`PersistencePolicy` and `SessionError`). Other session-management APIs retain their
origin crate's contract. Language-provider, hook and evaluator extensions are not
claimed to have a complete single-direct-dependency closure; their existing named
exports are not a blanket re-export of the corresponding crates.

Run the source quickstart with `cargo run --locked -p talos-runtime --example quickstart`.
It implements `LanguageModel` locally and uses facade event types. The other examples
demonstrate tools, approval and request preview with the optional built-in mock provider.
The independent `tests/fixtures/runtime-sdk-external` consumer checks the facade boundary
outside workspace dependency unification; examples alone do not prove that boundary.

Construction and safety policies are unchanged: `RuntimeBuilder::new()` stays minimal,
permission Deny retains precedence, unresolved Ask fails closed, and sandbox fallback
defaults to Deny. Import migration grants no execution authority and does not bypass
admission, persistence or shutdown. Submit is async; consume typed completion events,
then await bounded runtime shutdown. See [SDK contract](RUNTIME-SDK-CONTRACT.md).
