# 051: One-Shot Multimodal Tool Continuation Boundary

> Status: Accepted
> Date: 2026-07-22
> Iteration: I154 (MODEL-009-E)
> Gate: This ADR is the implementation contract for `read_image`. A tool implementation that
> cannot satisfy every decision below must remain unregistered.

## Context

MODEL-009-D lets a user explicitly attach a locally validated image to their next message. I154
adds a distinct, model-invoked `read_image` tool: after the model explicitly requests one local
path and the normal permission pipeline authorizes it, the following provider request must receive
that image exactly once. A normal text path and the existing text `read` tool must remain unable to
read an image.

The current `AgentTool` boundary returns textual `ToolResult` values. Encoding image bytes or a
base64 data URL in that result would leak binary data into model-visible tool text, UI events,
history, exports, and persistence. A mutable side channel owned by the tool would also make
parallel tool-call ordering and retry behavior ambiguous.

## Constraint Decomposition

| Constraint | Type | Source | Decision impact |
| --- | --- | --- | --- |
| A tool may not bypass path permissions | Hard | ADR-047 / AGENTS.md #4 | `read_image` uses its own exact `read_image` capability, never `attach_image`. |
| Image input is fail-closed | Hard | ADR-050 | Only `ImageInputCapability::Supported` may see or execute the tool. |
| No binary/full-path leakage in retained text | Hard | ADR-023, ADR-035, ADR-050 | Artifact is separate from `ToolResult`; retained output is a safe summary only. |
| Public APIs are semver-bound | Hard | AGENTS.md #6 | Do not add a required field to public `ToolResult`; use additive API. |
| No global event bus or hidden coupling | Hard | ADR-006 | The continuation is scoped to one agent round, not a registry/global side channel. |
| Provider JSON remains adapter-owned | Hard | ADR-013, ADR-050 | The artifact uses `ContentPart`; only adapters make OpenAI/Anthropic JSON. |
| Decoder and file reads must fail safely | Hard | AGENTS.md #9, ADR-050 | Reuse bounded validation and provider-side digest revalidation. |

## Decision

### 1. Add an additive execution-output boundary

`talos-core::tool` gains a public, documented additive `ToolExecutionOutput` containing a normal
`ToolResult` plus `next_provider_parts: Vec<ContentPart>`. `AgentTool` gains an additive
`execute_authorized_with_output` method whose default delegates to the existing
`execute_authorized` and returns no parts.

`ToolResult` itself is not changed. This preserves existing struct literals and avoids treating a
provider artifact as textual tool output. Existing tool implementations remain behaviorally and
source compatible; the two permission wrappers must forward the new method after they have
obtained the same authorizations they would use for `execute_authorized`.

### 2. `read_image` is a normal, separately authorized file tool

`ReadImageTool` lives in `talos-tools`, declares tool name `read_image`, `ToolNature::Read`, a
single required local `path` input, `ToolFamily::File`, and an exact path permission facet. It
must re-resolve and check a `ToolExecutionAuthorization` for that tool, nature, and canonical path
before producing an artifact. An authorization for `attach_image`, text `read`, another path, or
another resource kind never authorizes it.

The tool is registered and presented only when the session's active model capability is
`Supported`; `Unknown` and `Unsupported` omit its schema. Execution repeats the capability check
so a stale presentation cannot read bytes after a model change.

### 3. Move shared ingestion to the file-tool boundary

The MODEL-009-C validation/content-digest construction currently used by CLI attachment handling
must move to a documented `talos-tools` image-validation module (or an equivalently narrow shared
module owned by `talos-tools`). `talos-cli` calls that shared function for `/attach` and `--attach`;
`ReadImageTool` calls it after authorization. The migration must preserve all existing limits and
tests: regular-file/canonical-path checks, magic MIME detection, byte/count/pixel limits,
panic-contained decoding, SHA-256 digest creation, and no raw path display.

`talos-provider::image_io` remains the final boundary: immediately before encoding it must
canonicalize again and compare the stored digest to the bytes it reads. `talos-provider` must not
depend on `talos-tools`.

### 4. One successful call creates one one-shot overlay

For the MVP, one successful `read_image` call yields one image part. The agent collects the
execution output together with the normal text result, appends only the projected `Message::Tool`
to durable turn state, and creates an ephemeral provider-message overlay containing the image part
for the immediately following `stream_with_tools` call.

The overlay is never appended to the session transcript, TLOG, resume state, compaction input,
hook payload, scrollback, copy, export, or a later provider round. It is consumed and dropped when
that following provider invocation begins, even if the provider fails; automatic retry must not
send the image a second time. A failed/denied/invalid call returns only a safe error `ToolResult`
and produces no overlay. The tool result for success is bounded to:

```text
[Image read: `<basename>` (<byte_count> bytes, <mime>); attached to next provider request]
```

It contains no canonical path, digest, raw bytes, or data URL. An interactive approval may show
the canonical path ephemerally because approval must be meaningful for the exact capability, but
that value must not be copied into retained result/history/export/log surfaces.

### 5. Preserve protocol-valid ordering inside adapters

