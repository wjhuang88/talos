# Next Task: Session Todo Handoff

Status: Pending handoff / not activated.
Responsible recipient: To be assigned by maintainer.
Source: Maintainer requested a durable handoff and explicitly stopped further implementation in this session.
Baseline: local `main`, HEAD `63c3ab6` (merge PR #697). Recheck HEAD and both staged/unstaged diffs before resuming.
Completion evidence: None for this pending task. This document is a handoff, not an effective Collaboration Claim or implementation authorization.

## Start Here

1. Preserve the dirty worktree listed below. Review staged and unstaged changes separately; do not reset, clean, overwrite, or assume that every change belongs to this agent.
2. Read AGENTS.md, docs/sop/AGENT-COLLABORATION.md, docs/sop/START-ITERATION.md, and the scope owners below. Inventory every Active, Review, Planned and Blocked iteration from its owner before selecting work. The Board is derived, not authoritative.
3. Establish appropriate effective target-branch claims for the new deliverables. MODEL-014 is Refinement / Unclaimed and MEM-005 is Planned. Completed I212 and I298 claims do not authorize their expanded scopes. Do not invent PRs, authorization, completion SHAs or remote evidence.
4. Investigate task 7's structured Git permission resource defect before relying on those tools for subsequent work. The maintainer explicitly authorized a one-time bash Git fallback for publishing this handoff; it does not repair or authorize bypassing the tool permission gate for future work. Resolve the validation execution environment, then finish one deliverable at a time. Suggested subsequent order: validation environment and command memory parameter; validate/agile tools; custom-model resolution and regressions; compaction policy; Actor-owned compaction tool; full verification and local-change closeout. This is a recommendation, not an activated iteration plan.
5. Run focused tests, the locked workspace test suite and standard preflight. Complete owner status only with already-existing implementation SHA evidence; synchronize indexes/Board and remote Issues through the required workflow.

## Outstanding Tasks: Seven Independent Deliverables

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

## Existing Local Work: Preserve And Close Out Separately

Earlier session todo items are marked completed locally, not committed/merged delivery:
- Tool activity animation and queued message body visibility.
- Body prefix changed to the small bullet.
- Process tool human-readable summary fields: action/job_id/cursor/max_bytes/wait_ms, plus unit assertion.
- Todo status glyphs and in-progress highlighting (display-only).
- Edit diff text uses normal text with soft red/green backgrounds. All normal/head/tail output paths use an optional space fill. Projection pads each visual row for space fills (including CJK wrapping); decorative non-space fills remain last-row-only. Original logical text is unchanged.
- Model intake/evidence-scope correction: MODEL-014 introduced and MODEL-011/MODEL-013/index/Board adjusted; no capability implementation.

Important tests: `tool_display.rs::todo_tool_status_glyphs_and_highlight_are_display_only`, `edit_tints_survive_wrapping_and_head_tail_but_not_errors`; `history_projection.rs::edit_background_fills_every_projected_row` (2/40 lines; widths 7/20/80); `app/app_tests.rs::queued_body_remains_visible_beside_pending_tool_animation`.

Git status snapshot before writing this handoff:
- Added: `docs/backlog/active/MODEL-014-catalog-capability-inference.md`, `docs/tasks/2026-10-10-session-todo-recovery.md`.
- Modified: `crates/talos-agent/src/process_tool.rs`.
- Modified TUI: `src/app.rs`, `src/app/app_tests.rs`, `src/app/frame.rs`, `src/app/tool_activity.rs`, `src/history_projection.rs`, `src/scrollback.rs`, `src/state_tests.rs`, `src/tests.rs`, `src/theme.rs`, `src/tool_display.rs` under `crates/talos-tui/`.
- Modified docs: `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/backlog/active/MODEL-011-custom-model-capability-probe.md`, `docs/backlog/active/MODEL-013-catalog-context-window-inference.md`.

Structured staged diff inspection reported the two added docs as binary or unreadable; do not interpret that as proof of staging or content corruption. Reinspect staged/unstaged state with repository-supported tools before committing. No commit, branch creation, push, remote Issue update or release was performed in the visible continuation. These local files must travel with the handoff; HEAD alone does not contain them.

## Validation Evidence And Build Blocker

Pinned toolchain: Rust 1.99.0 from rust-toolchain.toml; use --locked and preserve Cargo.lock.

Passed:
- `cargo fmt --all`.
- `cargo test -p talos-tui --locked -j 1 -- --quiet`: 602 unit tests, 2 integration tests, 2 doctests.
- `cargo test --locked -p talos-config -j 1 catalog`: 19 tests, 207 filtered out.
- `cargo clippy --workspace --all-targets --locked -j 1 -- -D warnings`.

Build environment for the recorded Cargo runs:
```sh
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
RUSTFLAGS='-C link-arg=-Wl,--threads=1'
```

Failed:
- Standard `./scripts/release_preflight.sh`, job `job_40bd63ef-76cd-433a-b83a-c1351cf53a0a`, exit 101. Full logs recovered at handoff: site/installers/governance/collaboration/text/classifier/format/check and default workspace Clippy passed. Failure is in cargo test compiling CLI binary and CLI test binary, with `memory allocation of 5832720 bytes failed` / `2097152 bytes failed`, rustc SIGABRT. SDK fixture phase was not reached.
- `cargo test --workspace --locked -j 1 -- --quiet`, job `job_79e8c938-ac63-4751-8337-71d73eb5ae29`, exit 101. CLI test compilation fails with `memory allocation of 2097152 bytes failed` and SIGABRT; no workspace tests executed in this attempt.

Correction to earlier checkpoint: preflight failure stage was initially unknown because only part of the output was read. Continued cursor reads established the above test-compilation failure; use this document as the latest evidence. The jobs are terminated naturally; no live build remains.

Known fact: the source hardening default is 2 GiB and the current model-visible bash/exec schemas expose no override. Inference only: inherited address-space restriction may explain the allocation failure. Physical memory pressure versus RLIMIT/cgroup/other limits was not measured. Verify the actual execution process limits and memory pressure; do not report 2 GiB as proven root cause or weaken sandbox/permissions to get a green build. A trusted environment with sufficient address space may unblock validation before the memory-parameter feature is delivered.

## Delivery And Documentation Gates

- This handoff does not activate a product owner or complete any of the seven tasks. The twice-reproduced structured git_add error remains task 7; the maintainer authorized temporary bash Git publication of only the two handoff documents. Verify publication in Git history/remote state; fallback success is not defect resolution.
- Verify independent claims/overlap against current target branch, not just this local snapshot. Owner documents retain authority.
- Tool work must update relevant user-facing tool/command reference; custom-model work must update setup/provenance/variant docs; compaction must document status/manual/model controls and persistence; memory parameter must document units/platform limitations. Choose existing targets through docs/sop/DOC-CHECK.md.
- Run both governance validators after governance edits and standardized locked preflight before merge. Require appropriate exact-head CI and API/privacy/permission/security review for affected slices.
- Review staged diff, secrets and conventional commit requirements. Completion status needs existing implementation SHAs, never a status-only self-reference. No publication, tag or release is authorized.
- Related prior checkpoint: `docs/tasks/2026-10-10-session-todo-recovery.md`. This handoff supersedes its incomplete failure diagnosis, not the product scope owners.