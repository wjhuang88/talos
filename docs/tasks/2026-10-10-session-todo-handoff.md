# Session Todo Handoff — completed

Status: Complete (accepted implementation queue and owner evidence).
Completion Commit: `04022b5ada916f9c01f04b5fa19662d3eb8efe27` (final T6 implementation).

The full queue I309 → T8/I308 → T2+T3 MODEL-014 → T4 → T5 → T1 → T6,
including C0/T7 carryover, is complete in its owners. Final status publication
merged in [PR #857](https://github.com/wjhuang88/talos/pull/857) at
`748557edc4dc7eb95c71dbb3836179880f6a2a85` after governance review and CI
`42046345169`. No release or unrelated owner completion is authorized.

## Read first

- [Final acceptance audit, implementation SHAs and evidence limits](2026-10-10-session-todo-final-audit.md)
  is the concise completion record. It distinguishes current inspection from
  inherited reviews and historical manual observations; it is not a fresh run of every test.
- Product scope and acceptance remain in the owners linked below. The baseline
  sections retained here describe requirements and observations at intake; their
  Pending/Blocked/unimplemented statements are historical, not current task state.
- This coordinator is closed. Its former build-session handles, temporary logs,
  local branches and instructions to resume a gate must not be used as current state.
  New work requires current repository/owner evidence and applicable authorization.

## Delivered scope and authoritative owners

| Item | Owner | Scope retained |
|---|---|---|
| C0 | [I299](../iterations/I299-preserved-presentation-recovery.md) | Preserved TUI/process presentation and trusted display provenance |
| T7 | [I300](../iterations/I300-structured-git-repair.md) | Structured Git resources, diff fidelity, native operation boundaries and publication recovery |
| I309 | [I309](../iterations/I309-gix-090-upgrade.md) | gix 0.90 upgrade, consumer locks, regression/MSRV/security evidence and post-merge dependency baseline |
| T8 | [I308](../iterations/I308-builtin-auto-review.md) | Registered built-in inventory, shared effect-aware Auto review, operation-specific admission/execution, policy/visibility/recovery and platform evidence |
| T2+T3 | [I311](../iterations/I311-model014-capability-inference.md) | Authority/schema, catalog capability/variant inference, explicit negative overrides and custom-model walkthrough |
| T4 | [I312](../iterations/I312-whole-request-compaction.md) | Whole-request compaction, manual control, visible results, raw-history/pairing/privacy and failure/cancellation |
| T5 | [I313](../iterations/I313-model-checkpoint-history.md) | Actor safe-boundary checkpoints and bounded authoritative recovery; discovery/order/concurrency/privacy/persistence |
| T1 | [I315](../iterations/I315-model-validation-agile-tools.md) | Shared bounded validation and supported read-only agile service, explicit applicability and execution permission gates |
| T6 | [I316](../iterations/I316-model-process-memory-limit.md) | Model-selectable memory limit, unchanged 2 GiB default, units/range/inheritance and platform/composition evidence |

I309 and T8 were accepted additions to the original seven deliverables. I309's
published scope, non-goals and five acceptance rows remain in I309. T8's seven
acceptance rows and PERM-007-H/G boundaries remain in I308; Pull authorization does
not authorize unrelated Git operations. I293 search retains independent ownership.
T2/T3 were one consolidated implementation/regression slice; T5 depended on T4.

## Delivery contract and limits

The maintainer authorized serial implementation, stable-stage commits/pushes,
necessary claim/implementation PRs, independent review and gated merges. Each new
scope required an effective target-branch claim; closed claims cannot be reused.
Owners were synchronized before derived indexes. Completion cites existing
implementation SHAs, never the status-only publication commit.

Validation required pinned/locked Rust commands, applicable standard preflight,
exact-head CI, independent API/privacy/permission/security review and merge-time
CAS. Windows cross-checks use cargo xwin/MSVC; native execution is separate evidence.
User-facing tool, setup, model-selection, compaction and memory-limit documentation
belongs to the corresponding owners. No paid probing or unsupported endpoint
inference, release/tag, security waiver or takeover of unrelated owners was included.

The final audit retains known limits, including historical review reliance,
platform enforcement boundaries, no RSS/process-tree memory quota and the absence
of a proven root cause for the earlier allocation/restart-time failures. Original
failures and superseded validation results are preserved in the history below;
this cleanup does not reclassify them as successes.

## Historical record and supersession

The former 7,172-line coordinator is preserved verbatim at
[the immutable pre-cleanup snapshot](https://github.com/wjhuang88/talos/blob/748557edc4dc7eb95c71dbb3836179880f6a2a85/docs/tasks/2026-10-10-session-todo-handoff.md).
That snapshot contains every chronological checkpoint, original authorization,
failed attempt, acceptance amendment and temporary gate handle. It is historical
source material, superseded for current status by the final audit and owners.
Locally, retrieve it with:

```sh
git show 748557edc4dc7eb95c71dbb3836179880f6a2a85:docs/tasks/2026-10-10-session-todo-handoff.md
```

Original seven-task requirements, C0 scope and the cross-cutting compaction
contract are retained verbatim below (headings normalized only). Later accepted
scope/decisions and their validation remain in the linked owners and immutable
snapshot. The [earlier recovery note](2026-10-10-session-todo-recovery.md) is also
historical; its allocation-failure diagnosis was superseded by this handoff.
No second copy of the execution journal is introduced.

## Compaction acceptance and scope boundaries

- Runtime owns hard admission and safe application; the model owns semantic retention.
  Checkpoint fields must retain goal, user constraints, decisions, completed/pending work,
  unresolved failures and next steps with bounded source references.
- Keep recent complete tool exchanges and instructions; never replay tools as a compression
  side effect. In-flight batches cannot mutate Actor history directly.
- Checkpoint provenance must identify covered raw history across repeated checkpoints;
  summary-of-summary must not become the only recoverable evidence.
- Preserve raw durable history and stable prompt prefix. Exclude hidden reasoning, private
  arguments and UI-only output from model-visible recovery unless separately authorized.
- A TLOG archive marker is storage metadata, not proof of a usable semantic checkpoint.
- Trigger/target/reservation and observability require initial, continuation, manual,
  irreducible-budget and missing-usage tests. A successful request after truncation alone does
  not satisfy semantic preservation acceptance.
- MEM-007 broader compression, #143 filters, cross-session memory and complete SESSION-011
  rearchitecture are dependencies or residual owners, not implicit scope additions.

## Original seven deliverables — historical baseline

This document is the durable work queue for the receiving agent. Tasks 1-6 below preserve the six earlier unfinished feature/test tasks; task 7 records the subsequently discovered Git permission defect and blocked publication. The recipient does not need access to the previous session's todo list or conversation. None of these seven tasks is completed.

Session todo IDs are navigation aids only; a new session may not inherit the todo store. This document preserves the requirements independently.

### 1. Model-callable validate and agile

Todo: `f8cdca5b-53c5-4e59-ab7e-a68e5340b09b`; Blocked. No implementation yet.

Expose the existing command capabilities as tools the model can discover and select. Reuse business logic rather than duplicating command behavior. Verify exact supported operations, inputs, outputs and side effects before defining schema. Preserve the permission pipeline and add discovery, execution, invalid-input and failure tests plus user-facing tool documentation.

Verified entry points:
- `crates/talos-conversation/src/engine/commands.rs`: `handle_agile_command` currently ignores its argument and reads the bound workspace through `governance_summary::format_governance_summary`; do not imply arbitrary agile mutation support.
- The same file: `handle_validate_command` supports only governance, builds a shared validation plan and executes it.
- `crates/talos-cli/src/validation.rs`: CLI validate exposes plan/run and governance/i076/workspace profiles.
- `crates/talos-conversation/src/validation.rs`: shared service; internal governance checks are distinct from host-tool checks. Host checks currently use synchronous `std::process::Command::output` without a timeout at that call site. An allowlisted command is not permission authorization. Do not expose this runner as unrestricted/read-only execution; review timeout, cancellation, output bounds, panic/native boundary and permission facets.
- `crates/talos-runtime/src/composition.rs`, `crates/talos-cli/src/registry.rs`, and `crates/talos-cli/src/mode_interactive.rs`: tool contribution/registration surfaces. Determine intended host coverage and crate dependency direction before adding a shared adapter.

### 2. Custom-provider variant selection and context matching

Todo: `655b3c1b-2b30-43fd-91e4-3b8b4b58e263`; Pending. Investigation completed earlier, implementation not completed.

Correct variant-stage triggering and automatic named-model metadata matching through the existing architecture. The maintainer also requested inference of image-input capability, reasoning capability and output limit, not just context size. This expanded scope is recorded in `docs/backlog/active/MODEL-014-catalog-capability-inference.md`; do not lose it or describe it as delivered.

Owners and boundaries:
- MODEL-013 / I212: completed context-only baseline. Exact opaque model ID first; only supported single leading slash/colon prefix normalization; unique candidate required; ambiguity/unknown/missing metadata remain unknown. No fuzzy matching or stripping opaque version/@ suffixes. Explicit context values win.
- MODEL-014: new capability/output/variant expansion, Refinement / Unclaimed. Resolve explicit-negative overrides, inferred/unknown representation, protocol-compatible reasoning invocation, image eligibility and adapter output caps. Inference is not proof of gateway/endpoint support and must not be labeled successful probing.
- MODEL-011 / Issue #124: separate active-probe/evidence authority path; do not claim it complete or silently expand it through catalog inference.
- MODEL-007 / ADR-048: existing declared variant selection and identity semantics. No declared variants means skip variant picker; do not fabricate endpoint support from a preset.

Entry points: `crates/talos-config/src/config.rs`, `model.rs`, `variant.rs`, `tests.rs`; `crates/talos-cli/src/model_lifecycle.rs`, `mode_runtime.rs`, `mode_runners_tests.rs`. Existing tests explicitly enforce that context-only catalog projection does not inherit output limits; expanded behavior requires new scope/authority decisions rather than rewriting historical acceptance silently.

### 3. Custom-model regression coverage and verification

Todo: `f88d7365-eaba-49a0-96cc-b0fa551bcdd0`; Pending; depends on task 2.

Cover custom providers, conditional variant selection, exact/normalized/ambiguous/unknown model identity, context size, explicit overrides (including false capability overrides), image/reasoning/output inference, adapter requests and visible provenance. Focused existing catalog tests passing is a baseline, not evidence that the requested fixes exist. Run the custom-provider user walkthrough and update setup/model-selection docs after implementation.

### 4. Context compaction trigger and observability

Todo: `5c3baae4-1de6-4985-88d2-76458b54c560`; Pending. User reported budget near 100% without visible summary compaction.

Owner: `docs/backlog/active/MEM-005-context-compaction-policy.md`. I298 / MEM-005-A is Complete through #691, Completion Commit `8d4ad50e4f8e9bc5c620db2641056b1c1118183d`, but only delivered request-projection trimming and budget recovery. It explicitly does not complete manual compact or new summary strategy. Establish a new effective claim.

Unify trigger and target with the complete outbound request budget, including stable prompt/tool schemas, reserved output and applicable reasoning budget, rather than history-only estimates. Make success, skipped/no-op and failure reasons observable; cover initial and continuation requests plus manual triggering. Preserve raw durable history, complete tool-call/result pairing, hidden-output boundaries, safe fallback and cancellation. Do not summarize hidden tool output into visible scrollback.

Entry points to inspect: `crates/talos-agent/src/compaction.rs`, `compaction/engine.rs`, `compaction/policy.rs`, `compaction/tests.rs`, `request_plan.rs`, `session.rs`, `session/`, and conversation/CLI/TUI typed-command/status adapters. Existing manual_compact APIs do not by themselves prove runtime wiring or whole-request triggering.

### 5. Model-callable context compaction

Todo: `b0bc5602-72b5-4a17-a67b-45425acd2fa4`; Pending; depends on task 4.

Model must be able to choose active compaction of its session. Route through the session Actor at a safe execution boundary; do not directly rewrite history while executing an in-flight tool batch. Return before/after estimates and explicit success/skipped/failure reasons. Preserve persistence, pairing, permissions and hidden-output boundaries. Add discovery, execution, active-turn ordering, concurrent cancellation and failure tests. Independent effective ownership is required; do not reuse closed I298 authorization. Exact tool name/schema and deferred-operation protocol remain to be designed.

### 6. Model-selectable command memory limit

Todo: `84e1268a-1091-458b-ad3d-546fcf4c8264`; Pending. Maintainer authorized exposing a subprocess memory-limit parameter; no implementation yet.

Cover bash and exec, foreground/background, sequential/parallel/pipeline and per-step input as applicable. Preserve default 2 GiB. Specify unit, accepted range, default/inheritance and invalid-value handling. Explain RLIMIT_AS is virtual address space, not RSS or a total process-tree quota. Do not silently imply enforcement on unsupported platforms.

Verified boundary: `crates/talos-tools/src/process_boundary.rs::apply_process_hardening` configures RLIMIT_CORE=0, RLIMIT_CPU=300 and RLIMIT_AS=2 GiB in Unix pre_exec. setrlimit errors are currently best-effort ignored; this is existing behavior, not a verified portable guarantee. `bash_tool.rs`, `exec_tool.rs` and UnixBackgroundLauncher call the shared boundary. Inspect all paths before changing signatures. Review escape vectors and ADR-007 authorization; do not add unaudited unsafe code. Include permission-boundary/security review and schema, propagation, invalid-value, background/foreground and platform tests plus docs.

### 7. Git permission resource defect and blocked handoff publication

Status: Blocked; newly added maintainer-requested recovery task. This is independent of tasks 1-6 and must not be omitted when transferring ownership.

The maintainer explicitly requested committing and pushing the handoff, then requested a retry. Both structured `git_add` attempts failed before staging succeeded. Identical arguments were used on both attempts:

```json
{
  "paths": [
    "docs/tasks/2026-10-10-session-todo-handoff.md",
    "docs/tasks/2026-10-10-session-todo-recovery.md"
  ]
}
```

Both returned exactly:

```text
Permission denied: permission state error: permission facet 0 has no safe typed resource
```

Publication update: after the two failures, the maintainer explicitly authorized temporary bash Git operations. Host `git status` confirmed an empty index and untracked handoff files; host `git add` of exactly the two documents and `git diff --cached --check` succeeded. Commit/push verification is still required at this writing; use Git history and remote state for the resulting publication SHA. This fallback does not fix the structured-tool defect, which remains outstanding. Historical statements below describe the checkpoint before this authorization.

Observed impact and state:
- No successful git_add from these attempts; git_commit and git_push were not attempted after either denial.
- No commit SHA or remote publication exists for the handoff from this session. The documents remain local. Read-only git_status/git_log/git_diff worked; file read/write/edit also worked. This does not prove other Git mutation tools work or share this defect.
- Intended commit scope was ONLY these two handoff/recovery documents, not the unrelated uncommitted code or MODEL intake changes. The maintainer requested commit and push; preserve that pending publication requirement after resolving the blocker.
- Do not treat the error as Git rejecting a file, a remote authentication failure, or an ordinary user denial: the reported failure is permission-state resource validation. Its implementation-level cause has not been traced.

Investigation and implementation checklist:
1. Reproduce against the actual active Git tool implementation and capture input-to-permission-facet resolution. Find the `git_add` implementation and the permission-state error literal; inspect tool permission_profile, resource kind/path construction, workspace containment, normalization and wrapper registration. Start in `crates/talos-tools` and `crates/talos-permission`, then trace runtime/CLI composition as needed. Paths are investigation leads, not verified defect locations.
2. Determine why facet 0 lacks a safe typed resource. Check whether this is missing facet resource metadata, a mismatch between Git staging resources and permission resource kinds, or validation/registration loss. These are hypotheses only. Record the actual cause and verify the active executable/build matches the source before claiming a fix.
3. Review related Git mutation tools for the same construction pattern, without expanding into unrelated refactors. Retain staging write permissions and fail-closed checks. Do not bypass the permission pipeline, disable validation, grant global permission, reset the worktree, or silently use host-shell Git. A specific shell fallback requires explicit user approval under the repository instructions.
4. Establish the appropriate governed owner/claim for the repair if needed. Changes to talos-permission or protected execution/permission boundaries require security review against escape vectors.
5. Add regression coverage for a valid workspace-relative file and this two-file staging request through the real permission wrapper. Cover approved success, explicit denial, missing/invalid typed resource, and outside-workspace/path-escape rejection as applicable to the discovered contract. Confirm errors degrade gracefully rather than panic or mutate on denial.
6. Validate that authorized git_add succeeds without relaxing unauthorized behavior. Then inspect staged and unstaged diffs separately and ensure the handoff commit contains only the two intended documents; preserve any preexisting staged changes. Follow conventional commit/model attribution rules, record the real commit SHA, push without force, and verify remote success. If commit/push encounters a separate failure, record it independently rather than assuming this fix covers it.

Acceptance: root cause is documented; focused permission/Git regression tests pass; security review is obtained where applicable; the exact authorized two-file staging operation succeeds; staged diff is reviewed; an actual handoff commit and successful push are recorded. Until then the task remains Blocked, and a receiving agent in another checkout must obtain the local documents and existing local changes separately.

## C0 preserved work — historical baseline

Earlier session todo items are marked completed locally, not committed/merged delivery:
- Tool activity animation and queued message body visibility.
- Body prefix changed to the small bullet.
- Process tool human-readable summary fields: action/job_id/cursor/max_bytes/wait_ms, plus unit assertion.
- Todo status glyphs and in-progress highlighting (display-only).
- Edit diff text uses normal text with soft red/green backgrounds. All normal/head/tail output paths use an optional space fill. Projection pads each visual row for space fills (including CJK wrapping); decorative non-space fills remain last-row-only. Original logical text is unchanged.
- Model intake/evidence-scope correction: MODEL-014 introduced and MODEL-011/MODEL-013/index/Board adjusted; no capability implementation.

Important tests: `tool_display.rs::todo_tool_status_glyphs_and_highlight_are_display_only`, `edit_tints_survive_wrapping_and_head_tail_but_not_errors`; `history_projection.rs::edit_background_fills_every_projected_row` (2/40 lines; widths 7/20/80); `app/app_tests.rs::queued_body_remains_visible_beside_pending_tool_animation`.