The ephemeral item remains provider-neutral `ContentPart` data until `talos-provider` builds a
request. The adapter behavior is explicit and fixture-tested:

- OpenAI-compatible: serialize the normal tool result followed by one user multimodal message.
- Anthropic-compatible: when the transient image follows a tool result, coalesce that tool-result
  block and the ordered text/image blocks into one user content array, rather than emitting two
  consecutive user messages.

Normal user attachments keep their current request behavior. The image must appear exactly once in
the immediate request and in neither the preceding request nor a subsequent text-only request.

## Semver Impact And Migration

This is an additive pre-1.0 public API change: downstream custom `AgentTool` implementations need
no change because the new method has a default implementation. Downstream wrappers that intentionally
intercept authorized execution should forward `execute_authorized_with_output`; otherwise their
wrapped tools correctly produce no continuation parts. `ToolResult` remains unchanged. The next
release containing implementation must be a minor version bump under the project pre-1.0 policy.

## Rejected Alternatives

- **Put base64 in `ToolResult`**: leaks into retained text and creates unbounded text payloads.
- **Store the artifact in a global registry, tool singleton, or event bus**: makes concurrent
  calls, cancellation, and retries non-local and violates ADR-006's coupling boundary.
- **Reuse `attach_image` authorization**: would make approval identity broader than the actual
  tool invocation.
- **Make ordinary `read` multimodal**: changes an established text tool contract and turns a
  pasted path into an ambiguous file-read trigger.
- **Persist the transient overlay**: makes session resume/replay send an image that was intended
  for only one provider call.

## Reversal Trigger

Revisit this decision if a future provider needs an upload/media-ID flow, if one provider cannot
represent the required tool-result/image ordering, or if the product explicitly requires durable
tool-produced multimodal artifacts. Any such change needs a new ADR and iteration rather than
silently broadening I154.

## Related

- ADR-006: Event Architecture Boundary
- ADR-013: Provider Config Schema Boundary
- ADR-023: Inline API Key Storage and Display Boundary
- ADR-047: External-Path Tool Authorization
- ADR-050: Multimodal Image Input Architecture And Security Boundary
- MODEL-009-E / I154

## Implementation Facts

Implemented 2026-07-23 across commits `6d4677e`–`13bc157`. All automated gates pass.

| Decision point | Implementation | Evidence |
| --- | --- | --- |
| 1. Additive `ToolExecutionOutput` + `execute_authorized_with_output` + `execute_with_output` | `talos-core/src/tool.rs`: `ToolExecutionOutput` struct with `result` + `next_provider_parts`; trait methods with default impls preserving source compatibility. Both `PermissionAwareTool` and `TuiPermissionAwareTool` override `execute_with_output` to perform the same approval flow and return the full output. | Commit `6d4677e`, `5eeb8e1` |
| 2. `read_image` is a separately authorized file tool | `talos-tools/src/read_image_tool.rs`: name `read_image`, `ToolNature::Read`, `ToolFamily::File`, `permission_profile` returns `ToolPermissionFacet::with_resource(Read, path, Path)`. `execute()` is a safety stub that returns an error without reading the file. `execute_authorized_with_output` calls `resolve_authorized_path` for exact-path authorization. | Commit `9009096`, `4a0616a` |
| 3. Shared ingestion moved to `talos-tools` | `talos-tools/src/image_validation.rs`: shared module with `create_image_content_part`. `talos-cli` re-exports it. Provider-side `image_io::read_image_with_toctou_guard` remains in `talos-provider` with no `talos-tools` dependency. | Commit `ad46eba` |
| 4. One-shot overlay | `talos-agent/src/lib.rs`: `pending_continuation_parts` collected from tool execution, injected as `Message::Multimodal` via `std::mem::take` before `stream_with_tools`. Never persisted. `execute_single_tool_with_presentation` calls `execute_with_output` instead of `execute`. Batch limit: `AtomicUsize` quota rejects 2nd `read_image` before execution. | Commit `5eeb8e1`, `bc38112` |
| 5. Protocol ordering | OpenAI: tool result (`tool` role) then multimodal (`user` role) as separate messages — fixture test proves order. Anthropic: `coalesce_consecutive_user_messages` merges consecutive user messages into one content array — fixture test proves coalescing. | Commit `4a0616a`, `13bc157` |
| Capability gate | `Agent.image_input_supported` field; filtered from `presented_tool_names` and `tool_definitions` at construction and in `run_inner`. Execution-boundary check in `execute_single_tool_with_presentation` rejects `read_image` when `!image_input_supported`. `set_image_input_capability` helper wires from model metadata at all construction sites. | Commit `2270f21`, `4a0616a` |
| Path sanitization | `execute()` and `PathEscape` error messages contain no raw path. Tests assert path non-disclosure. | Commit `4a0616a` |
| Tests | 9 tool unit tests + 4 agent integration tests + 1 Anthropic coalescing fixture + 1 OpenAI continuation fixture + 4 permission chain tests = 19 new tests. | Commits `36d987c`, `9ecca94`, `4a0616a`, `13bc157` |
