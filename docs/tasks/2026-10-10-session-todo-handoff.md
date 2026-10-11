# Next Task: Session Todo Handoff

Status: Serial execution active; I299/C0 implementation merged and owner closure recorded; all seven original deliverables remain incomplete.
Responsible recipient: Codex under maintainer direction; product claims remain separate.
Source: Maintainer requested a durable handoff and explicitly stopped further implementation in this session.
Baseline: local `main`, HEAD `63c3ab6` (merge PR #697). Recheck HEAD and both staged/unstaged diffs before resuming.
Completion evidence: None for this long task. This document is a handoff, not an effective Collaboration Claim or implementation authorization.

## 2026-10-11 Receiving Checkpoint

I299's original C0 claim became effective when #698 merged at `50781eab`. The
maintainer accepted ADR-088's producer-owned, transient Todo display metadata contract;
its cross-crate scope amendment #700 merged at `2f9420f95cd2db3654de463c928debe592a89845`
after exact-head CI, independent Agent review and merge-time CAS. Preserved local C0 implementation stays on
`feat/i299-presentation`; no implementation merge or native acceptance is claimed.
All seven numbered deliverables below remain incomplete. The receiving session must
still follow their full requirements, not treat this governance checkpoint as closure.

### Current C0 checkpoint (supersedes earlier implementation-pending notes)

Completion Commit: 690926759229818fcec92b1487d0cffef1bf5fcb (C0/I299 only;
not completion evidence for this whole task or any of its seven original deliverables).

PR #701 is merged as `690926759229818fcec92b1487d0cffef1bf5fcb`; I299 is
Complete / Closed with that existing implementation SHA as evidence. Final head
`58e74613` passed all six CI jobs in run `38115866825`, independent Agent review
`6105965862`, and merge-time CAS `6106128866`. All four native observations passed.
This is the current delivery checkpoint; earlier pending statements below are
historical. No release occurred. Next is T7, with a new effective claim required
before implementation. This does not close any of the seven numbered requirements.

Latest acceptance: the maintainer completed all four native observations on `b19fc58e`,
ending with “符合预期，已退出”. I299 is Review / Claimed, ready for stable publication;
the previous manual gate is satisfied. Remaining: final-head review, remote CI/CAS/merge
and owner-first closure. The seven original deliverables remain incomplete. Earlier
pending-human statements below are preserved checkpoint history, not current blockers.

The local ADR-088 chain is now implemented from native Todo formatting through permission
wrappers, Runtime/Agent events and ConversationEngine to TUI rendering. The migration record
is `docs/reference/I299-TODO-PRESENTATION-MIGRATION.md`. Independent incremental dirty-tree
review found the two previous defects resolved (orphan truncated-item highlight and missing
real execution/export regression); this is not exact-head merge approval.

`cargo test -p talos-cli --test todo_presentation_chain --locked` passed (1 test), proving
real native Todo execution through Runtime's permission wrapper, transient UI metadata,
unchanged durable text and both export modes against literal replay. Both governance
validators passed with zero warnings. Native drawing/selection acceptance is still outstanding;
the test ends at UI output. A dedicated offline fixture is being prepared at
`crates/talos-cli/examples/i299_presentation_acceptance.rs` for that observation.
Remaining C0 gates: final complete preflight, stable commit and independent exact-head review,
native acceptance, remote CI/CAS/merge and owner-first closeout. No implementation has been
published and none of the seven original deliverables is complete.

Subsequent local validation: full standardized preflight exited 0 (`release preflight: passed`),
including workspace tests and external SDK fixtures. The final I299 acceptance example's
`--check` and targeted Clippy passed; four native human observations remain unchecked in I299.
Stable local candidate `e96438eb84066ef178a830ea4b42edb452c78d69` subsequently received independent
Agent API/privacy APPROVE against base `2f9420f95cd2db3654de463c928debe592a89845`.
The full task remains active. Next: guide the four native observations before remote publication;
stage 1 instructions have been sent, but no human result has been received. No T7 or later
product slice is activated here. Later documentation edits do not broaden that exact-head approval.

## Receiving Checkpoint And Revised Plan - 2026-10-10

Current execution: I299 claim PR #698 merged at
`50781eab529e424894d11e7b1c502fb1d0cb50f5`; implementation branch
`feat/i299-presentation` starts from that merge. Preserved code has been reapplied locally.
The rebuilt TUI suite passed 603 unit tests, 2 integration tests and 2 doctests. Todo
multiline spoofing now has regression coverage; full-list decoration still needs trusted
provenance. On 2026-10-11 the maintainer accepted ADR-088's transient, producer-owned,
exact-text/invocation-bound metadata and API migration boundary. ADR and I299 owner record
the acceptance; #700 establishes the expanded claim, while cross-crate implementation remains pending.
The implementation branch is now based on `main@2f9420f9`, including #590 Auto locale work.
Preserved TUI/process code was compared with recovery stash `4de7707f` after restoration:
no code difference. Documentation conflicts were reconciled while retaining the full queue.
The recovery stash retains the original staged/unstaged snapshot until candidate convergence.
Both governance validators passed with zero warnings for #700.
Native acceptance, full preflight and implementation
publication remain pending. All seven original deliverables remain incomplete.

This section governs the receiving session's task order. The original seven deliverables and
earlier checkpoints below remain preserved requirements and historical evidence, not current
claims that the checkout is dirty or the handoff is unpublished.

### Goal Execution Ledger - 2026-10-10

Requested outcome: close this entire handoff, including all seven original requirements and
the preserved TUI/process integration candidate. Documentation refinement is not completion.
Work mode: Standard. Deferred validation tracker: None; required manual acceptance remains a
gate until the maintainer explicitly changes the work mode.

Artifacts: this coordinator, each selected Story/iteration owner, implementation and tests,
user-facing documentation, and applicable ADR/migration records. Preserve all historical
checkpoints, raw session history, other-session branches and published acceptance.
Synchronize product owners before derived Backlog, Board and iteration indexes.

| Item | Current execution state | Required closure evidence |
|---|---|---|
| C0 preserved TUI/process candidate | Complete; I299 owner closed | #701 merge `69092675`; full preflight, all native observations, exact-head CI/review and CAS recorded in I299 |
| T7 structured Git resources/diff | Investigation; no repair committed | Real-wrapper reproduction, cause, approved/denied/escape tests, diff fidelity, protected-boundary review and delivered repair |
| T2/T3 MODEL-014 | Unactivated; consolidated implementation/test slice | Accepted authority/schema decisions, inference/override/variant/adapter tests, walkthrough, docs and merged evidence |
| T4 compaction policy/controls | Unactivated | Whole-request policy, safe summary replacement, failure/cancellation tests, observable status, runnable manual command and integrated evidence |
| T5 Actor compaction/recovery | Unactivated | Safe-boundary model-callable operation, bounded authoritative history recovery, discovery/concurrency/privacy tests and merged evidence |
| T1 validate/agile tools | Unactivated | Shared services, supported read-only agile summary, gated bounded validation execution, discovery/failure tests and docs |
| T6 memory parameter | Unactivated | Retained default, explicit units/range/inheritance, foreground/background/composition coverage, platform truth and security review |

Execution environment: pinned Rust 1.99.0; receiving workspace has no `target/` cache and
approximately 14 GiB available at validation start. Standard preflight started with
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
These resource settings do not omit any preflight phase or weaken process permissions.
Site and installer checks passed; remaining phases have no result yet.

External authorization: the maintainer accepted the consolidated contract on 2026-10-10:
serial implementation, stable-candidate commits/pushes, necessary claim/implementation PRs,
independent subagent review and gated merges. Human acceptance and new ADR decisions return
to the maintainer. Release, deletion of other-session branches, paid probing and security
waivers are excluded. No new product implementation is committed before an effective claim.

Branch/checkpoint plan: preserve the existing four commits unchanged; publish them through one
reviewable recovery candidate rather than directly pushing local main. New governed slices start
from their effective claim merge. Use this checkout serially; no extra worktree is required.
Append actual commands, heads, reviews and closure evidence before moving to each next phase.
Resource policy: single-concurrency, debug-free local builds; monitor free space, stop before
disk exhaustion and seek approval for destructive cache cleanup. No paid or sensitive live
provider test. Retry a failed gate only after inspecting its cause; never waive it for progress.
Fallback: keep the affected item non-terminal with its owner and diagnosis; independent work may
proceed only when its dependencies, overlap checks and explicit scheduling authorization hold.

Initial T7 source finding: `GitAddTool` uses the default permission profile, which constructs an
untyped Write facet; `compile_proposal` requires a supported typed resource for consequential
facets. This explains a plausible rejection path but is not yet a verified real-wrapper
reproduction or a complete assessment of related Git tools/pathspec behavior.

Read-only T7 follow-up on `e96438eb` (2026-10-11; no activation or code change):

- `crates/talos-tools/src/git_write.rs` confirms `GitAddTool` has no permission-profile
  override; the default in `talos-core` supplies Write without resource kind/value.
  `talos-permission/src/grant.rs` rejects consequential untyped facets when compiling a grant
  proposal. This matches the recorded error, but active-binary identity and real-wrapper
  reproduction remain mandatory before claiming the production root cause verified.
- The same Git add implementation appends caller path strings directly after `git add`,
  without an option terminator or literal-pathspec boundary. A repair must bind the actual
  staging effect to approved resources, including option-looking names, Git pathspec syntax,
  repository/workspace root differences, symlinks and linked worktree index locations.
  Do not merely mark every request as a workspace-wide write.
- `GitDiffTool` enumerates index/worktree status for both modes, while reading HEAD as the
  old side for both. The existing `git_diff_staged_shows_head_vs_index` regression explicitly
  adds a second unstaged edit so the iterator sees the file. It therefore does not prove the
  staged-only case. Add independent staged-only, unstaged-only, mixed, new/deleted and binary
  fixtures; compare staged HEAD/index and default index/worktree semantics separately.
- These are source facts and test-coverage findings, not a successful execution reproduction
  or a delivered fix. Retain the original scope, permission gates and independent security
  review; do not mix this implementation into I299. I299 still awaits four human observations.

Recovery: refresh Git/PR state, inspect the live preflight process before restarting any build,
record its actual exit result, then resolve C0 ownership and publication gates. Residuals stay
in their original requirement row; none may be silently dropped to close this coordinator.

### Integration And Validation Truth

- Maintainer requested integrating the preservation branch and reorganizing the handoff.
  Local `main` fast-forwarded from `e58e2678` to `385a2d6a`; no remote-main push or
  validated remote merge is claimed. The four preserved commits remain unchanged.
- Existing TUI/process-tool code is an integration candidate, not completed delivery.
  Prior focused tests and Clippy passed, but prior full workspace validation failed during
  CLI compilation. Exact-head remote CI/review and applicable ownership reconciliation remain
  required before publishing the integrated code to remote main.
- The receiving shell reports unlimited virtual-address-space limit and approximately 19 GiB
  free disk. This does not establish the earlier allocation failure's cause. Do not weaken
  process hardening or require the memory-parameter feature merely to run validation.
- Receiving preflight passed site, installer, SQLite-consumer, workflow and project-governance
  checks before being deliberately stopped for this documentation-only refinement. It is NOT
  a completed workspace preflight or code-validation result.
- The structured Git permission defect and staged-diff discrepancy remain unresolved. Earlier
  host-Git publication succeeded; publication recovery is no longer a pending deliverable.
  Repair the actual Talos wrapper, not this receiving session's unrelated development tools.

### Research Findings That Change The Plan

1. **LLM compaction already runs in production paths.** `session.rs` calls
   `Compactor::compact(..., self.agent.provider())` in both runtime-managed and compatibility
   submission paths. MEM-003's statement that layers 4/5 are unwired is stale. Preserve its
   original acceptance, append corrected current facts, and verify missing integration evidence
   rather than implementing a second provider wiring path or declaring MEM-003 Complete.
2. **Current trigger is not whole-request pressure.** `should_compact` estimates history against
   80% of model context; prompt/tool definitions and request output reservation belong to the
   separate request-admission calculation. I298 fixes request estimates and bounded overflow
   recovery, not semantic checkpoint policy. Reuse its admission authority; do not add another
   token-budget authority or reopen completed I298 scope.
3. **Summary failure handling needs verification before expansion.** The current summary reader
   collects text deltas, does not explicitly fail on streamed error events, and substitutes
   `[No summary generated]` when empty. Test error-only, partial-text-then-error, cancellation,
   oversized output and failed terminal outcome. An invalid summary must not replace usable
   history; provider failures must remain observable. This is a source-inspection risk, not a
   reproduced production data-loss incident.
4. **Manual API is not a user command.** `manual_compact` exists with unit tests, but inspection
   found no registered `/compact` command or Actor-owned model-callable compaction operation.
5. **Existing summaries lack the target semantic contract.** Layers 4/5 use an unstructured
   summary projected as `Message::User`. Issue #390 proposes structured model-directed
   checkpoints and bounded current-session history recovery. Coordinate provenance/summary
   representation with SESSION-011/#612 rather than silently changing transcript semantics.
6. **Pre-entry compression is separate work.** MEM-007 has default-off deterministic Bash
   compression with raw UI/export evidence. Other tool strategies, cross-turn dedup and
   representative measurements remain open; #143 owns related Bash/PowerShell filtering.
   Do not bundle those expansions into compaction-policy completion.
7. **Chat thinking transport is proven for one test.** A non-sensitive arithmetic request to
   the currently configured Codex custom endpoint using `gpt-6-astra`, Chat Completions and
   `reasoning_effort=low` returned HTTP 200 with nonempty `delta.reasoning_content` (20 characters)
   and reported 13 reasoning tokens. Talos parses this field. Lack of Responses support alone
   does not explain absent thinking. Reproduce through actual Talos request/config/UI paths;
   this observation neither verifies every gateway capability nor authorizes active probing.

### Consolidated Serial Queue

| Order | Deliverable / existing ownership | Original tasks | Exit evidence |
|---|---|---|---|
| 0 | Close out the preserved TUI/process integration candidate | Existing local work | Reconcile owner/authorization, review diff, reproduce focused tests, complete locked preflight and obtain exact-head CI/review before remote-main delivery; retain a separate rollback path |
| 1 | Repair structured Git permission resources and inspect diff fidelity | 7 | Actual wrapper reproduction, root cause, approved/denied/escape regression tests and independent security review; do not repeat already-completed handoff publication |
| 2 | Custom-model metadata, variant and thinking-path consistency / MODEL-014 | 2 + 3 | Accepted capability precedence; explicit false overrides preserved; unique/ambiguous/unknown matching tests; protocol-compatible output/reasoning/image behavior, provenance display and real Talos walkthrough |
| 3 | Reliable compaction policy and controls / MEM-005, MEM-003, #390 | 4 | Whole-request budget integration; valid-summary-only atomic replacement; cancellation/fallback; observable before/after and skip/failure; typed `/compact`; long-session integration evidence with recoverable raw history |
| 4 | Actor-owned model-directed checkpoint and history recovery / #390 | 5 | Bounded schema, session/branch references, safe-boundary application, pressure eligibility, tool discovery, cancellation/concurrency tests and bounded private-history recovery; no second transcript store |
| 5 | Model-callable validation and read-only agile summary | 1 | Shared business service, typed schemas, permission-gated host execution with deadlines/cancellation/output limits, failure tests and tool documentation; do not imply unsupported agile mutations |
| 6 | Explicit subprocess memory-limit parameter | 6 | Units/range/default/inheritance specified; 2 GiB default retained; all supported foreground/background paths tested; honest platform enforcement and independent process-security review |

Tasks 2/3 from the original handoff form one implementation-and-regression slice, not two PR
loops. Compaction stages 3/4 share the MEM-005/#390 roadmap; separate candidates only when
they are independently runnable and claims/acceptance warrant it. Local corrections, tests and
documentation updates stay within their candidate. Do not create an Issue per checklist item.

### Compaction Acceptance And Scope Boundaries

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

### Existing Iteration Disposition

Receiving inventory of explicit non-terminal iteration headers and open PRs:

| Owner | Observed state | Disposition |
|---|---|---|
| I291 / #682 | Review; open Auto locale PR | Other-session work; retain, do not modify or merge under this handoff |
| I293 / #696 | Active; open Search router PR | Other-session work; retain and recheck overlap before activation |
| I294 | Active; native Search evidence | Retain external ownership; no takeover; historical #693 wording may need its owner's refresh |
| I290 | Review; locale baseline | Retain published evidence and follow-up relation to I291; no duplicate locale implementation |
| I277 | Review; remaining human acceptance Deferred | Keep deferred device/accessibility acceptance with its existing owner |
| I249 | Planned / Unclaimed | Keep dependency pilot deferred; no dependency upgrade in this queue |
| I296 / I297 / I298 | Terminal owners; claims closed | Preserve completion evidence; never reuse closed claims for these expanded tasks |

No new iteration or product Collaboration Claim is activated by this plan. Before each selected
slice, refresh main, all owner statuses and open PRs; record any additional non-terminal owner
found by full activation inventory and establish the required effective claim. This task document
coordinates the queue; Story owners remain authoritative for product scope and completion.

### Next Action And Documentation Closure

First finish the preserved code candidate's validation/ownership path. Then prepare the bounded
Git defect scope, followed by MODEL-014 and the unified MEM-005/#390 sequence above. Decisions
about capability authority, checkpoint representation and public API changes must be recorded
before implementation. No release, tag, Issue closure or remote-main direct push is authorized
by a planning checkpoint.

This refinement changes only the coordination document. Product owner statuses, historical
acceptance and completion evidence remain unchanged. Run documentation/governance validators
and diff checks for this edit, not a Rust rebuild. Remaining code-validation and claim gates
are registered above rather than hidden behind a Complete status.

## Latest Preservation Checkpoint

The maintainer subsequently requested committing and pushing all remaining worktree changes. The receiving agent should use branch `handoff/session-worktree-20261010`, not only `main`, to recover the complete work. The earlier dirty-worktree statements below are historical snapshots.

- `e58e2678`: handoff and recovery documents; successfully pushed to `origin/main`.
- `0a0622c0`: existing TUI activity, queue and result-rendering changes plus regressions.
- `3af78f04`: process-tool summary fields and regression assertion.
- `1bda5181`: MODEL-014 intake and related owner/index references.

The last three commits are preserved on the handoff branch; this checkpoint accompanies their push. Verify remote branch state before resuming. No merge into main, feature activation, claim repair, or completion of tasks 1-7 is implied. Full workspace validation remains blocked by the previously recorded CLI compilation allocation failure. Both governance validators were rerun successfully, and the process-tool filtered agent suite passed 5 tests. The structured Git mutation defect remains unresolved; staging/commit/push used the maintainer-authorized host Git fallback. Read-only structured staged-diff output also disagreed with host Git during this operation (reporting the untracked MODEL-014 file instead of the staged code); inspect that separately when investigating task 7.

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

## Resumed Verification Checkpoint

- Full standardized preflight passed on local candidate
  `0b396c48865b74e4cf3e7d0f711ccb208f264afd` (2026-10-10):
  `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 ./scripts/release_preflight.sh`.
  This includes workspace check, Clippy, tests/doctests, both governance validators
  with zero warnings, and the independent Runtime SDK fixture in default and coding configurations.
  The process exited 0; no build remains running. Earlier allocation failures did
  not recur, but their root cause remains unproven.
- Independent preserved-work review found a display-only Todo defect:
  `tool_display.rs` recognizes status rows by textual prefixes, so a multiline
  description containing `  [~] literal checklist` can be incorrectly rewritten
  and highlighted. C0 must repair contextual recognition and cover multiline
  titles/descriptions, including head/tail truncation, before delivery.
  This review is not merge approval; native acceptance remains outstanding.
- Follow-up source inspection confirms both `item.title` and `description` are
  interpolated without newline escaping in `talos-session/src/todo/formatting.rs`.
  A suffix/UUID regex alone cannot establish row provenance: user text can mimic
  that suffix or even a list header. The fix must preserve the raw transcript and
  avoid interpreting ambiguous user-authored continuation text as status metadata.
  Regression coverage must include forged list headers and UUID-shaped suffixes,
  not only ordinary multiline descriptions; evaluate producer-provided display
  provenance versus conservative rendering before choosing the implementation.
- Autonomous serial implementation and gated publication are authorized. On
  2026-10-10 the maintainer explicitly confirmed non-overlapping parallel work
  with other sessions' Active I293/I294. This queue remains internally serial;
  other-session ownership and branches remain untouched. Each new slice still
  requires its effective target-branch claim before committed implementation.
- This checkpoint records verification only. It does not complete C0 or any of
  the seven requirements, establish an effective claim, or authorize a release.

## Delivery And Documentation Gates

- This handoff does not activate a product owner or complete any of the seven tasks. The twice-reproduced structured git_add error remains task 7; the maintainer authorized temporary bash Git publication of only the two handoff documents. Verify publication in Git history/remote state; fallback success is not defect resolution.
- Verify independent claims/overlap against current target branch, not just this local snapshot. Owner documents retain authority.
- Tool work must update relevant user-facing tool/command reference; custom-model work must update setup/provenance/variant docs; compaction must document status/manual/model controls and persistence; memory parameter must document units/platform limitations. Choose existing targets through docs/sop/DOC-CHECK.md.
- Run both governance validators after governance edits and standardized locked preflight before merge. Require appropriate exact-head CI and API/privacy/permission/security review for affected slices.
- Review staged diff, secrets and conventional commit requirements. Completion status needs existing implementation SHAs, never a status-only self-reference. No publication, tag or release is authorized.
- Related prior checkpoint: `docs/tasks/2026-10-10-session-todo-recovery.md`. This handoff supersedes its incomplete failure diagnosis, not the product scope owners.
