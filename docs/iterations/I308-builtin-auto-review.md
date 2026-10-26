# Iteration I308: Shared Auto Review For All Built-In Tools

## Current delivery checkpoint — 2026-10-26

Foundation #769 merged as `d45d08f0a82706c2abb26fcb32899f8b24fa64b1`.
Exact head `ffa35973022c18f752cacad1922325e15c648379` passed all six jobs in
CI 40274739232, including native Windows. Independent permission/security/API
review and merge-time CAS are recorded in
[#769 review receipt](https://github.com/wjhuang88/talos/pull/769#issuecomment-6409815251).
The review was by a separate read-only Agent sharing the session/GitHub identity,
not a separate natural person. It did not independently run tests.

I308/H/G/T8 remain Active and incomplete. Mutation #770 merged as
`687b84a93b4a9f40ede1eb600151c0a22a8f2d59`; policy/read coverage commits `82667ca54` and `07c005b87` are published in
unmerged #771. Ordinary effectful Execute/HTTP/WebSearch Auto, remaining registered
operation/profile dispositions and final delivery gates remain outstanding.
Earlier checkpoints below preserve historical states and do not override this one.

> Document status: Active (#765 merged; effective at `8f0ce2340d7728ac5b3ee8eba814050a2360e194`)
> Published plan date: 2026-10-26
> Planned objective: complete handoff T8 with model-assisted, invocation-local approval across registered native tools, including suitable Write, Network and Execute operations and higher-risk file changes.
> Baseline rule: preserve this published target; implementation stages do not reduce acceptance.
> MVP deliverable: a production native tool request needing approval can receive one model AllowOnce and actually execute through normal admission; every registered native operation has a tested disposition.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | I308 / PERM-007-H and PERM-007-G: native-tool Auto evidence, shared assessment and invocation-local admission, higher-risk file recovery, production registration coverage, tests and documentation; excludes search transport and provider protocol implementation. |
| Claimed At | 2026-10-26 |
| Source Issue | #563; handoff T8 |
| Governance Claim PR | #765 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized continued full handoff delivery; #765 requires independent permission/security/API review, exact-head CI, both validators and final merge-time CAS before merge. |
| Implementation PR | #769 (foundation); #770 (stacked file mutation stage) |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Satisfy full H/G acceptance with existing implementation commits, independent permission/security/API review and platform evidence; no release authorized. |

This governance candidate proposes claim and activation atomically in #765. Both are ineffective
until the finalized exact-head record merges to main; implementation starts from that merge or later.
No production implementation belongs in this governance candidate.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-H | PERM-007 | Refinement / Unclaimed; acceptance explicit and ready for selection | I300 completed at `ee84889fd`; existing shared evaluator and permission pipeline | All registered native operations have tested, effect-aware dispositions and suitable requests can actually execute after model AllowOnce. |
| PERM-007-G | PERM-007 / #563 | Refinement / Unclaimed; selected together with H | Shared typed evidence and atomic admission | Existing-file edits and destructive/high-impact native file operations have bounded diff/impact evidence, real backup/recovery and audit behavior before Auto allowance. |

### Scope And Policy Reconciliation

- Inventory actual production assembly in CLI interactive/TUI/MCP, Runtime and Desktop, including feature-gated registrations, Todo, scheduler, status, process and conditional skill tools. Distinguish dormant constructors and externally supplied overrides.
- Reuse one shared model assessment path with implementation-owned typed, bounded evidence bound to exact normalized execution arguments, current user intent, resources and freshness. Presentation/redacted arguments are not authority.
- Implement actual one-shot approval for suitable native Write, Network and Execute requests. Category-wide refusal, tool-name allowlists, advisory-only responses and unconditional human fallback do not satisfy H.
- Preserve deterministic policy-allowed read-only fast paths with no additional model call. Multi-facet approval is atomic; partial approval cannot execute.
- Preserve explicit Deny, explicit mandatory-human user policy, execution-unknown replay protection, admission identity checks, cancellation, evaluator isolation, bounded output and secret redaction.
- H and G share this one iteration/claim. G's historical W2/W3 human gate remains until the relevant implementation is reviewed and validated; ADR-064's accepted 2026-10-22 amendment permits suitable effectful model approval. Do not silently remove G's diff, impact, backup/recovery or audit requirements.
- Cover higher-risk operations exposed by existing native schemas: existing-source edits, delete and any existing rename/batch behavior, dependency/config/release/security files. Record inventory evidence where a proposed operation has no native tool; do not invent new tools to satisfy an inventory label.
- Implement bounded backup/recovery for native file mutations, with failure/cancel and stale-target behavior. Describe actual recovery limits; never promise transactional rollback for arbitrary shell, Git, network or scheduled agent work. If a concrete effect cannot be safely assessed/enforced, expose its precise human decision and retain incomplete coverage honestly.
- Unknown/unbound effects, insufficient intent/evidence, uncertainty, assessor failure and unsupported enforcement give actionable human/headless fallback. Model denial cannot execute; cancellation cannot leak an AllowOnce.
- Trusted implementation ownership must survive production composition and wrappers without name spoofing: custom RuntimeBuilder tools and plugins do not become trusted native implementations through matching names, default provenance or diagnostic contribution source.
- Retain I300's reviewed whole-command Pull semantics, one-shot credentials and process supervision; integrate through shared assessment without reopening per-path effect prediction.
- Public API adaptations must preserve embedder compatibility or obtain the required ADR and migration decision before implementation; no silent exhaustive-enum or struct-literal break.

### Non-Goals

- Automatic authorization of external MCP/plugin/embedder tools merely because they resemble native tools.
- New native tool families, search backend activation/transport, provider protocol work, universal rollback or permanent grants.
- Dependency upgrades, new unsafe code, release/tag/publication or waiving any remaining handoff task.

### Acceptance

1. Given each production profile and applicable feature set, registration coverage identifies every native operation and a tested policy/model/human disposition; an undeclared new built-in fails the invariant. External same-name replacements cannot inherit native approval.
2. Given a suitable native request needing permission, the shared assessor receives bound intent/effect evidence and model AllowOnce leads to actual production execution; representative non-shell Write, Network and Execute operations prove this end to end.
3. Given multi-facet effects, all are admitted together or none execute. Existing deterministic read-only allows incur zero model calls.
4. Given explicit Deny/human policy, uncertainty, error, cancellation, changed input/resource/evidence or replay, no unauthorized execution occurs and the user receives a truthful reason/fallback. Verify allow, deny, uncertain, error, cancel and stale outcomes through the production pipeline.
5. Given higher-risk native file changes, bounded diff/impact evidence and actual backup/recovery behavior are tested before Auto execution, including special/config/security paths, destructive operations, partial failure and stale identity. No prose-only rollback claim satisfies G.
6. Given CLI/Runtime/Desktop/MCP-hosted native composition and applicable hosts, intended behavior is reachable with visible model decision/rationale and safe secret handling; missing platform evidence stays outstanding.
7. Auto user docs, API/migration notes and coverage inventory match implemented behavior. Independent permission/security/API review, full locked preflight, exact-head CI and merge CAS pass before completion.

### Execution Stages

1. Inventory and contract: complete operation/profile matrix, trusted evidence ownership and compatible API design; registry invariant and focused tests.
2. Shared admission: unify assessment and atomic single-use admission; implement representative non-shell production allow/deny/error/cancel/stale paths.
3. Full native coverage: finish all registered families and G diff/impact/backup/recovery behavior; preserve existing policy fast paths.
4. Integration and closure: real host/feature coverage, user-visible diagnostics, documentation, independent review and final exact-head gates.

Stages are logical commits/reviewable candidates, not substitutes for the complete acceptance above.
Converge each stage locally before pushing; record remaining acceptance explicitly after any stage merge.
Consult relevant existing implementations and upstream best practices when a design stalls; record the
source and applicable limits instead of extending speculative local machinery.

### Planned Validation

- Repository-pinned Rust 1.99.0; all workspace checks/tests/builds use `--locked`.
- Focused production registration/permission/assessment/execution tests, including external overrides and all decision outcomes.
- Profile/feature coverage checks for native assemblies, deterministic read-only no-model tests and multi-facet denial tests.
- Higher-risk write diff/impact/backup/recovery tests with failure, cancellation and stale targets.
- `./scripts/release_preflight.sh` (full workspace checks, Clippy, tests and SDK consumers).
- Windows compile through `cargo xwin` with locked dependencies; native Windows CI execution is a separate mandatory evidence row, not inferred from cross-compilation. Native Unix tests cover applicable behavior.
- `scripts/validate_project_governance.sh .` and `bash scripts/validate_collaboration_claims.sh .`.
- Independent exact-head permission/security/API review, CI and final merge-time CAS.

### Documentation To Update

- `README.md` Auto behavior/configuration sections and SDK examples affected by the final API.
- Tool behavior/API migration notes, ADR-064/070/089 where affected, and a complete production operation coverage matrix in this owner or a linked reference.
- H/G owners, #563, handoff ledger, backlog, iteration index and Board; completion requires already-existing implementation SHA evidence.

### Risks And Rollback

- Hard: write-capable effects remain permission-gated; explicit Deny/human policy and immutable safety constraints cannot be overridden.
- Soft: accepted operation-level model review replaces historical category-level eligibility limits only after concrete implementation and independent review.
- Assumption to validate: a compatible trusted evidence interface can serve all actual native assemblies without accepting embedder-provided claims as native authority.
- Risks: stale evidence, native-name spoofing, partial multi-facet admission, sensitive diff leakage, unsafe recovery and implicit scheduled work. Each needs focused adversarial tests and independent review.
- Rollback: revert a reviewed implementation stage to prior runnable behavior; do not auto-reset user data or imply external effects can be undone. Runtime backup recovery must be implemented and separately tested, not inferred from Git revert.

## Selection Inventory And Disposition — 2026-10-26

| Owner | Current state / disposition |
|---|---|
| I293 SEARCH-001-C | Active; retain its transport/endpoint-authority ownership. I308 assesses registered native requests without changing search routing/transport contracts. |
| I294 SEARCH-001-D | Active; retain provider research/evidence, no overlapping implementation selected. |
| I306 PROVIDER-008-A | Active, open implementation #749; retain provider protocol ownership, no protocol changes in I308. |
| I302 TEST-003 / #717 | Review; retain unexplained Windows resource residual and #705 diagnostics; not absorbed or waived. |
| I277 Desktop validation | Review, human/device rows deferred and nonblocking; retain original acceptance. |
| I249 dependency pilot | Planned; explicitly defer unchanged behind the selected handoff queue. |
| I164 | Paused; preserve supersession/layout disposition, do not reactivate. |
| I146/I147/I150/I162/I262 | Current owner headers Complete; historical Review lines/stale index do not reopen them. |
| I300 / I307 | Owners are Complete; T7 and its sandbox prerequisite have merged evidence, not activation blockers. |

Existing parallel owners remain separately scoped; the maintainer's continued handoff execution
selects I308 as this session's only new implementation stream. Open PR inspection found #749 and
#705, neither claims H/G. Recheck target owner/PR overlap and dependencies immediately before merge.
Handoff ordering after T8 remains T2/T3 → T4 → T5 → T1 → T6, all incomplete.

## Actual Activation And Execution

- 2026-10-26: governance draft only; source Issue #563 and its comments reviewed. No effective claim, implementation or activation yet.

## Verification Evidence

- Draft baseline: both local governance validators passed with zero warnings. Finalized candidate rerun, independent review, exact-head CI and merge CAS remain required.

## Completion Evidence

No implementation completion evidence exists. Do not mark H, G or T8 Complete from this plan.

## Variance And Residuals

- Full production operation matrix and compatible API design are stage 1 implementation deliverables, not claimed complete by discovery notes.
- G remains human-gated until implemented/validated; no higher-risk policy implementation is claimed here.
- Remaining handoff tasks and gix capability/advisory tracking retain separate owners.

- 2026-10-26: #765 number backfilled; proposed Claimed/Active record and H/G owner synchronization are ineffective until main merge.

### Local execution checkpoint — 2026-10-26

- Activation #765 is merged; implementation branch `feat/i308-builtin-auto-review`
  starts from `8f0ce2340d7728ac5b3ee8eba814050a2360e194`.
- Added a compatible default `ApprovalResolver::resolve_network_invocation` entry point.
  The pipeline now passes the retained transport manifest separately from exact execution
  input and human presentation. Existing adapters delegate to their previous exact-input
  resolution method. The manifest does not establish native implementation ownership.
- Focused validation: `cargo test --locked -p talos-agent --lib
  permission_pipeline::execution_input_tests` passed all four tests, including network
  evidence delivery, denial without grants and legacy adapter compatibility.
- This is unmerged admission plumbing, not model-based network allowance. Shared typed
  assessment, complete native ownership coverage, actual Auto execution and G
  recovery remain unimplemented; full preflight and independent review remain outstanding.

- Ownership foundation: added explicit built-in instance registration, with same-name
  ordinary replacement revoking the marker and duplicate batches remaining atomic.
  Product-owned CLI groups and Runtime coding-preset/skill contributions now use it;
  caller overrides and plugin registration retain ordinary APIs. Direct scheduler,
  process and other registrations still need coverage; no complete inventory claim.
- Core ownership tests passed 3/3. Runtime `shared-composition` locked check passed
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`. An earlier default-debug
  check exhausted disk space and is not a code-validation success. Previously authorized
  `target/debug/deps` and incremental cleanup recovered space; no source/Git data removed.
- CLI production registry ownership assertions are in a separate test module.
  Ownership is not yet consumed for model approval; implementing
  the common assessment/admission route remains the next step.

- CLI production registry ownership test passed (1/1) across print/TUI/MCP assemblies.
  Formatting and both governance validators passed. Ownership foundation is recorded
  in local implementation commit `d2d31b9b6`; it has not been pushed or independently
  reviewed. No stage merge, full preflight, native Windows run or T8 completion claimed.

- Network review implementation checkpoint: dispatch now conveys registered built-in
  ownership with retained network evidence to a dedicated model assessment route.
  The response must match the fresh invocation digest and closed schema; only
  high-confidence AllowOnce can admit the request. Explicit human/Deny policy,
  stale state, replay, provider failure and external instances remain covered by
  focused tests. Serialization stops at 16 KiB, with an input check before cloning.
  The initial human-policy test incorrectly appended its rule after the default
  Ask; it now constructs an explicit-rule engine so that the intended rule matches.
- Validation: locked `talos-agent --lib auto_resolver` tests passed 71/71;
  log `/private/tmp/i308-auto-resolver-tests.log`. This includes encoded-size
  boundary coverage and 15 network-admission scenarios. These are pipeline-level
  results, not production tool execution evidence. The network changes remain
  uncommitted; complete native registration coverage, production end-to-end
  execution, G recovery, full preflight, Windows and independent review remain open.
- Agent-loop dispatch test passed for five scenarios: model-approved built-in
  executes with live network authority, external same-name registration and ordinary
  replacement do not inherit trust, explicit Deny blocks execution, and deterministic
  Allow executes without model assessment. This uses an offline prepared transport
  fixture, not the WebSearch HTTP implementation. Actual product transport evidence
  remains outstanding. Log: `/private/tmp/i308-auto-network-dispatch.log`.
- Independent working-tree review found JSON credential fields could bypass the
  shell-oriented secret heuristic. Added bounded structural key/value inspection
  and consistent text scanning for intent and serialized assessment fields.
  Six credential scenarios (keys, nested objects, arrays, prefixes, intent and
  endpoint paths) assert zero assessor calls. Auto resolver tests remain 71/71.
- Standard preflight initially failed in two Git process integration tests under
  sandboxing (process observation EPERM; configured hook start timeout). Both
  actual target tests passed outside the sandbox with `--features git` (1/1 each).
  An earlier rerun without that feature selected zero tests and is not validation.
  Full standard preflight is rerunning outside the sandbox; current process 64384,
  log `/private/tmp/i308-network-stage-preflight-unsandboxed.log`. No full pass
  or stage readiness is claimed until its terminal result is inspected.
- Terminal validation: the sandbox-external full standard preflight exited 0,
  including workspace check, Clippy, tests, governance and the independent Runtime
  SDK consumer (`release preflight: passed` in the log above). Independent working-tree
  security/API follow-up found the credential fixes resolved its blockers. This
  is local stage evidence, not final exact-head CI/Windows evidence or T8 completion.
  Fetch confirmed origin/main remains `8f0ce2340`; no rebase was needed.
- Production ownership follow-up: interactive mode now explicitly registers its locally
  constructed scheduler instances as built-ins, and MCP registers its concrete `status`
  instance as a built-in. Generic registry builders still treat caller-supplied scheduler
  instances as external; a regression test verifies they cannot inherit built-in ownership.
  Locked CLI `builtin_registration_tests` passed 2/2. The earlier filter
  `registry_builtin_tests` selected zero tests and is not evidence. These changes are
  local and unreviewed; they do not enable default HTTP/WebSearch Auto execution or
  implement G recovery. Complete production inventory, common admission, final platform
  validation and independent review remain open.
- Ordinary invocation evidence plumbing: dispatch now conveys exact arguments, complete
  evaluated facets, implementation description and optional implementation-owned working
  directory for explicitly registered ordinary built-ins. Prepared network, prepared Git
  and shell routes retain precedence. The additive resolver default delegates to the
  existing execution-context/exact-input methods; external adapters cannot construct the
  evidence, and no ordinary-tool model AllowOnce implementation is enabled by this change.
- Locked `talos-agent --lib permission_pipeline::execution_input_tests` passed 4/4,
  including registered evidence delivery and legacy-adapter delegation. The Auto resolver
  suite passed 71/71 before the private context-enum cleanup; focused input tests passed
  again after cleanup. Independent working-tree review found no blocker in ownership,
  routing or API compatibility; this is not final exact-head delivery approval.
- Registration follow-up is committed locally as `06aa93f3e`; no push/merge claimed.
  Both governance validators passed with zero warnings before the ordinary evidence
  extension. Default HTTP/WebSearch model execution and G backup/recovery remain
  unimplemented, so H/G/T8 remain Active and incomplete.

- Latest local checks: locked agent Clippy passed with warnings denied after the context-enum cleanup; registration/input tests and independent incremental review are stage evidence only. Ordinary evidence plumbing is committed as `a44cba863`. Fetch reconfirmed unchanged main `8f0ce2340`; no rebase required.

- HTTP configuration stability prerequisite: per-request timeout now uses the retained
  client's request builder instead of constructing a client that resets redirect policy
  and rereads proxy environment. Separate regression tests verify timeout clamping and
  rejection of a redirect under a zero redirect budget. Locked `talos-tools --features
  network --lib http_request::` passed 26/26. The initial no-feature invocation selected
  zero tests and is not validation evidence; an initial regression assertion incorrectly
  expected a 302 rather than reqwest's zero-budget redirect error and was corrected.
  This does not implement HTTP Auto admission, G recovery or final delivery gates.

- HTTP bounded-response prerequisite: response collection now stops after the configured
  body limit plus one byte, rather than downloading the complete body before truncation.
  Output reports a lower bound instead of claiming an unknown total. Separate transport
  tests cover empty, exact-limit, over-limit and zero-limit bodies plus a chunked server
  that never completes its stream. This bounds retained application body bytes, not
  reqwest's internal per-chunk/decompression buffers. Locked `talos-tools --features
  network --lib http_request::` passed 29/29 after adding the independent reviewer's
  suggested incomplete-body error test. Static incremental review found no blocker;
  both governance validators passed with zero warnings before this test-only follow-up.
  This is not final exact-head delivery approval. HTTP Auto admission remains
  unimplemented. A maintainer question is pending on whole-operation network review
  versus per-destination isolation: ADR-064 does not currently authorize unknown/unbound
  targets, while ADR-090's prepared route requires every destination to be declared.
  No scope expansion is implemented or assumed while that decision is pending.

- File recovery implementation checkpoint: `edit` now accepts opt-in `retain_backup`
  for legacy and anchored edits and returns a real recovery ID. A separate `edit`
  call with path/ID restores original contents through current Write authorization,
  verifies the post-edit digest and consumes the ID on successful restoration.
  The instance-local memory store expires entries after 15 minutes, caps originals at
  2 MiB/file, 16 MiB total and 64 records, and refuses capacity before mutation without
  evicting live entries. README describes actual use and limits. Mutation errors retain
  recovery IDs; post-mutation snapshot errors explicitly report the changed state.
- Locked file-tools tests passed 76/76 and locked file-write library Clippy passed with
  warnings denied before the additional concurrent-restore test. Independent security/API
  review and that final test rerun are pending. This is uncommitted implementation,
  not complete G: no durable/delete/directory recovery, full OS identity/CAS protection
  against external writers, higher-risk Auto admission, complete impact evidence or
  cancellation/platform validation is claimed. H/G/T8 remain Active and incomplete.
- Independent incremental review requested changes: recoverable and ordinary anchored
  edits used separate locks, and a post-rename restore error omitted its potentially
  completed effect. The working implementation now shares the registry path lock for
  anchored edits/backups/restores and reports uncertain restore completion with retained
  ID. Prior tests passed 77/77 including single-consumption concurrent restore; latest
  fixes still require regression coverage for mixed edit modes and injected post-rename
  failure plus reviewer follow-up. Do not treat the recovery stage as review-approved.
- Recovery follow-up: locked file-tools tests passed 80/80, including injected
  post-rename edit and restore failures, concurrent single-consumption recovery and
  mixed recoverable/ordinary atomic writes sharing a revision lock. Independent
  incremental security/API re-review withdrew both blockers and found no new concrete
  blocker; it was static review, not final exact-head approval. Legacy non-backup
  string writes retain their existing non-atomic behavior and do not join this lock.
  Both governance validators passed before this checkpoint-only update. The stage
  remains local; complete H/G acceptance and final platform/preflight/CI gates remain.

### Production inventory checkpoint

`registry_builtin_tests::product_registries_have_no_uninventoried_native_operations`
compares the complete Print/TUI/MCP registries against an explicit inventory and checks
instance ownership for every entry. New registrations fail this test until inventoried.
This is a coverage assertion, never a runtime tool-name allowlist or proof of Auto support.
Runtime shared composition has a separate Product/Runtime inventory-equivalence test.

| Registered operations | Current I308 disposition / remaining evidence |
|---|---|
| `read`, `ls`, `read_image`, `document_extract`, `grep`, `glob`, `diff`, `stat`, `tree`, `find_symbol`, `find_references`, `list_symbols`, `list_imports` | Real Agent dispatch tests cover default Read Allow and explicit Read Deny for all listed operations, with actual results and zero model assessments/grants. Image attachment reaches only the next provider request after an allowed read. Complete profile/platform coverage remains outstanding. MCP omits `read_image`. |
| `write`, `edit`, `delete` | Exact-content atomic creation is merged in #769; complete-image review and opt-in real recovery for string/anchored edit and text deletion are merged in #770. Binary/NUL and secret-like content retain human fallback. Expanded special-path/policy tests are published in #771; full G acceptance remains incomplete. |
| `save_url`, `fetch_url`, `http_request`, `web_search` | Protected prepared network assessment exists; ordinary product transport Auto allowance remains unimplemented and its pending contract decision is not assumed accepted. |
| `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list` | Real Agent dispatch verifies default Read Allow returns actual temporary-repository status/diff/history/commit/branch evidence and explicit Read Deny rejects all five, with zero assessor calls/grants. Complete profile/platform evidence remains outstanding. |
| `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout` | I300 prepared command route retained; no reopening of accepted whole-command Pull semantics. I308 full coverage evidence outstanding. |
| `bash` (Unix) / `powershell` (Windows), `exec` | Existing shell/direct-command paths retained; I308 full profile/decision evidence outstanding. |
| `todo_create`, `todo_create_batch`, `todo_update_batch`, `todo_update_status`, `todo_update`, `todo_delete`, `todo_add_dependency`, `todo_remove_dependency`, `todo_query` | Session-scoped Internal facets retain default deterministic Allow, not model approval. Local real-Agent tests cover all nine operations under default Allow and explicit Internal Deny, actual SQLite effects, unchanged other-session data, zero assessor calls and zero grants. MCP does not add Todo; complete profile/platform evidence remains outstanding. |
| `status` | MCP-only read tool; concrete built-in ownership checked. |
| `delay`, `schedule`, `list_scheduled_tasks`, `cancel_scheduled_task` | Product-owned scheduler factory now registers actual instances as built-ins in interactive/Print/Inline/RPC/TUI; caller-supplied scheduler instances remain external. Model-assisted scheduling allowance remains unimplemented. |

Inventory work found and repaired a production gap: Print/Inline/RPC/TUI previously
constructed scheduler tools locally but passed them through generic external registration.
The new internal factory creates and registers the instances together; it accepts no
caller-supplied tools. Generic registry constructors preserve external ownership semantics.
Desktop-specific extensions, conditional skills, process capabilities and complete feature/host
coverage still require separate verification; the table does not declare acceptance 1 complete.

Validation: locked CLI `builtin_registration_tests` passed 5/5, including rejection
of external scheduler collisions without relabeling. The CLI bin suite passed
375/375 before the final collision test was added; CLI Clippy passed with warnings
denied. Independent incremental ownership/security review found no new concrete
blocker and confirmed registration failure starts no scheduler actor. This is static
incremental review, not final exact-head approval or full T8 validation.

### Exact-input creation checkpoint

Trusted ordinary `write` now uses an additive, default-unsupported
`assess_builtin_invocation` assessor method. The provider receives exact bounded
contents, complete facets, current user intent, the existing capability-bound create
projection and a fresh nonce/digest. High-confidence matching responses return Once;
explicit human/Deny policy and admission checks remain authoritative. Existing
third-party assessors do not automatically acquire this capability. The complete
encoded input is checked before cloning and the complete model payload is capped at
16 KiB; oversized or secret-like evidence falls back to human approval. This narrows
the model-reviewed payload relative to the legacy 64 KiB content-shape classifier.

Real Agent tests use `WriteTool` and `CapStdAtomicCreateCapability`, not a logging
tool: allow creates the exact file, while uncertainty, provider error, Auto cancellation,
session rebinding and oversized input create no file and no persistent grant. Final
locked Agent library tests passed 492/492 and Agent Clippy including tests passed
with warnings denied. Independent incremental static review identified the pre-clone limit correction
and otherwise found no concrete new blocker. This stage does not enable existing-file
edits/deletes, expand network authorization or complete T8. Ordinary Execute and the
remaining H/G coverage remain unimplemented.

### Creation stage validation checkpoint

- Local implementation `9852b65b3` adds exact-content creation assessment;
  `75ffc8f9f` adds real Agent refusal cases for low confidence, mismatched digest,
  wrong schema, extra response fields, malformed JSON and oversized responses.
  All cases prove no file creation and no persistent grant; the focused test passed.
- Independent incremental static re-review confirmed the pre-clone input limit
  finding is resolved and found no new concrete blocker. It is not final whole-stage
  exact-head approval, and the reviewer did not independently rerun tests.
- Standard locked preflight at `75ffc8f9f` passed exit 0, including workspace checks,
  Clippy, tests (Tools 951/951) and both Runtime SDK consumers. Log:
  `/private/tmp/i308-creation-stage-preflight-authorized.log`. The preceding sandboxed
  run failed one process-observation test with EPERM; the authorized rerun passed.
- These commits remain local. Native Windows evidence, complete ordinary
  Write/Network/Execute coverage, full higher-risk recovery, final review, CI and
  merge are outstanding. This checkpoint does not complete T8 or any later task.

### Recoverable deletion local checkpoint — 2026-10-26

- Added optional regular-file `delete` backup/recovery without changing public
  `DeleteInput` struct literals. Backup capture declares and checks Read in addition
  to Write; restoration requires Write and never overwrites a recreated path.
  Instance/path-bound, expiring receipts reuse the existing edit store limits.
- Unix backup opening uses the existing Git integration pattern with libc
  O_NOFOLLOW/O_NONBLOCK; Windows uses the existing windows-sys reparse-point flag
  and rejects reparse metadata. The file-write feature now explicitly enables these
  already-declared system dependencies; no dependency package or unsafe code added.
- Locked file-write file-tool tests passed 86/86 and scoped tests Clippy passed with
  warnings denied. An initial default-feature run selected zero tests and is not
  evidence; a subsequent missing test import was fixed before the passing run.
- Independent incremental review identified the original blocking/FIFO and symlink
  backup-open race; the revised no-follow/nonblocking handle check closed that finding.
  It did not grant whole-stage or merge approval. Ancestor/open-to-delete external
  path races remain a limitation; no atomic rollback or concurrency isolation claimed.
- README documents actual recovery limits. Auto deletion and full G coverage remain
  unimplemented. Final full preflight, xwin/native Windows, exact-head review/CI and
  merge remain outstanding. This local checkpoint does not complete T8.

### Bounded recoverable edit follow-up — 2026-10-26

- Recoverable legacy/anchored edits now declare Read alongside Write and require
  matching resolved targets before backup capture. Original content is read through
  the shared no-follow/nonblocking regular-file handle with a 2 MiB maximum.
- Atomic revision validation uses the same bounded handle read and its permissions,
  eliminating its previous full-file read. Recoverable edits reject either before
  or after content above 2 MiB before backup insertion/mutation, so admitted backups
  remain restorable within the bound. External concurrent path replacement is still
  not isolated; this is not a general transactional rollback guarantee.
- Locked file-write file-tool tests pass 90/90, including external Read/Write
  authorization, oversized originals/replacements and enlarged revision targets.
  Log: `/private/tmp/i308-bounded-edit-tests.log`. Final-source cargo-xwin locked
  MSVC test-target checking passes; log:
  `/private/tmp/i308-file-recovery-xwin-authorized.log`. This is compilation, not
  native Windows runtime evidence. The first sandboxed xwin attempt failed cache
  permission; the authorized rerun supplied compilation evidence.
- Full H/G model review/execution coverage and final platform/review/CI/merge remain
  outstanding. No Auto edit/delete allowance or T8 completion is claimed here.

### Recovery stage full validation — 2026-10-26

- Implementation candidate `c924a3ee6` passed the complete standard preflight:
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`.
  Authorized run exited 0; log:
  `/private/tmp/i308-recovery-stage-preflight-authorized.log`.
  This includes both governance validators, formatting, locked workspace check,
  Clippy with warnings denied, workspace tests and both external Runtime SDK fixtures.
- The preceding sandbox run exited 101 with 959 tools tests passing and one
  process-observation EPERM in `prepared_local_push_runs_all_eight_admitted_stages`.
  The authorized complete rerun passed; the sandbox result is not a product pass
  and was not bypassed by excluding the test.
- This verifies the local recovery implementation stage only. Auto edit/delete
  review and execution, ordinary effectful Execute/Network coverage, remaining
  host evidence, final independent review, exact-head CI and merge are outstanding.
  T8 remains Active; no later handoff task is completed by this evidence.

### Stable foundation stage submission — 2026-10-26

- Stage delivery state: Review; the complete iteration remains Active and partial.
  The candidate contains built-in ownership/registration, retained network review,
  exact-input creation review, bounded edit/delete recovery and HTTP boundary fixes.
  This is a stable implementation stage under the existing #765 claim, not a new
  requirement or completion of H/G.
- Refreshed main to `fef542c0186dc8a3413a66396d2e77d8d8eaa666` and rebased all
  18 local commits without conflict. `git range-diff` reports every patch unchanged
  from the previous `8f0ce2340..d56feeb3a` series. New base adds the dormant I293
  SearchRouter core; the rebased stage requires fresh full validation and review.
- Previous independent security/API review found no concrete blocking issue, but
  it is not the new candidate's exact-head approval. Fresh stage CI/review and
  merge-time CAS remain required; native Windows runtime remains outstanding.
- Remaining implementation: higher-risk edit/delete model evidence and actual Auto
  execution, ordinary effectful Execute/Network admission under resolved contracts,
  complete tool/profile/host dispositions and full I308 acceptance. No later
  handoff item is completed or waived by this submission.

### 2026-10-26 Current-head validation failures and fixture isolation

At `99c828313`, two authorized parallel standard preflights each failed one
unchanged search refusal socket fixture (Tools 970 passed / 1 failed). An isolated
workspace-feature rerun passed. A temporary test-only error-chain diagnostic in
a complete library run captured `ConnectionReset` / errno 54 with
`is_connect=false`; it was removed. No production classifier change or root-cause
fix is claimed. Issue #768 owns this independent validation residual; the earlier
I300 BrokenPipe observation and this reset must not be relabelled as typed
ConnectionRefused or used to broaden failover.

The serial standard preflight then failed earlier at this stage's new CLI
inventory assertion: Print lacked all Todo tools. `default_sessions_dir()` depends
on process-global home environment, and existing init-wizard tests remove HOME.
The inventory fixture now uses the existing explicit Todo-contribution builders
with a temporary sessions directory, preserving the full expected inventory and
instance-ownership assertions without depending on ambient home state. Focused
registry tests pass; full post-fix preflight, renewed exact-head review and remote
platform evidence remain outstanding. No T8 completion or merged stage is claimed.

Local logs: `/private/tmp/i308-foundation-rebased-preflight.log`,
`/private/tmp/i308-foundation-preflight-retry.log`,
`/private/tmp/i308-search-classification-diagnostic.log`,
`/private/tmp/i308-foundation-preflight-serial.log`, and
`/private/tmp/i308-registry-isolation.log`. Serial scheduling is a diagnostic and
validation alternative, not a fix for #768 or evidence of parallel reliability.

### Foundation candidate published — 2026-10-26

- Source Issue #768 is the independent search-fixture validation residual discovered
  while validating this stage. This owner retains the evidence and index mapping;
  fixture implementation remains outside the I308 Work Slice and requires its own
  effective ownership before changes.
- Candidate `aed967c4322f942a239c4b12cd1ab9747203adfb` passed complete standard
  preflight with `RUST_TEST_THREADS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0
  CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`: exit 0 and final
  `release preflight: passed`. CLI 376/376, Tools 971/971 and both external Runtime
  SDK fixtures passed. Log: `/private/tmp/i308-foundation-preflight-serial-fixed.log`.
  This proves this serial execution, not parallel reliability or repair of #768.
- Independent read-only Agent static permission/security/API review approved that
  exact candidate without blocking findings; it did not rerun the tests and shares
  the session's tool/GitHub identity, not a separate natural-person account.
- Pushed the stable stage and opened Draft implementation PR #769. Remote CI is
  running. The initial remote reconciliation failure was the missing #768 index
  row, now synchronized here; no functional test failure is inferred from it.
- Stage remains Review; I308/T8 remains Active / partial. Native Windows runtime,
  final exact-head CI/review/CAS and all remaining H/G implementation are open.


### File mutation review implementation started — 2026-10-26

- Local continuation branch `feat/i308-file-mutation-auto` starts at the published
  foundation head `ffa35973022c18f752cacad1922325e15c648379`; #769 stays unchanged.
- Added a compatible default prepared-invocation file evidence hook and a separate
  plan-bound before/after text carrier. Each image is bounded to 2 MiB without
  truncation; relative targets are rejected and Debug hides contents. The carrier
  is not authority; dispatch must independently check built-in instance ownership.
- `cargo test --locked -p talos-core` with incremental/debug disabled passed:
  122 unit tests plus all eight integration tests. Existing downstream trait
  defaults compile. The two new tests cover plan rebinding/private Debug and both
  over-limit images. No new dependency or changed existing public struct shape.
- This is local implementation plumbing only: no file producer, dispatcher or
  Auto assessor consumes it yet. Edit/delete Auto execution is still unimplemented.
  Next implement read-admitted content capture, identity-bound mutation review,
  fresh-state checks and mandatory bounded recovery through the normal pipeline,
  then end-to-end outcome tests and independent review before publication.

### File mutation dispatch admission — 2026-10-26

- Prepared normal-tool dispatch now routes file-change evidence through an
  instance-ownership check and verifies its retained plan identity before
  resolution. Simultaneous command and file-change evidence is rejected.
- The additive resolver receives complete private images, normalized arguments,
  permission facets and user intent separately from redacted presentation. Its
  default preserves existing approval behavior and adds no Auto allowance.
- Focused locked agent pipeline tests pass 20/20, including trusted allow/deny,
  same-name external instance rejection and mismatched-plan rejection. Approved
  evidence yields an exact one-use prepared proof without persistent grants.
  `cargo fmt --all -- --check` passes.
- This stage does not implement the file-tool producer or model assessor.
  Actual edit/delete Auto execution, stale-state/recovery integration tests,
  independent security review and final platform/CI gates remain outstanding.

### Recoverable string edit producer — 2026-10-26

- Added a prepared producer for legacy string edits with `retain_backup=true`.
  Preparation performs no file read. An exact read admission captures bounded
  complete text; a fresh plan exposes before/after evidence and requires a new
  Read/Write admission before atomic replacement through the owning backup store.
  The final replacement checks the captured content digest and rejects changed
  content. Anchored edits, restore and non-backup edits retain their existing route.
- Explicit `cargo test --locked -p talos-tools --features file-write --lib
  file_tools::` with incremental/debug disabled passed 93/93. New tests exercise
  real capture, first-match replacement, owning-instance restore, missing proof
  refusal and concurrent-content preservation. Tests live in a separate file.
- The initial default-feature check did not compile this feature-gated module;
  its zero-test run is not evidence. Explicit feature validation exposed and
  fixed a provenance move and incorrect test permission setup before the passing
  run above. No default-feature success is claimed as file-write coverage.
- Changes are local and still require source/security review, final integration
  and platform validation. The Auto assessor is not implemented, and this does
  not establish model-approved edit/delete execution or T8 completion.

- Follow-up standalone file-write Clippy exposed that test dev-dependencies masked
  the production optional permission dependency. `file-write` now enables the
  existing `talos-permission` dependency directly; standalone locked Clippy with
  `--features file-write --lib -- -D warnings` passes. The producer refuses an
  oversized after image before allocating it; the additional test also proves
  the file remains unchanged, and capture/commit identities differ.

### Model-reviewed recoverable string edits — 2026-10-26

- Added a separate complete-image assessor payload and provider adapter. Auto
  dispatch reviews registered native `edit` with `retain_backup=true`; the
  payload includes exact arguments, original/result, facets, user intent and a
  fresh nonce/digest. Strict matching high-confidence `allow_once` is required.
  Permission revision/reset checks remain live, and all outcomes are one-use.
- The shared 16 KiB encoded review budget is enforced without truncation. Larger
  files may use bounded recovery but require human approval; suspected secrets,
  unsupported assessors, invalid responses and uncertain outcomes also fall back.
  Delete and anchored-edit Auto remain unimplemented, not implicitly supported.
- Agent end-to-end tests cover 13 scenarios: real replacement on allow; human
  required, assessor error, disabled/stale context, oversized request, low
  confidence, wrong digest/schema, extra response fields, malformed/oversized
  response, and a concurrent writer during review. That writer's data survives
  even when the assessor returns allow. No persistent grant is created.
- Locked agent library tests pass 494/494; locked agent library Clippy with
  `-D warnings` passes. These are local checks, not full workspace, native Windows
  or independent review evidence. Foundation PR #769 remains pending remotely
  (run 40274739232 has no jobs); the continuation is not pushed or merged.

### Captured recoverable deletion — 2026-10-26

- Added read-admitted deletion capture with a fresh final plan identity. Text files
  expose complete-original evidence to the same invocation-local model review;
  binary files retain human-approved backup/deletion without text-model evidence.
  Preparation itself performs no filesystem read. Recovery and other deletion
  modes keep their existing routes.
- Final execution consumes exact-plan Read/Write evidence, re-resolves the target,
  and checks the captured bytes under the recovery/path lock before creating a
  backup and removing the file. The returned receipt restores through the owning
  tool instance. This lock serializes participating Talos mutations; it does not
  claim exclusion of arbitrary external writers after the final check.
- Separate producer tests cover real text/binary deletion and restoration, changed
  content preservation, missing admission and distinct capture/commit identities.
  Explicit standalone file-write file-tool tests pass 94/94. Agent end-to-end
  review tests pass for edit and delete, with 13 scenarios each. These remain local
  tests; security review, final workspace/platform checks and merge are outstanding.
- String-edit assessor stage committed as `81df1fc45`. Anchored-edit Auto, ordinary
  effectful Execute and ordinary HTTP/WebSearch Auto remain unimplemented. T8 and
  all later handoff items remain incomplete.
- Independent static review found a binary-boundary omission: UTF-8 with NUL was
  treated as text. The continuation now excludes captured NUL files from deletion
  text evidence and rejects NUL in either edit image before invoking the assessor.
  Added Agent refusal cases for NUL original/replacement and a producer case proving
  NUL deletion remains human-admitted and recoverable. Review is not final approval;
  the review finding and current full-preflight outcome still require verification.
- Follow-up independent static inspection confirmed the NUL fix and found no new
  concrete blocker in the staged continuation. Focused Agent tests pass with 15 edit
  scenarios and 14 delete scenarios, including the zero-assessor NUL cases. The
  complete agent library suite passed 495/495 before the final NUL fixture addition;
  do not treat that earlier suite as exact-final evidence. Full serial locked
  preflight is running with log `/private/tmp/i308-mutation-preflight.log`; no final
  outcome, native Windows evidence or publication approval is claimed.

### Retained anchored-edit review — 2026-10-26

- Extended the existing prepared edit path to snapshot-anchored operations with
  `retain_backup=true`. Both edit schemas share read-admitted capture, complete
  before/after evidence, one-use approval and real recovery-backed execution.
  Final anchored execution also rechecks the snapshot record and captured revision;
  an invalidated snapshot does not execute even when file bytes are unchanged.
- Extracted one shared anchored-edit calculation used by ordinary and prepared
  execution. It preserves anchor/revision/overlap checks, checks the complete output
  length before allocation for recoverable edits and builds the output in one pass.
  This is implemented behavior, not a separate future refactor promise.
- Explicit standalone file-write file-tool tests pass 95/95. A separate prepared
  anchored test covers actual edit/restoration, changed content before/after capture,
  snapshot invalidation, oversize, bad anchor and overlapping operations. Agent
  production dispatch tests pass for string edit, delete and anchored edit; anchored
  review uses the same allow/uncertain/error/cancel/stale/malformed/oversize/NUL matrix.
- Full preflight started before this anchored continuation is still running at
  `/private/tmp/i308-mutation-preflight.log`. Source changed during that run, so even
  a passing result will not establish final exact-head validation for this stage.
  Final workspace/platform validation, independent review and merge remain open.
  Ordinary effectful Execute and ordinary HTTP/WebSearch Auto remain unimplemented;
  T8 and the original later handoff queue are incomplete.
- Follow-up validation: Agent library tests pass 496/496, Agent and standalone
  file-write library Clippy pass with `-D warnings`, formatting and both governance
  validators pass. Incremental independent static review found no concrete blocker;
  this is not final exact-head publication approval.
- The earlier full preflight terminated with Tools 974 passed / 1 failed:
  `prepared_local_push_runs_all_eight_admitted_stages` could not launch `ps`
  (`Operation not permitted`) to observe maintenance descendants under the sandbox.
  A full serial preflight is now running outside that sandbox with stable source;
  log: `/private/tmp/i308-mutation-preflight-final.log`. Its outcome is pending.
- Anchored continuation committed as `037bf5e55`. Locked cargo-xwin checking of
  `talos-tools --no-default-features --features file-write --tests` for
  `x86_64-pc-windows-msvc` passes; log: `/private/tmp/i308-anchored-xwin.log`.
  This proves test-target compilation only, not native Windows execution. The
  sandbox attempt could not update the cargo-xwin cache link; the authorized
  outside-sandbox rerun succeeded. Fetch of `origin/main` found no upstream-only
  commits at this checkpoint; no rebase was necessary.
- Locked Agent test-target cargo-xwin checking also passes for
  `x86_64-pc-windows-msvc`; log: `/private/tmp/i308-agent-mutation-xwin.log`.
- Independent read-only Agent permission/security/API static review APPROVE:
  base `ffa35973022c18f752cacad1922325e15c648379`, head
  `099e6bd4754e79e0eca7e8cbb4a86c09c5d15a46`. No blocking findings remain;
  the earlier NUL boundary issue is closed. Reviewer shares this session's tool
  and GitHub identity, is not an independent human, and did not independently run
  tests. Approval covers this file-mutation stage only; full preflight, native
  Windows, remote CI and merge-time CAS remain pending. No external-writer
  isolation or universal secret detection is claimed.
- Current full preflight has passed Tools 976/976, including the formerly
  sandbox-blocked eight-stage local Push test, and workspace doctests. Runtime
  SDK consumer checks are still running; no complete preflight pass is claimed.
  Both governance validators pass after the evidence update.
- Full serial locked preflight subsequently exited 0 (`release preflight: passed`),
  including both independent Runtime SDK consumer fixtures. Log:
  `/private/tmp/i308-mutation-preflight-final.log`. Production source remained
  stable from `037bf5e55`; subsequent commits only record validation evidence.
  Native Windows execution, remote exact-head CI and merge CAS remain required.

### File mutation policy coverage continuation — 2026-10-26

- File-mutation candidate published as stacked draft #770, head `400b76a67`,
  based on foundation #769. Both have live remote CI; neither is merged.
- The next local test-only continuation extends real Agent dispatch for string
  edits, anchored edits and deletions to configuration, dependency, release and
  security paths. Complete captured images and the exact target are asserted at
  the assessor boundary. These path fixtures prove routing, not semantic safety
  of arbitrary configuration contents or a real provider's judgment.
- Explicit priority Write Deny and Ask rules prevent both model review and
  mutation, leaving file bytes intact. Secret-like original/replacement fixtures
  also fall back before any assessor call. No persistent grants are created.
  An initial fixture incorrectly appended Ask after the default rule; it was
  corrected to install an effective first-match explicit rule, without changing
  production policy ordering.
- Locked Agent library tests pass 496/496 with the expanded scenarios, and Agent
  library/test Clippy passes with `-D warnings`. Logs:
  `/private/tmp/i308-mutation-policy-agent.log` and
  `/private/tmp/i308-mutation-policy-clippy.log`. This continuation has no new
  production code; publication review/platform gates remain outstanding. Full T8,
  ordinary Execute/HTTP/WebSearch Auto and all later handoff items remain open.

### Read policy execution continuation — 2026-10-26

- Separate `tests/read_auto.rs` exercises the real Read, Ls, Stat and Tree tools
  through production Agent dispatch with Auto enabled. Workspace-policy Allow
  returns actual file content/metadata/listings with zero assessor calls and zero
  persistent grants. Explicit Read Deny returns an error, also with zero assessor
  calls. Each invocation must produce exactly one tool result.
- This verifies four read operations, not the complete production/profile inventory.
  The file-mutation policy tests are committed independently as `82667ca54`.
- Locked Agent library tests pass 497/497; library/test Clippy passes with
  `-D warnings`. Logs: `/private/tmp/i308-read-policy-agent.log` and
  `/private/tmp/i308-read-policy-clippy.log`. No new production behavior or full
  workspace/native Windows validation is claimed by this test-only continuation.

### Todo deterministic admission continuation — 2026-10-26

- Separate `tests/todo_auto.rs` runs all nine actual Todo tools through Agent
  dispatch against initialized SQLite storage. Default Internal Allow performs
  actual create/batch/update/status/delete/dependency/query operations; explicit
  Internal Deny leaves rows and dependency edges unchanged. Both paths require
  exactly one result, zero assessor calls and zero persistent grants. Other-session
  data remains unchanged and absent from results.
- This corrects the earlier inventory assumption that Todo mutation needed model
  assessment: the existing session-scoped Internal policy already admits it. No
  production permission rule or authority is changed. Cross-session malicious-ID
  attempts and complete profile/platform coverage are not established by this test.
- Locked Agent library tests pass 498/498; library/test Clippy passes with
  `-D warnings`. Logs: `/private/tmp/i308-todo-agent.log` and
  `/private/tmp/i308-todo-clippy.log`. Full T8 remains incomplete.
- Issue #563 was incorrectly completed when foundation #769 merged, despite its
  remaining acceptance. It is reopened; #769's body now states the issue remains
  incomplete without a closing-keyword phrase. #771's failed reconciliation must
  be rerun after its current CI run is terminal; no product failure is inferred
  from that metadata mismatch.

### Search and diff read admission continuation — 2026-10-26

- Existing `tests/read_auto.rs` now also dispatches real grep, glob and two-file
  diff tools under default Read Allow and explicit Read Deny. Allowed results
  contain the fixture path; denied calls return errors. All seven operations
  retain zero assessor calls, zero grants and exactly one result per invocation.
- Agent's dev-only talos-tools dependency enables its existing `search` feature
  so these tests also compile in isolated package checks. No production feature
  default, dependency version or permission behavior changes.
- Locked Agent library tests pass 498/498; library/test Clippy passes with
  `-D warnings`. Logs: `/private/tmp/i308-search-read-agent.log` and
  `/private/tmp/i308-search-read-clippy.log`. Complete read-family/profile/platform
  evidence remains outstanding; these fixtures do not prove arbitrary search or
  diff content correctness.
- Todo commit `3626be38682369d3f490d285db5c86ece6f3a4b4` received independent
  read-only Agent APPROVE against base `e8e8dccdb5a7d8de281cf54206036778e45e5ac2`.
  No blocking findings; reviewer did not independently execute tests and shares
  this session's tooling/GitHub identity. Full T8 is not complete.

### Retained mutation stage merged — 2026-10-26

- #770 merged as `687b84a93b4a9f40ede1eb600151c0a22a8f2d59` after terminal
  all-success CI 40279276903, including native Windows. Independent read-only
  Agent renewed APPROVE for exact main base `d45d08f0a82706c2abb26fcb32899f8b24fa64b1`
  and head `400b76a678381421ee6a265364ae952a4744776a`: the foundation merge tree
  equals the earlier reviewed base; subsequent mutation changes only record
  evidence. No blocking findings; no independent test rerun or natural-person review.
- Merge CAS and review receipt:
  https://github.com/wjhuang88/talos/pull/770#issuecomment-6410153470
  Issue #563 remains Open and has the partial-stage update. Full T8 and later
  handoff tasks remain incomplete. #771 is still unmerged.
- Search/diff test commit `29212de48809b0147fb9f03f46df6c0e8dd18762` received
  independent read-only Agent APPROVE against `3626be38682369d3f490d285db5c86ece6f3a4b4`;
  no blockers, no independent test execution. Complete local serial locked preflight
  for this continuation is running; `/private/tmp/i308-dispositions-preflight.log`
  is not yet final success evidence.

### Serial workspace validation result — 2026-10-26

- The continuation's authorized full preflight completed successfully with
  `RUST_TEST_THREADS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0 ./scripts/release_preflight.sh`.
  The script uses locked dependency checks; both external SDK fixture runs passed
  and the terminal result is `release preflight: passed`. Evidence log:
  `/private/tmp/i308-dispositions-preflight-authorized.log`.
- The earlier sandbox run in `/private/tmp/i308-dispositions-preflight.log`
  failed while observing the prepared local push subprocess with EPERM. The
  authorized rerun passed; the earlier failure is retained rather than reported
  as success. Production/test content is `29212de48809b0147fb9f03f46df6c0e8dd18762`;
  subsequent changes record evidence only. Native Windows CI for the unpublished
  Todo/search continuation is still outstanding. I308 and T8 remain Partial.

### Todo cross-session input verification — 2026-10-26

- The real-Agent Todo matrix additionally supplies another session's item ID to
  batch update, status update, update, delete, dependency addition and dependency
  removal under default Allow and explicit Deny. Current-session rows/edges and
  the other session's item remain unchanged; results do not disclose its title.
  Missing-item delete and missing-edge removal preserve their existing successful
  no-op response, explicitly asserted as `not found`; the other foreign-target
  mutations error. Assessment and persistent-grant counts remain zero.
- Locked Agent library tests pass 498/498 after this extension; formatting passes.
  This establishes these concrete foreign-ID cases, not arbitrary cross-session
  attack coverage, new production behavior or complete I308 acceptance.

### Git read admission verification — 2026-10-26

- `tests/read_auto.rs` now dispatches all five actual Git read tools under
  default Read Allow and explicit Read Deny, alongside the seven existing file
  read/search/diff operations. Fixture initialization uses a cleared environment,
  an empty template and a local unsigned commit; no user repository is modified.
  Allowed calls must return fixture-specific path, commit message or branch
  evidence. Every call produces one result, zero model assessments and zero
  persistent grants. The focused locked production-dispatch test passes.
- This does not implement new Git behavior or prove the full feature/host/profile
  matrix. T8 remains incomplete; ordinary Execute/HTTP/WebSearch and remaining
  native families retain their outstanding requirements.

### Document and symbol read verification — 2026-10-26

- The real-Agent read matrix additionally exercises document extraction and all
  four symbol operations against text/Rust fixtures. Allowed results contain the
  actual content, function, reference path or imported type; explicit Read Deny
  rejects each operation. Each invocation has one result and no assessor calls
  or grants. The focused locked test passes across 17 operations.
- Agent's dev-only tools dependency enables existing `document` and
  `code-intelligence` features. Production defaults and dependency versions are
  unchanged. `find_symbol` takes a directory search root; its fixture uses `.`.
  Image continuation delivery, complete host/profile evidence and full T8 remain
  outstanding; this test stage does not add effectful Auto approval.
- Independent read-only Agent review approved base
  `29212de48809b0147fb9f03f46df6c0e8dd18762` through published #772 head
  `9514b0152e48f4db73fbabdbdd61f0bc9ca0c7fc`, with no blockers and no independent
  test rerun. This review does not cover the later document/symbol additions.

### Image read continuation verification — 2026-10-26

- Added the actual `read_image` tool to the existing real-Agent policy matrix,
  bringing it to 18 operations. An allowed PNG read reaches only the second
  provider request with the expected basename, MIME and byte count; explicit
  Read Deny produces no image attachment in either request. Both paths retain
  zero model assessments and zero persistent grants.
- Enabled the existing image feature only in Agent's development dependency.
  Locked Agent library tests pass 498/498, Agent library/test Clippy passes with
  warnings denied, and workspace format checking passes. These results do not
  establish native Windows or final workspace acceptance.
- Published #772 CI 40285467815 failed its Runtime conditional Checkout test:
  retained repository metadata changed after preparation. The focused local
  reproduction passes; the cause remains unestablished and is not repaired by
  this image test change. Existing Windows jobs remain running. T8 remains
  Partial, with effectful invocation families and final acceptance outstanding.
- Independent read-only Agent static review approved image commit
  `b3b1eb28cff6788f295462ab0a97e75a8167ccf8` against
  `824a276b748e702ae1bac7ca84401cfb356de701`, without blockers or an independent
  test rerun. This was a same-session Agent sharing tools/GitHub identity, not
  a separate human. Mock provider capture does not establish real provider
  acceptance, pixel decoding or longer-history attachment lifetime.
- Additional local investigation: locked Runtime library tests pass 72/72, and
  the compiled conditional Checkout test passes 30/30 isolated repetitions.
  This narrows reproduction conditions but does not explain or repair the
  published macOS CI failure. No security revalidation was weakened.

### Checkout CI diagnosis prerequisite — 2026-10-26

- Retained-repository refusal now reports only fixed changed category names
  (for example `entries` or `object_layout`), including when Tools is compiled
  as a Runtime dependency. Previously that diagnostic existed only under Tools'
  own test configuration and was absent from the failing Runtime CI output.
  No paths, configuration values or captured contents are disclosed, and the
  comparison and refusal behavior are unchanged. This is diagnostic capability,
  not a root-cause fix or permission relaxation.
- Locked Tools Git repository-state tests pass 14/14, Tools Git library/test
  Clippy passes with warnings denied, and Runtime's focused Checkout test passes.
  The initial Tools command omitted the Git feature and selected zero tests;
  only the subsequent explicit `--features git` run counts as evidence.
- Local Git is 2.54.0 while the failing CI recorded 2.55.0. Version difference
  is an observation, not an established cause. Existing Windows jobs continue;
  no failed CI rerun or production fixture change is used to claim resolution.

### Recovery invocation permission verification — 2026-10-26

- A separate real-Agent dispatch test covers restoration through both native
  edit and delete instances. Explicit Write Allow restores and consumes the
  receipt exactly once. Write Deny and Ask with a denying resolver leave the
  mutation untouched and the receipt usable. A concurrent edit or recreation
  is preserved even when current Write policy allows restoration. Every case
  returns one tool result and creates no persistent grant.
- This tests current permission enforcement on a later recovery invocation;
  it does not implement model-assisted recovery approval. The focused locked
  Agent test and Agent library/test Clippy pass. The first test attempt assumed
  edit's multiline receipt format for delete too; the fixture parser was
  corrected to accept delete's existing inline format.
- Standard serial preflight was started at `a110a12fe` before this new test
  existed. Its eventual result cannot establish final-head coverage of this
  additional test; the separate focused run supplies that evidence.
- Independent static read-only Agent review approved `a110a12fe` against
  `17c6e458f`, without blockers or independent test reruns. It confirmed fixed
  diagnostic categories expose no captured paths/values and preserve refusal.
  This same-session Agent shares tool/GitHub identity and is not a human review.
