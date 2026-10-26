# Iteration I308: Shared Auto Review For All Built-In Tools

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
| Implementation PR | Not started |
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
| `read`, `ls`, `read_image`, `document_extract`, `grep`, `glob`, `diff`, `stat`, `tree`, `find_symbol`, `find_references`, `list_symbols`, `list_imports` | Existing policy/read paths retained; complete per-operation Auto fallback/production evidence still required. MCP omits `read_image`. |
| `write`, `edit`, `delete` | Existing creation path retained; opt-in edit recovery implemented locally; higher-risk effect assessment and Auto execution still incomplete. |
| `save_url`, `fetch_url`, `http_request`, `web_search` | Protected prepared network assessment exists; ordinary product transport Auto allowance remains unimplemented and its pending contract decision is not assumed accepted. |
| `git_status`, `git_diff`, `git_log`, `git_show`, `git_branch_list` | Existing read policy retained; I308 complete disposition evidence outstanding. |
| `git_add`, `git_commit`, `git_push`, `git_pull`, `git_checkout` | I300 prepared command route retained; no reopening of accepted whole-command Pull semantics. I308 full coverage evidence outstanding. |
| `bash` (Unix) / `powershell` (Windows), `exec` | Existing shell/direct-command paths retained; I308 full profile/decision evidence outstanding. |
| `todo_create`, `todo_create_batch`, `todo_update_batch`, `todo_update_status`, `todo_update`, `todo_delete`, `todo_add_dependency`, `todo_remove_dependency`, `todo_query` | Print/TUI session-scoped registrations inventoried; mutation Auto assessment and full disposition evidence outstanding. MCP does not add Todo. |
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
